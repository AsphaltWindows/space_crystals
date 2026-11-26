//! The Gunner's TailDisjointed attack in play (slice M5-O3; `factions/cults/units/gunner.md > AttackAttributes`, design
//! change 0304; `systems/combat.md > TailDisjointed`), with its own numbers, through the gate: "the target's position is
//! locked at the end of Aiming and the shot lands there at the end of Firing, so a target that moves during the Firing
//! phase is missed, and whatever unit stands at the spot then is hit instead (any owner)"; and its Attack Ground,
//! which "keeps firing at the location" until another order (`systems/controls/command_behaviors.md >
//! AttackingLocation`).
//!
//! Its Firing lasts 4 frames (design change 0333, answering Q0126: "at 4 a walking infantry unit can step off it"): a
//! Peacekeeper walking across the line of fire, 16 su in those frames against its Silhouette's 12 su half-width, is
//! missed, and a target that stands still is hit by every shot (0333's Verification). The other tests take the target
//! off the spot at once, with a debug teleport. Its MinRange against a location is in `tests/cults_combat.rs`.
//!
//! Every test runs under the invariants of `TestSim`.

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    AttackPhase, DebugCommand, ObjectId, ObjectKind, OrderKind, PlayerId, SimEvent, SpawnAt, Target, Tick, TilePos,
    Vec2, gu,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const GUNNER: ObjectId = ObjectId(1);
const TARGET: ObjectId = ObjectId(2);
/// `factions/cults/units/gunner.md > AttackAttributes`: "Damage: 16".
const DAMAGE: f32 = 16.0;
/// `factions/cults/units/gunner.md > AttackAttributes`: "FiringDuration: 4 frames" (design change 0333).
const FIRING: Tick = 4;
/// `factions/cults/units/gunner.md > AttackAttributes`: one shot every Aim 4 + Firing 4 + Cooldown 6 + Reload 8
/// frames (design change 0333: "so the rate of fire is unchanged").
const CYCLE: Tick = 22;
/// Half a LightInfantry's 24 su Silhouette: the Gunner's, the Militant's and the Peacekeeper's.
const HALF: f32 = 12.0;

/// 16 × 10 open tiles.
fn open() -> String {
    "................\n".repeat(10)
}

/// What a shot did, from its event: what it struck at the spot (`None`: it missed), from where, to where.
#[derive(Copy, Clone, Debug, PartialEq)]
struct Shot {
    tick: Tick,
    struck: Option<ObjectId>,
    from: Vec2,
    to: Vec2,
}

/// The Gunner's shots so far: its hits (`SimEvent::Attacked`) and its misses (`SimEvent::Missed`), aimed at `aimed`
/// (none: at the ground) for the misses.
fn shots(t: &TestSim, aimed: Option<ObjectId>) -> Vec<Shot> {
    let shot = |(tick, e): &(Tick, SimEvent)| match *e {
        SimEvent::Attacked {
            attacker: GUNNER,
            target,
            from,
            to,
        } => Some(Shot {
            tick: *tick,
            struck: Some(target),
            from,
            to,
        }),
        SimEvent::Missed {
            attacker: GUNNER,
            target,
            from,
            to,
        } => {
            assert_eq!(target, aimed, "the miss names what it was aimed at");
            Some(Shot {
                tick: *tick,
                struck: None,
                from,
                to,
            })
        }
        _ => None,
    };
    t.events().iter().filter_map(shot).collect()
}

/// The HP damage `id` took in each hit of the run so far: (tick, damage).
fn hits_on(t: &TestSim, id: ObjectId) -> Vec<(Tick, f32)> {
    let hit = |(tick, e): &(Tick, SimEvent)| match *e {
        SimEvent::Hit { target, damage, .. } if target == id => Some((*tick, damage)),
        _ => None,
    };
    t.events().iter().filter_map(hit).collect()
}

fn current(t: &TestSim, id: ObjectId) -> Option<OrderKind> {
    let orders = t.sim().object(id).expect("alive").orders.as_ref().expect("a unit");
    orders.current.as_ref().map(|active| active.order.kind)
}

fn queued(t: &TestSim, id: ObjectId) -> Vec<OrderKind> {
    let orders = t.sim().object(id).expect("alive").orders.as_ref().expect("a unit");
    orders.queue.iter().map(|order| order.kind).collect()
}

/// Runs until the Gunner's Aiming ends, and returns the tick and the spot it locked (`AttackPhase::FiringAt`).
fn until_locked(t: &mut TestSim) -> (Tick, Vec2) {
    loop {
        assert!(t.sim().tick() < 300, "it never locked a spot\n{}", t.ascii(None));
        t.step();
        let attack = t
            .sim()
            .object(GUNNER)
            .expect("alive")
            .attack
            .as_ref()
            .expect("it attacks");
        if let Some(AttackPhase::FiringAt { at, .. }) = attack.phase
            && attack.elapsed == 0
        {
            return (t.sim().tick(), at);
        }
    }
}

/// Player 1's Gunner, and player 2's Peacekeeper 3 grid units east of its edge (within its band of 2 to 5), which the
/// Gunner is ordered to attack in tick 2. Auto-targeting is off: the Peacekeeper doesn't fight back, and nobody moves
/// but by the test's hand.
fn duel() -> (TestSim, Vec2) {
    let from = Vec2::new(300.0, 320.0);
    let there = from + Vec2::new(2.0 * HALF + gu(3.0), 0.0);
    let mut t = ScenarioBuilder::new(&open()).start();
    t.without_auto_targeting()
        .spawn_unit_at(1, ObjectKind::Gunner, P1, from)
        .spawn_unit_at(1, ObjectKind::Peacekeeper, P2, there)
        .order_at(2, P1, &[GUNNER], OrderKind::Attack(TARGET), false);
    (t, there)
}

/// Acceptance, "a Gunner's shot hits a target that stands still": the spot is locked where the Peacekeeper stands at
/// the end of Aiming, and the shot lands there at the end of Firing, four frames on, on the Peacekeeper still there:
/// 16 less its PointArmor 1. Its event runs from the Gunner to the spot. One shot a cycle.
#[test]
fn a_shot_at_a_target_that_stands_still_hits_it() {
    let (mut t, there) = duel();
    let (locked, at) = until_locked(&mut t);
    assert_eq!(at, there, "where the Peacekeeper stands");
    let from = t.sim().object(GUNNER).unwrap().pos;
    t.run_until(locked + FIRING - 1);
    assert!(shots(&t, Some(TARGET)).is_empty(), "not before the end of Firing");
    t.step();
    let hit = Shot {
        tick: locked + FIRING,
        struck: Some(TARGET),
        from,
        to: at,
    };
    assert_eq!(shots(&t, Some(TARGET)), [hit]);
    assert_eq!(hits_on(&t, TARGET), [(locked + FIRING, DAMAGE - 1.0)]);
    t.run_until(locked + FIRING + CYCLE);
    let next = Shot {
        tick: locked + FIRING + CYCLE,
        ..hit
    };
    assert_eq!(shots(&t, Some(TARGET)), [hit, next], "the next shot a cycle later");
}

/// Acceptance, "it misses a target that leaves the locked spot during Firing (16 damage not dealt)": the Peacekeeper is
/// taken two tiles south in the first frame of Firing. The shot lands on the spot all the same, empty now: a
/// `SimEvent::Missed`, drawn to the spot, and no damage to anyone; the Peacekeeper has its 50 HP. The Attack goes on,
/// and the next shot, aimed where the Peacekeeper stands now, hits it there.
#[test]
fn a_target_that_leaves_the_spot_during_firing_is_missed() {
    let (mut t, there) = duel();
    let (locked, at) = until_locked(&mut t);
    let from = t.sim().object(GUNNER).unwrap().pos;
    let gone = there + Vec2::new(0.0, gu(2.0));
    t.debug_at(locked + 1, DebugCommand::SetPosition { id: TARGET, pos: gone })
        .run_until(locked + FIRING);
    let miss = Shot {
        tick: locked + FIRING,
        struck: None,
        from,
        to: at,
    };
    assert_eq!(shots(&t, Some(TARGET)), [miss]);
    assert!(hits_on(&t, TARGET).is_empty(), "16 damage not dealt");
    assert!(
        !t.events().iter().any(|(_, e)| matches!(e, SimEvent::Hit { .. })),
        "to anyone"
    );
    assert_eq!(t.sim().object(TARGET).unwrap().hp, Some(50.0));
    assert_eq!(
        current(&t, GUNNER),
        Some(OrderKind::Attack(TARGET)),
        "the Attack goes on"
    );
    t.run_until(locked + FIRING + CYCLE);
    let next = Shot {
        tick: locked + FIRING + CYCLE,
        struck: Some(TARGET),
        from,
        to: gone,
    };
    assert_eq!(shots(&t, Some(TARGET)), [miss, next]);
    assert_eq!(hits_on(&t, TARGET), [(next.tick, DAMAGE - 1.0)]);
}

/// Design change 0333's Verification, "A Peacekeeper walking across a Gunner's line is sometimes missed: the tracer
/// ends on empty ground". Player 2's Peacekeeper, 3 grid units east of the Gunner's edge and 3 tiles north of its line
/// of fire, walks due south across the line, 6 tiles at its 4 su a frame without stopping, while the Gunner attacks it
/// (within its band of 2 to 5 grid units all the way). Each spot is locked where the Peacekeeper is at the end of
/// Aiming; in the 4 frames of Firing it walks 16 su on, past its Silhouette's 12 su half-height, so every shot of its
/// walk lands on empty ground: a `SimEvent::Missed` to the spot, which no object's shape covers then, and no damage.
#[test]
fn a_peacekeeper_walking_across_the_line_of_fire_is_missed() {
    let from = Vec2::new(300.0, 320.0);
    let x = from.x + 2.0 * HALF + gu(3.0);
    let (start, end) = (Vec2::new(x, from.y - gu(3.0)), Vec2::new(x, from.y + gu(3.0)));
    let mut t = ScenarioBuilder::new(&open()).start();
    t.without_auto_targeting()
        .spawn_unit_at(1, ObjectKind::Gunner, P1, from)
        .spawn_unit_at(1, ObjectKind::Peacekeeper, P2, start)
        .order_at(2, P2, &[TARGET], OrderKind::Move(Target::Point(end)), false)
        .order_at(2, P1, &[GUNNER], OrderKind::Attack(TARGET), false)
        .run_until(2);
    let covered = |t: &TestSim, at: Vec2| {
        t.sim().objects().any(|o| {
            let half = o.def().half_extents().expect("units");
            (at - o.pos).abs().cmple(half).all()
        })
    };
    let mut seen = 0;
    while t.sim().object(TARGET).expect("alive").pos != end {
        assert!(t.sim().tick() < 200, "it never arrived\n{}", t.ascii(None));
        t.step();
        let all = shots(&t, Some(TARGET));
        for shot in &all[seen..] {
            assert_eq!((shot.struck, shot.from), (None, from), "tick {}: a miss", shot.tick);
            assert!(
                !covered(&t, shot.to),
                "tick {}: the spot {} is empty ground",
                shot.tick,
                shot.to
            );
        }
        seen = all.len();
    }
    assert!(seen >= 3, "it fired {seen} times across the walk");
    assert!(hits_on(&t, TARGET).is_empty(), "16 damage never dealt");
    assert_eq!(t.sim().object(TARGET).unwrap().hp, Some(50.0));
}

/// Design change 0333's Verification, "A standing target is always hit": the Peacekeeper of [`duel`] stands still, and
/// each of the Gunner's first three shots lands at the spot locked where it stands, on it, a cycle apart: three hits of
/// 16 less its PointArmor 1, and no miss.
#[test]
fn a_standing_target_is_hit_by_every_shot() {
    let (mut t, there) = duel();
    let (locked, _) = until_locked(&mut t);
    let first = locked + FIRING;
    t.run_until(first + 2 * CYCLE);
    let struck: Vec<(Tick, Option<ObjectId>, Vec2)> = (shots(&t, Some(TARGET)).iter())
        .map(|shot| (shot.tick, shot.struck, shot.to))
        .collect();
    let hit = |i: Tick| (first + i * CYCLE, Some(TARGET), there);
    assert_eq!(struck, [hit(0), hit(1), hit(2)]);
    let taken: Vec<f32> = hits_on(&t, TARGET).iter().map(|&(_, damage)| damage).collect();
    assert_eq!(taken, [DAMAGE - 1.0; 3]);
    assert_eq!(t.sim().object(TARGET).unwrap().hp, Some(50.0 - 3.0 * (DAMAGE - 1.0)));
}

/// Acceptance, "it hits whatever other unit, of any owner, stands on the spot at the end of Firing": in the first frame
/// of Firing the Peacekeeper is taken off the spot and one of the Gunner's own side, a Militant of player 1, put on it.
/// The shot strikes the Militant (16, no armour), and the Peacekeeper takes nothing.
#[test]
fn whatever_stands_on_the_spot_is_hit_in_its_place_its_own_side_too() {
    let militant = ObjectId(3);
    let (mut t, there) = duel();
    t.spawn_unit_at(1, ObjectKind::Militant, P1, Vec2::new(300.0, 500.0));
    let (locked, at) = until_locked(&mut t);
    let from = t.sim().object(GUNNER).unwrap().pos;
    let gone = there + Vec2::new(0.0, -gu(2.0));
    t.debug_at(locked + 1, DebugCommand::SetPosition { id: TARGET, pos: gone })
        .debug_at(locked + 1, DebugCommand::SetPosition { id: militant, pos: at })
        .run_until(locked + FIRING);
    let struck = Shot {
        tick: locked + FIRING,
        struck: Some(militant),
        from,
        to: at,
    };
    assert_eq!(shots(&t, Some(TARGET)), [struck]);
    assert_eq!(hits_on(&t, militant), [(locked + FIRING, DAMAGE)]);
    assert!(hits_on(&t, TARGET).is_empty());
    assert_eq!(t.sim().object(militant).unwrap().owner, Some(P1), "the shooter's own");
}

/// A structure stands still as well: the Gunner ordered to attack player 2's Generator, a 2x2 (`factions/gdo/economy.md >
/// Generator`), locks the spot at its centre, and the shot lands there, on its footprint: 16 less its PointArmor 1.
/// What stands at the spot is any object's shape there, a footprint as well as a Silhouette, or a Gunner could never
/// damage a building (slice M5-O3's reading of "whatever unit stands at the spot").
#[test]
fn a_shot_at_a_structure_hits_it() {
    let generator = ObjectId(2);
    let from = Vec2::new(300.0, 320.0);
    let mut t = ScenarioBuilder::new(&open()).start();
    t.without_auto_targeting()
        .spawn_unit_at(1, ObjectKind::Gunner, P1, from)
        .debug_at(
            1,
            DebugCommand::Spawn {
                kind: ObjectKind::Generator,
                owner: Some(P2),
                at: SpawnAt::Tile(TilePos::new(8, 4)),
                orientation: Default::default(),
            },
        )
        .order_at(2, P1, &[GUNNER], OrderKind::Attack(generator), false);
    let (locked, at) = until_locked(&mut t);
    assert_eq!(at, Vec2::new(gu(9.0), gu(5.0)), "the centre of its footprint");
    t.run_until(locked + FIRING);
    let hit = Shot {
        tick: locked + FIRING,
        struck: Some(generator),
        from,
        to: at,
    };
    assert_eq!(shots(&t, Some(generator)), [hit]);
    assert_eq!(hits_on(&t, generator), [(hit.tick, DAMAGE - 1.0)]);
}

/// Acceptance, Attack Ground: "a Gunner ordered to attack a location fires there again and again until Stop or
/// another order. A queued order after it never starts on its own" (design change 0293: "like an attack on a unit
/// that never dies"). The point is 3 grid units from its edge: one shot a cycle at the empty ground, each a
/// `SimEvent::Missed` aimed at nothing, the order current all along and the Move Shift-queued behind it waiting. A
/// Peacekeeper of player 2 put on the point meanwhile is struck by the next shot. Stop ends it: no shot after it, and
/// the queued Move never starts.
#[test]
fn an_attack_ground_fires_at_the_point_until_stop() {
    let from = Vec2::new(300.0, 320.0);
    let at = from + Vec2::new(HALF + gu(3.0), 0.0);
    let then = OrderKind::Move(Target::Point(Vec2::new(300.0, 500.0)));
    let mut t = ScenarioBuilder::new(&open()).start();
    t.without_auto_targeting()
        .spawn_unit_at(1, ObjectKind::Gunner, P1, from)
        .order_at(2, P1, &[GUNNER], OrderKind::AttackGround(at), false)
        .order_at(2, P1, &[GUNNER], then, true);
    let (locked, spot) = until_locked(&mut t);
    assert_eq!(spot, at, "the point clicked");
    let first = locked + FIRING;
    t.run_until(first + 2 * CYCLE);
    let missed = |i: Tick| Shot {
        tick: first + i * CYCLE,
        struck: None,
        from,
        to: at,
    };
    assert_eq!(shots(&t, None), [missed(0), missed(1), missed(2)]);
    assert_eq!(current(&t, GUNNER), Some(OrderKind::AttackGround(at)), "it goes on");
    assert_eq!(queued(&t, GUNNER), [then], "the queued Move waits");

    let peacekeeper = ObjectId(2);
    t.spawn_unit_at(first + 2 * CYCLE + 1, ObjectKind::Peacekeeper, P2, at)
        .run_until(first + 3 * CYCLE);
    let struck = Shot {
        struck: Some(peacekeeper),
        ..missed(3)
    };
    assert_eq!(shots(&t, None)[3..], [struck]);
    assert_eq!(hits_on(&t, peacekeeper), [(struck.tick, DAMAGE - 1.0)]);

    let stopped = first + 3 * CYCLE + 1;
    t.order_at(stopped, P1, &[GUNNER], OrderKind::Stop, false)
        .run(3 * CYCLE);
    assert_eq!(shots(&t, None).len(), 4, "none after the Stop");
    assert_eq!(current(&t, GUNNER), None, "stopped");
    assert_eq!(queued(&t, GUNNER), [], "the queued Move never started");
    assert_eq!(t.sim().object(GUNNER).unwrap().pos, from, "where it stood");
}
