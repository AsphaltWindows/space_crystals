//! (M5-U1) A unit's turret's behaviour in phase 5a (`super`), and what the base's behaviours set of it, on the test
//! turret units (`combat::turret::testing`): plan U1's acceptance of the scan (`systems/controls/behaviors.md >
//! TurretAutonomousScanning`), its order, MinRange skip and facing; the retention rule
//! (`systems/controls/commands.md > TurretCommandState`), and when a lock ends (design change 0376, answering Q0137:
//! `systems/controls/commands.md > TurretCommandState > When a lock ends`); a Move's scan, which never stops the base;
//! an Attack, a Hold and a Stop.

use glam::Vec2;

use crate::combat::inside_min_range;
use crate::combat::turret::testing::{
    GUN, TURRET_UNIT, current, facing_degrees, locked, open, shots, toward, turret, turret_of, turret_unit,
};
use crate::command::DebugCommand;
use crate::defs::{AttackAttrs, ObjectKind, TurretAttrs};
use crate::ids::{ObjectId, PlayerId};
use crate::orders::{OrderKind, Target};
use crate::shape::Shape;
use crate::testing::{ScenarioBuilder, TestSim};
use crate::units::{Tick, gu};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const UNIT: ObjectId = ObjectId(1);

/// An enemy to place: its kind, and where (or its offset from the turret unit).
type Enemy = (ObjectKind, Vec2);

/// The gap between objects `a` and `b` (`systems/combat.md > Range Measurement`).
fn gap(t: &TestSim, a: ObjectId, b: ObjectId) -> f32 {
    let shape = |id| Shape::of(t.sim().object(id).expect("exists"));
    shape(a).gap(&shape(b))
}

/// What unit `id`'s base engages (`AttackState::target`).
fn engaged(t: &TestSim, id: ObjectId) -> Option<ObjectId> {
    t.sim().object(id).unwrap().attack.as_ref().unwrap().target
}

/// A test turret unit at `at` facing east, and player 2's `enemies` placed around it (ids from 2 on, in order), on a
/// field of `wide` × 10 open tiles; the sim is at tick 1, after the spawns, and nothing is ordered yet.
fn field(wide: usize, at: Vec2, enemies: &[Enemy]) -> TestSim {
    let mut t = ScenarioBuilder::new(&open(wide, 10)).start();
    t.spawn_unit_at(1, TURRET_UNIT, P1, at);
    for &(kind, pos) in enemies {
        t.spawn_unit_at(1, kind, P2, pos);
    }
    t.run_until(1);
    assert_eq!(facing_degrees(&t, UNIT), 0.0, "it starts facing east");
    t
}

/// The enemy that a turret of `attrs` locks in its first tick, holding position (its base engages nothing,
/// `orders::engage::hold`) at the centre of the field with `enemies` around it, each a kind and its centre's offset
/// from the unit's: their ids are 2 on, in order.
fn first_lock(attrs: TurretAttrs, enemies: &[Enemy]) -> Option<u32> {
    let _def = turret_unit(attrs, GUN);
    let at = Vec2::new(500.0, 320.0);
    let placed: Vec<_> = enemies.iter().map(|&(kind, offset)| (kind, at + offset)).collect();
    let mut t = field(16, at, &placed);
    t.order_at(2, P1, &[UNIT], OrderKind::Hold, false).step();
    locked(&t, UNIT).map(|id| id.0)
}

/// Plan U1's acceptance: the scan's order (`systems/controls/behaviors.md > TurretAutonomousScanning > Target
/// Selection Priority`), its design's example among them: "an enemy at max range straight ahead and an adjacent
/// enemy 5° to the side are in the same step, so the adjacent one is chosen". Threatening targets first (a
/// Peacekeeper's attack targets the ground, the turret unit's domain; a Drone has none), then the least rotation in
/// 45° steps, a step up to its upper bound, then the closest, then the lowest id; only within Range ("turret remains
/// inactive") and within the arc (Algorithm 1: "within Range and within TurnAngle arc"). The test gun reaches 192 su.
#[test]
fn a_turret_s_scan_takes_the_threatening_then_the_least_rotation_then_the_closest() {
    let (drone, peacekeeper) = (ObjectKind::Drone, ObjectKind::Peacekeeper);
    let full = turret(360.0, 12.0);
    let ninety = turret(90.0, 10.0);
    let off = |degrees: f32, distance: f32| toward(Vec2::ZERO, degrees, distance);
    let rows: Vec<(&str, TurretAttrs, Vec<Enemy>, Option<u32>)> = vec![
        (
            "the design's example: one step, the closer",
            full,
            vec![(drone, off(0.0, 227.0)), (drone, off(5.0, 40.0))],
            Some(3),
        ),
        (
            "less rotation beats nearer",
            full,
            vec![(drone, off(70.0, 60.0)), (drone, off(10.0, 200.0))],
            Some(3),
        ),
        (
            "45° is the first step's",
            full,
            vec![(drone, Vec2::new(100.0, 100.0)), (drone, Vec2::new(136.0, 0.0))],
            Some(2),
        ),
        (
            "just over 45° is the second step's",
            full,
            vec![(drone, Vec2::new(100.0, 101.0)), (drone, Vec2::new(136.0, 0.0))],
            Some(3),
        ),
        (
            "threatening first",
            full,
            vec![(drone, off(0.0, 60.0)), (peacekeeper, off(150.0, 150.0))],
            Some(3),
        ),
        (
            "ties: the lowest id",
            full,
            vec![(drone, Vec2::new(110.0, 50.0)), (drone, Vec2::new(110.0, -50.0))],
            Some(2),
        ),
        (
            "ties the other way round",
            full,
            vec![(drone, Vec2::new(110.0, -50.0)), (drone, Vec2::new(110.0, 50.0))],
            Some(2),
        ),
        ("out of range: inactive", full, vec![(drone, off(0.0, 236.0))], None),
        (
            "out of the arc, though threatening",
            ninety,
            vec![(peacekeeper, off(60.0, 80.0)), (drone, off(40.0, 150.0))],
            Some(3),
        ),
        (
            "nothing in the arc: inactive",
            ninety,
            vec![(drone, off(90.0, 80.0)), (drone, off(-120.0, 80.0))],
            None,
        ),
    ];
    for (name, attrs, enemies, locks) in rows {
        assert_eq!(first_lock(attrs, &enemies), locks, "{name}");
    }
}

/// `systems/controls/behaviors.md > TurretAutonomousScanning > Target Selection Priority` 2: "the least turret
/// rotation from the turret's current facing", not the base's. A 360° turret holding position locks a Drone due
/// south and turns to it; two more come, one straight ahead of the base (nearer) and one 10° from the turret (far).
/// Once the first is gone, the turret takes the one 10° from it: from the base's facing, the other would need less.
#[test]
fn a_turret_s_scan_measures_the_rotation_from_the_turret() {
    let _def = turret_unit(turret(360.0, 12.0), GUN);
    let at = Vec2::new(500.0, 320.0);
    let south = ObjectId(2);
    let (ahead, beside) = (ObjectId(3), ObjectId(4));
    let mut t = field(16, at, &[(ObjectKind::Drone, at + Vec2::new(0.0, 100.0))]);
    t.order_at(2, P1, &[UNIT], OrderKind::Hold, false)
        .spawn_unit_at(10, ObjectKind::Drone, P2, at + Vec2::new(60.0, 0.0))
        .spawn_unit_at(10, ObjectKind::Drone, P2, toward(at, 100.0, 200.0))
        .run_until(14);
    assert_eq!(locked(&t, UNIT), Some(south), "it keeps what it can fire at");
    let facing = turret_of(&t, UNIT).unwrap().facing.to_degrees();
    assert!((facing - 90.0).abs() < 1e-3, "the turret faces south: {facing}°");
    t.debug_at(15, DebugCommand::Remove { id: south }).step();
    assert_eq!(locked(&t, UNIT), Some(beside));
    assert!(gap(&t, UNIT, ahead) < gap(&t, UNIT, beside));
}

/// `systems/controls/behaviors.md > BaseAutoTargeting > Targets inside MinRange` (design change 0182): the
/// turret's scan "skips enemies inside the weapon's MinRange and picks one it can hit; with none, it does nothing"
/// (`combat::inside_min_range`, through `combat::within_range`). With a MinRange of 1.5 grid units, a turret holding
/// position locks the enemy beyond it, which needs more rotation and is farther, never the one inside it; once that
/// one is gone, it locks nothing and fires no more.
#[test]
fn a_turret_s_scan_skips_the_enemies_inside_its_min_range() {
    let gun = AttackAttrs { min_range: 1.5, ..GUN };
    let _def = turret_unit(turret(360.0, 12.0), gun);
    let at = Vec2::new(500.0, 320.0);
    let (near, far) = (ObjectId(2), ObjectId(3));
    let enemies = [
        (ObjectKind::Drone, at + Vec2::new(80.0, 0.0)),
        (ObjectKind::Drone, toward(at, 120.0, 200.0)),
    ];
    let mut t = field(16, at, &enemies);
    assert!(inside_min_range(&gun, gap(&t, UNIT, near)));
    assert!(!inside_min_range(&gun, gap(&t, UNIT, far)) && gap(&t, UNIT, far) <= gu(gun.range));
    t.order_at(2, P1, &[UNIT], OrderKind::Hold, false).run_until(40);
    assert_eq!(locked(&t, UNIT), Some(far));
    let fired = shots(&t, UNIT);
    assert!(
        fired.len() >= 3 && fired.iter().all(|&(_, target)| target == far),
        "{fired:?}"
    );
    t.debug_at(41, DebugCommand::Remove { id: far }).run_until(80);
    assert_eq!(locked(&t, UNIT), None);
    assert_eq!(shots(&t, UNIT), fired, "nothing it can hit");
    assert_eq!(t.pos(1), at, "it holds");
}

/// The A and B of the retention tests: a test turret unit with a 360° turret, a Drone due south (A, id 2, a gap of
/// 114 su) and one 10° from it and nearer (B, id 3, a gap of about 43 su), on a field 32 tiles wide. A turret that
/// scans from A's direction would take B: one rotation step, and nearer.
fn a_and_b() -> (TestSim, ObjectId, ObjectId) {
    let at = Vec2::new(400.0, 300.0);
    let enemies = [
        (ObjectKind::Drone, at + Vec2::new(0.0, 150.0)),
        (ObjectKind::Drone, toward(at, 80.0, 80.0)),
    ];
    (field(32, at, &enemies), ObjectId(2), ObjectId(3))
}

/// Plan U1's acceptance, the retention rule (`systems/controls/commands.md > TurretCommandState`): "When the base
/// behavior does not specify a target, the turret retains its previous TurretCommandState. If the turret has no
/// assigned target, it falls back to autonomous target scanning." The turret, locked on A by an Attack
/// (`systems/controls/command_behaviors.md > AttackingObject` 3), keeps A through a Move that specifies no target,
/// though its scan would take B (as it does holding position, unlocked), and fires at A all the while A is within its
/// reach. Once A is out of range, it lets go and falls back to its scan, which takes B
/// (`systems/controls/commands.md > TurretCommandState > When a lock ends`, design change 0376: "When neither holds, it
/// drops the lock and scans for a new target in the same tick"). The unit drives on all the while.
#[test]
fn a_turret_keeps_its_target_through_an_order_without_one_and_then_scans() {
    let _def = turret_unit(turret(360.0, 12.0), GUN);
    let (mut control, _, b) = a_and_b();
    control.order_at(2, P1, &[UNIT], OrderKind::Hold, false).step();
    assert_eq!(locked(&control, UNIT), Some(b), "the scan's choice");
    let (mut t, a, b) = a_and_b();
    let order = OrderKind::Move(Target::Point(Vec2::new(1400.0, 300.0)));
    t.order_at(2, P1, &[UNIT], OrderKind::Attack(a), false)
        .order_at(12, P1, &[UNIT], order, false)
        .run_until(11);
    assert_eq!(locked(&t, UNIT), Some(a));
    assert!(!shots(&t, UNIT).is_empty(), "it fires at A");
    let reach = gu(GUN.range);
    let (mut at_a, mut released, mut started) = (0, None, false);
    while released.is_none() || shots(&t, UNIT).last().is_some_and(|&(_, target)| target != b) {
        assert!(t.sim().tick() < 100, "never at B: {:?}", shots(&t, UNIT));
        let (a_in_reach, x) = (gap(&t, UNIT, a) <= reach, t.pos(1).x);
        t.step();
        let tick = t.sim().tick();
        assert_eq!(current(&t, UNIT), Some(order), "tick {tick}");
        assert_eq!(engaged(&t, UNIT), None, "tick {tick}: the Move engages nothing");
        started |= t.pos(1).x > x;
        assert!(!started || t.pos(1).x > x, "tick {tick}: it drives on");
        if a_in_reach {
            assert_eq!(locked(&t, UNIT), Some(a), "tick {tick}: it keeps A");
        } else {
            released.get_or_insert(tick);
            assert_eq!(locked(&t, UNIT), Some(b), "tick {tick}: it scans, and takes B");
        }
        if shots(&t, UNIT).last() == Some(&(tick, a)) {
            at_a += 1;
        }
    }
    assert!(at_a >= 3, "it fired at A on the move {at_a} times");
    let released = released.unwrap();
    assert!(
        shots(&t, UNIT)
            .iter()
            .all(|&(tick, target)| (target == a) == (tick < released)),
        "{:?}",
        shots(&t, UNIT)
    );
}

/// Plan U1's acceptance: "A Move's scan never stops the base." The same Move east, with Drones beside the road that
/// the turret fires at and without them: the unit is where it would have been, every tick, and its base engages
/// nothing (`systems/controls/behaviors.md > BaseAutoTargeting > Not active during`: "Move — pure movement, no base
/// engagement").
#[test]
fn a_move_s_scan_never_stops_the_base() {
    let _def = turret_unit(turret(360.0, 12.0), GUN);
    let drive = |enemies: &[Enemy]| {
        let mut t = field(32, Vec2::new(200.0, 400.0), enemies);
        let order = OrderKind::Move(Target::Point(Vec2::new(1600.0, 400.0)));
        t.order_at(2, P1, &[UNIT], order, false).run_until(2);
        let mut path = Vec::new();
        while current(&t, UNIT) == Some(order) {
            assert!(t.sim().tick() < 400, "it never arrived");
            assert_eq!(engaged(&t, UNIT), None);
            path.push(t.pos(1));
            t.step();
        }
        (path, shots(&t, UNIT))
    };
    let (alone, none) = drive(&[]);
    let beside = [
        (ObjectKind::Drone, Vec2::new(700.0, 260.0)),
        (ObjectKind::Drone, Vec2::new(1100.0, 540.0)),
    ];
    let (passing, fired) = drive(&beside);
    assert!(none.is_empty());
    assert!(fired.len() >= 20, "{fired:?}");
    assert_eq!(passing, alone);
}

/// `systems/controls/command_behaviors.md > AttackingObject` 3: "set TurretCommandState.LockedTarget = TargetObject
/// (turret units)". A turret holding position has locked the enemy straight ahead of it, the nearest, and fires at
/// it; ordered to attack another, due south, it locks that one at once (in range and arc) and fires at it alone from
/// then, after the turn.
#[test]
fn an_attack_locks_the_turret_on_its_target() {
    let _def = turret_unit(turret(360.0, 12.0), GUN);
    let at = Vec2::new(400.0, 300.0);
    let (ahead, south) = (ObjectId(2), ObjectId(3));
    let enemies = [
        (ObjectKind::Drone, at + Vec2::new(60.0, 0.0)),
        (ObjectKind::Drone, at + Vec2::new(0.0, 150.0)),
    ];
    let mut t = field(16, at, &enemies);
    t.order_at(2, P1, &[UNIT], OrderKind::Hold, false).run_until(11);
    assert_eq!(locked(&t, UNIT), Some(ahead));
    let before = shots(&t, UNIT);
    assert!(!before.is_empty() && before.iter().all(|&(_, target)| target == ahead));
    t.order_at(12, P1, &[UNIT], OrderKind::Attack(south), false).step();
    assert_eq!((engaged(&t, UNIT), locked(&t, UNIT)), (Some(south), Some(south)));
    t.run_until(40);
    let after = &shots(&t, UNIT)[before.len()..];
    assert!(after.len() >= 3, "{after:?}");
    assert!(
        after.iter().all(|&(tick, target)| target == south && tick >= 19),
        "{after:?}"
    );
    assert_eq!(locked(&t, UNIT), Some(south));
}

/// `systems/controls/command_behaviors.md > HoldingPosition` 2: "For turret units: TurretCommandState is unchanged —
/// turret continues autonomous scanning and engaging targets within range and arc." Holding position, the unit's
/// turret fires at an enemy 60° off, in range, and leaves one beyond its range alone; the base engages nothing and
/// neither moves nor turns.
#[test]
fn a_turret_holding_position_scans_and_fires_while_its_base_stands() {
    let _def = turret_unit(turret(360.0, 12.0), GUN);
    let at = Vec2::new(400.0, 300.0);
    let (near, far) = (ObjectId(2), ObjectId(3));
    let enemies = [
        (ObjectKind::Drone, toward(at, 60.0, 120.0)),
        (ObjectKind::Drone, at + Vec2::new(-280.0, 0.0)),
    ];
    let mut t = field(16, at, &enemies);
    t.order_at(2, P1, &[UNIT], OrderKind::Hold, false);
    for tick in 2..=40 {
        t.step();
        assert_eq!(current(&t, UNIT), Some(OrderKind::Hold), "tick {tick}");
        assert_eq!(engaged(&t, UNIT), None, "tick {tick}");
        assert_eq!((t.pos(1), facing_degrees(&t, UNIT)), (at, 0.0), "tick {tick}");
    }
    let fired = shots(&t, UNIT);
    assert!(
        fired.len() >= 5 && fired.iter().all(|&(_, target)| target == near),
        "{fired:?}"
    );
    assert!(gap(&t, UNIT, far) > gu(GUN.range));
}

/// `systems/controls/command_behaviors.md > StoppingBehavior` 2: "Clear TurretCommandState.LockedTarget (turret falls
/// back to autonomous scanning)"; `systems/controls/commands.md > TurretCommandState > When a lock ends` (design change
/// 0376): "**Stop clears the lock**". The turret, locked by an Attack on one of its owner's own Drones (an Attack's
/// target may be anything destructible), keeps it through a Move a few steps on (the retention rule) and fires at it
/// all the while; a Stop lets it go, and its scan, and its idle base's, take enemies only, of which there are none: it
/// fires no more.
#[test]
fn a_stop_clears_the_turret_s_target() {
    let _def = turret_unit(turret(360.0, 12.0), GUN);
    let at = Vec2::new(400.0, 300.0);
    let own = ObjectId(2);
    let then = |order: OrderKind| {
        let mut t = ScenarioBuilder::new(&open(16, 10)).start();
        t.spawn_unit_at(1, TURRET_UNIT, P1, at)
            .spawn_unit_at(1, ObjectKind::Drone, P1, at + Vec2::new(0.0, 100.0))
            .order_at(2, P1, &[UNIT], OrderKind::Attack(own), false)
            .order_at(12, P1, &[UNIT], order, false)
            .run_until(11);
        assert_eq!(locked(&t, UNIT), Some(own));
        let before = shots(&t, UNIT).len();
        assert!(before > 0, "it fires at its own Drone");
        t.run_until(40);
        (locked(&t, UNIT), shots(&t, UNIT).len() - before)
    };
    let (kept, fired) = then(OrderKind::Move(Target::Point(at + Vec2::new(40.0, 0.0))));
    assert_eq!(kept, Some(own));
    assert!(fired >= 5, "{fired}");
    assert_eq!(then(OrderKind::Stop), (None, 0));
}

/// Design change 0376's Verification (`systems/controls/commands.md > TurretCommandState > When a lock ends`, answering
/// Q0137), on the test turret unit (no Raptor yet): "Force-Attack a friendly unit [...], then Move nearby: the turret
/// keeps firing at it. Press Stop: the turret drops it and scans for enemies." Locked by an Attack on one of its
/// owner's own Drones, the turret keeps it through a Move east ("**Commands that set no target leave the lock
/// alone**") and fires at it alone on the move, though an enemy Drone is within its reach all the while; a Stop
/// pressed on the way ("**Stop clears the lock**") lets it go, and in that very tick it is locked on the enemy, which
/// the scan of its base, idle by then, took (`systems/controls/behaviors.md > BaseAutoTargeting`: "For turret units,
/// the base sets TurretCommandState.LockedTarget"). It fires at the enemy alone from then on, the unit standing. (That
/// base's scan would lock the enemy even over a lock the Stop kept: [`a_stop_clears_the_turret_s_target`], with no
/// enemy about, shows the drop.)
#[test]
fn a_stop_drops_the_lock_a_move_kept_and_the_turret_scans_for_enemies() {
    const MOVE: Tick = 12;
    const STOP: Tick = 30;
    let _def = turret_unit(turret(360.0, 12.0), GUN);
    let at = Vec2::new(400.0, 300.0);
    let (own, enemy) = (ObjectId(2), ObjectId(3));
    let order = OrderKind::Move(Target::Point(at + Vec2::new(300.0, 0.0)));
    let mut t = ScenarioBuilder::new(&open(16, 10)).start();
    t.spawn_unit_at(1, TURRET_UNIT, P1, at)
        .spawn_unit_at(1, ObjectKind::Drone, P1, at + Vec2::new(0.0, 100.0))
        .spawn_unit_at(1, ObjectKind::Drone, P2, at + Vec2::new(100.0, -60.0))
        .order_at(2, P1, &[UNIT], OrderKind::Attack(own), false)
        .order_at(MOVE, P1, &[UNIT], order, false)
        .order_at(STOP, P1, &[UNIT], OrderKind::Stop, false)
        .run_until(MOVE - 1);
    assert_eq!(locked(&t, UNIT), Some(own));
    assert!(!shots(&t, UNIT).is_empty(), "it fires at its own Drone");
    let reach = gu(GUN.range);
    while t.sim().tick() < STOP - 1 {
        t.step();
        let tick = t.sim().tick();
        assert_eq!(current(&t, UNIT), Some(order), "tick {tick}: on the move");
        assert_eq!(locked(&t, UNIT), Some(own), "tick {tick}: it keeps its own Drone");
        assert!(gap(&t, UNIT, enemy) <= reach, "tick {tick}: the enemy is within reach");
    }
    assert!(t.pos(1).x > at.x + 50.0, "it drove on: {}", t.pos(1));
    let moving: Vec<_> = (shots(&t, UNIT).into_iter())
        .filter(|&(tick, _)| tick >= MOVE)
        .collect();
    assert!(
        moving.len() >= 3 && moving.iter().all(|&(_, target)| target == own),
        "{moving:?}"
    );
    t.step();
    assert_eq!(current(&t, UNIT), None, "stopped");
    assert_eq!(locked(&t, UNIT), Some(enemy), "it dropped its own Drone, and scanned");
    assert_eq!(engaged(&t, UNIT), Some(enemy));
    let stood = t.pos(1);
    t.run_until(STOP + 40);
    let after: Vec<_> = (shots(&t, UNIT).into_iter())
        .filter(|&(tick, _)| tick >= STOP)
        .collect();
    assert!(
        after.len() >= 3 && after.iter().all(|&(_, target)| target == enemy),
        "{after:?}"
    );
    assert_eq!(t.pos(1), stood, "it stands");
}
