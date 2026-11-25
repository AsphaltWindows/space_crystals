//! The Cults' units in combat, with their own numbers, through the gate (slice M5-A, plan §3.3 A): a debug-spawned
//! Militant (`factions/cults/units/militant.md`) fights a Peacekeeper to a result, and the Gunner
//! (`factions/cults/units/gunner.md`), the first kind of the design with a MinRange, never fires at a target nearer
//! than 2 grid units (`systems/controls/behaviors.md > AttackingObject > Target inside MinRange`,
//! `systems/controls/behaviors.md > BaseAutoTargeting > Targets inside MinRange`; slice M3-L2's code: plan decision
//! T55). It backs off from such a target, and fails its Attack with a `SimEvent::TargetTooClose` once it can't back
//! off further (design change 0204 point 3, which slice M5-A added to that code). A Gunner that can't move at all
//! fails at once: `sc_sim::orders`' test, since none can yet, so it needs a test def.
//!
//! Every test runs under the invariants of `TestSim`.

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{ObjectId, ObjectKind, OrderKind, PlayerId, SimEvent, Tick, Vec2, gu};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
/// Half a LightInfantry's 24 su Silhouette: the Militant's, the Gunner's and the Peacekeeper's.
const HALF: f32 = 12.0;
/// `factions/cults/units/gunner.md > AttackAttributes`: "Range: 5 grid units".
const RANGE_GU: f32 = 5.0;
/// `factions/cults/units/gunner.md > AttackAttributes`: "MinRange: 2".
const MIN_RANGE_GU: f32 = 2.0;

/// 16 × 10 open tiles.
fn open() -> String {
    "................\n".repeat(10)
}

/// The HP damage `id` took in each hit of the run so far.
fn hits_on(t: &TestSim, id: ObjectId) -> Vec<f32> {
    let hit = |(_, e): &(Tick, SimEvent)| match *e {
        SimEvent::Hit { target, damage, .. } if target == id => Some(damage),
        _ => None,
    };
    t.events().iter().filter_map(hit).collect()
}

/// The ticks of `attacker`'s attacks on `target` so far.
fn attacks(t: &TestSim, attacker: ObjectId, target: ObjectId) -> Vec<Tick> {
    let attack = |(tick, e): &(Tick, SimEvent)| match *e {
        SimEvent::Attacked {
            attacker: a, target: b, ..
        } if a == attacker && b == target => Some(*tick),
        _ => None,
    };
    t.events().iter().filter_map(attack).collect()
}

/// The gap between two units, edge to edge (`systems/combat.md > Range Measurement`), both side by side on a row.
fn gap(t: &TestSim, a: ObjectId, b: ObjectId) -> f32 {
    let pos = |id| t.sim().object(id).expect("alive").pos;
    pos(a).distance(pos(b)) - 2.0 * HALF
}

fn current(t: &TestSim, id: ObjectId) -> Option<OrderKind> {
    let orders = t.sim().object(id).expect("alive").orders.as_ref().expect("a unit");
    orders.current.as_ref().map(|active| active.order.kind)
}

/// Plan §3.3 A's acceptance, "A debug-spawned Militant fights a Peacekeeper to a result". Player 1's Militant and player
/// 2's Peacekeeper, debug-spawned 3 grid units apart, within both their Ranges of 4, take each other by auto-targeting.
/// The Militant's 4 damage loses 1 to the Peacekeeper's PointArmor, 3 a shot, one every 8 frames; the Peacekeeper's 10
/// meet no armour, one shot every 17 frames, and its fourth takes the Militant's 40 HP. The Militant dies; the
/// Peacekeeper stands, with 50 HP less what the Militant's shots took, and nothing else hit it.
#[test]
fn a_militant_fights_a_peacekeeper_to_a_result() {
    let (militant, peacekeeper) = (ObjectId(1), ObjectId(2));
    let at = Vec2::new(300.0, 320.0);
    let mut t = ScenarioBuilder::new(&open()).start();
    t.spawn_unit_at(1, ObjectKind::Militant, P1, at)
        .spawn_unit_at(
            1,
            ObjectKind::Peacekeeper,
            P2,
            at + Vec2::new(2.0 * HALF + gu(3.0), 0.0),
        )
        .run_until(1);
    assert_eq!(gap(&t, militant, peacekeeper), gu(3.0));
    while t.sim().object(militant).is_some() && t.sim().object(peacekeeper).is_some() {
        assert!(t.sim().tick() < 200, "no result\n{}", t.ascii(None));
        t.step();
    }
    let died: Vec<(ObjectId, ObjectKind)> = (t.events().iter())
        .filter_map(|(_, e)| match *e {
            SimEvent::Died { id, kind, .. } => Some((id, kind)),
            _ => None,
        })
        .collect();
    assert_eq!(died, [(militant, ObjectKind::Militant)]);
    assert_eq!(hits_on(&t, militant), [10.0; 4], "four shots of 10, no armour");
    let taken = hits_on(&t, peacekeeper);
    assert!(
        taken.len() >= 4 && taken.iter().all(|&damage| damage == 3.0),
        "{taken:?}"
    );
    let hp = t.sim().object(peacekeeper).expect("it stands").hp;
    assert_eq!(hp, Some(50.0 - 3.0 * taken.len() as f32));
}

/// The Gunner ordered to attack a Peacekeeper of player 2 that hugs it, 16 su off (auto-targeting off: the Peacekeeper
/// doesn't fight back). `systems/controls/behaviors.md > AttackingObject > Target inside MinRange`: "a unit that can
/// move **backs off** until the target is outside MinRange, then attacks": it
/// walks straight away from the Peacekeeper, never firing while it is within 2 grid units, its Attack order running
/// all the while; then it stops within its Range and fires, and its shot takes 16 less the Peacekeeper's PointArmor 1.
/// It stands while it attacks.
#[test]
fn a_gunner_backs_off_from_a_target_inside_its_min_range_then_fires() {
    let (gunner, target) = (ObjectId(1), ObjectId(2));
    let start = Vec2::new(400.0, 320.0);
    let mut t = ScenarioBuilder::new(&open()).start();
    t.without_auto_targeting()
        .spawn_unit_at(1, ObjectKind::Gunner, P1, start)
        .spawn_unit_at(
            1,
            ObjectKind::Peacekeeper,
            P2,
            start + Vec2::new(2.0 * HALF + 16.0, 0.0),
        )
        .order_at(2, P1, &[gunner], OrderKind::Attack(target), false)
        .run_until(2);
    assert!(gap(&t, gunner, target) < gu(MIN_RANGE_GU));
    while attacks(&t, gunner, target).is_empty() {
        assert!(t.sim().tick() < 120, "it never fired\n{}", t.ascii(None));
        let before = gap(&t, gunner, target);
        t.step();
        assert_eq!(
            current(&t, gunner),
            Some(OrderKind::Attack(target)),
            "tick {}",
            t.sim().tick()
        );
        if before < gu(MIN_RANGE_GU) {
            let x = t.sim().object(gunner).unwrap().pos.x;
            assert!(x <= start.x, "tick {}: away from the Peacekeeper", t.sim().tick());
        }
    }
    let fired = gap(&t, gunner, target);
    assert!(
        (gu(MIN_RANGE_GU)..=gu(RANGE_GU)).contains(&fired),
        "fired across a gap of {fired}"
    );
    let stood = t.sim().object(gunner).unwrap().pos;
    assert!(stood.x < start.x && stood.y == start.y, "straight back: {stood}");
    assert_eq!(hits_on(&t, target), [15.0], "16 less PointArmor 1");
    t.run(60);
    assert_eq!(t.sim().object(gunner).unwrap().pos, stood, "it stands while it attacks");
    assert!(attacks(&t, gunner, target).len() >= 2);
}

/// `systems/controls/behaviors.md > AttackingObject > Target inside MinRange`: "A unit that can't back off (cornered,
/// or blocked behind): it gives up the order with the same "Target too close" feedback, like an immobile unit" (design
/// change 0204 point 3). The Gunner stands with its back to the map's west edge, and a Peacekeeper of player 2 hugs
/// it from the east (auto-targeting off). Ordered to attack it, the Gunner backs off as far as the edge lets it, and
/// fails the order there with a `SimEvent::TargetTooClose`, its one feedback; it never fires, and stays where it
/// stopped.
#[test]
fn a_cornered_gunner_fails_its_attack_with_target_too_close() {
    let (gunner, target) = (ObjectId(1), ObjectId(2));
    let start = Vec2::new(HALF + 4.0, 320.0);
    let mut t = ScenarioBuilder::new(&open()).start();
    t.without_auto_targeting()
        .spawn_unit_at(1, ObjectKind::Gunner, P1, start)
        .spawn_unit_at(1, ObjectKind::Peacekeeper, P2, start + Vec2::new(2.0 * HALF + 8.0, 0.0))
        .order_at(2, P1, &[gunner], OrderKind::Attack(target), false)
        .run_until(2);
    let too_close = |t: &TestSim| -> Vec<(Tick, SimEvent)> {
        let failed = |(_, e): &&(Tick, SimEvent)| matches!(e, SimEvent::TargetTooClose { .. });
        t.events().iter().filter(failed).cloned().collect()
    };
    while too_close(&t).is_empty() {
        assert!(t.sim().tick() < 200, "it never gave up\n{}", t.ascii(None));
        assert_eq!(
            current(&t, gunner),
            Some(OrderKind::Attack(target)),
            "tick {}",
            t.sim().tick()
        );
        t.step();
    }
    let failed = too_close(&t);
    assert_eq!(failed.len(), 1, "{failed:?}");
    assert_eq!(failed[0].1, SimEvent::TargetTooClose { unit: gunner, target });
    assert_eq!(current(&t, gunner), None, "the order ended");
    assert!(gap(&t, gunner, target) < gu(MIN_RANGE_GU));
    let stood = t.sim().object(gunner).unwrap().pos;
    assert!(
        stood.x < start.x && stood.y == start.y,
        "it backed straight off: {stood}"
    );
    t.run(40);
    assert_eq!(attacks(&t, gunner, target), [] as [Tick; 0]);
    assert_eq!(t.sim().object(gunner).unwrap().pos, stood);
    assert_eq!(too_close(&t).len(), 1);
}

/// `systems/controls/behaviors.md > BaseAutoTargeting > Targets inside MinRange`: an idle Gunner, with a Peacekeeper
/// of player 2 hugging it and
/// another 3 grid units off, beyond its MinRange and within its Range, both on Hold, picks the far one, though the near
/// one is closer, and never fires at the near one.
#[test]
fn an_idle_gunner_skips_an_enemy_inside_its_min_range() {
    let (gunner, near, far) = (ObjectId(1), ObjectId(2), ObjectId(3));
    let at = Vec2::new(400.0, 320.0);
    let mut t = ScenarioBuilder::new(&open()).start();
    t.spawn_unit_at(1, ObjectKind::Gunner, P1, at)
        .spawn_unit_at(1, ObjectKind::Peacekeeper, P2, at + Vec2::new(2.0 * HALF + 8.0, 0.0))
        .spawn_unit_at(
            1,
            ObjectKind::Peacekeeper,
            P2,
            at + Vec2::new(0.0, 2.0 * HALF + gu(3.0)),
        )
        .order_at(2, P2, &[near, far], OrderKind::Hold, false)
        .run_until(16);
    let engaged = t.sim().object(gunner).expect("alive").attack.as_ref().unwrap().target;
    assert_eq!(engaged, Some(far));
    assert_eq!(attacks(&t, gunner, near), [] as [Tick; 0]);
    assert!(!attacks(&t, gunner, far).is_empty());
}
