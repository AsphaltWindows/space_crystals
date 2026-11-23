//! What an attack reaches (`systems/combat.md > AttackAttributes`, `> ValidTarget`): Range measured from the
//! attacker's centre to the nearest point of the target (question Q0009, default A), MinRange, TargetDomain, and the
//! ElevationModifier (`core/vision.md > ElevationModifier`) at the exact edge of a level.

use sc_sim::combat::{self, RangeMeasure};
use sc_sim::defs::{AttackType, Domain, FullyConnectedSubtype, TargetDomain};
use sc_sim::elevation::Elevation;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{ObjectId, ObjectKind, OrderKind, PlayerId, SimEvent, Tick, Vec2, gu};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const ATTACKER: ObjectId = ObjectId(1);
const TARGET: ObjectId = ObjectId(2);

/// 12 × 3 tiles. Level 0 up to x = 448 su; tile 7 ramps from 0 to 1 eastwards, so the height reaches level 1
/// exactly at x = 512, one level per 64 su; level 1 from there on.
const RAMP: &str = "
    ............
    ............
    ............

    0000000011111
    0000000011111
    0000000011111
    0000000011111
";

/// `attacker` (player 1, id 1) at `attacker_at` and `target` (player 2, id 2) at `target_at`, spawned in tick 1;
/// the attacker is ordered to attack in tick 2, and the game runs to tick 10. The ticks of its attacks.
fn attacks(map: &str, attacker: (ObjectKind, Vec2), target: (ObjectKind, Vec2)) -> (TestSim, Vec<Tick>) {
    let mut t = ScenarioBuilder::new(map).start();
    t.spawn_unit_at(1, attacker.0, P1, attacker.1)
        .spawn_unit_at(1, target.0, P2, target.1)
        .order_at(2, P1, &[ATTACKER], OrderKind::Attack(TARGET), false)
        .run_until(10);
    assert!(t.sim().object(TARGET).is_some(), "both spawned");
    let ticks = t
        .events()
        .iter()
        .filter(|(_, e)| matches!(e, SimEvent::Attacked { .. }))
        .map(|&(tick, _)| tick)
        .collect();
    (t, ticks)
}

fn can_attack_now(t: &TestSim) -> bool {
    let sim = t.sim();
    combat::can_attack_now(sim.map(), sim.object(ATTACKER).unwrap(), sim.object(TARGET).unwrap())
}

/// Question Q0009, default A: from the attacker's centre to the nearest point of the target's silhouette. A
/// Peacekeeper (Range 4 grid units, 256 su) reaches a Guard (36 su wide) whose near edge is exactly 256 su away,
/// and not one 1 su further.
#[test]
fn range_reaches_the_nearest_point_of_the_target() {
    assert_eq!(combat::RANGE_MEASURE, RangeMeasure::CentreToNearestPoint);
    let flat = RAMP.split("\n\n").next().unwrap();
    let pk = (ObjectKind::Peacekeeper, Vec2::new(100.0, 96.0));
    let (t, ticks) = attacks(flat, pk, (ObjectKind::Guard, Vec2::new(100.0 + 256.0 + 18.0, 96.0)));
    assert!(can_attack_now(&t));
    assert_eq!(ticks, [4]);
    let (t, ticks) = attacks(flat, pk, (ObjectKind::Guard, Vec2::new(100.0 + 257.0 + 18.0, 96.0)));
    assert!(!can_attack_now(&t));
    assert_eq!(ticks, [] as [Tick; 0]);
    // Diagonally, past the corner of the silhouette: 3-4-5.
    let corner = Vec2::new(100.0 + 153.6 + 18.0, 96.0 + 204.8 + 18.0);
    assert!(
        (combat::distance_to_box(pk.1, corner - 18.0, corner + 18.0) - 256.0).abs() < 1e-3,
        "the nearest point is the corner"
    );
}

/// `systems/controls/behaviors.md > AttackingObject` 2: the target must be beyond MinRange. No M1 unit has one, so
/// this checks the rule on a Peacekeeper's attack given a MinRange of 2 grid units. The ElevationModifier changes
/// the Range only.
#[test]
fn min_range() {
    let mut attrs = ObjectKind::Peacekeeper.def().unit.as_ref().unwrap().attack.unwrap();
    attrs.min_range = 2.0;
    let level = Elevation::Level(0);
    let high = Elevation::Level(1);
    assert!(!combat::within_range(&attrs, gu(2.0) - 0.5, level, level));
    assert!(combat::within_range(&attrs, gu(2.0), level, level));
    assert!(combat::within_range(&attrs, gu(4.0), level, level));
    assert!(!combat::within_range(&attrs, gu(4.0) + 0.5, level, level));
    assert!(
        !combat::within_range(&attrs, gu(2.0) - 0.5, high, level),
        "MinRange keeps its value"
    );
    assert!(combat::within_range(&attrs, gu(5.0), high, level));
}

/// `systems/combat.md > ValidTarget` 3: Ground attacks target Ground (and surfaced Underground) units, Air attacks
/// Air units, Universal attacks anything. Every M1 unit is a ground unit with a Ground attack.
#[test]
fn target_domain() {
    use Domain::{Air, Ground, Underground};
    let table = [
        (TargetDomain::Ground, [true, false, false]),
        (TargetDomain::Air, [false, true, false]),
        (TargetDomain::Universal, [true, true, true]),
    ];
    for (attack, allowed) in table {
        let got = [Ground, Air, Underground].map(|d| combat::domain_allows(attack, d));
        assert_eq!(got, allowed, "{attack:?}");
    }
    for kind in [ObjectKind::Peacekeeper, ObjectKind::Guard] {
        let unit = kind.def().unit.as_ref().unwrap();
        assert_eq!(unit.attack.as_ref().unwrap().target_domain, TargetDomain::Ground);
        assert_eq!(unit.base_def().domain, Ground);
    }
}

/// `core/vision.md > ElevationModifier`: +1 grid unit from higher ground, −1 from lower ground, by elevation level,
/// the height rounded down. A Peacekeeper one su below the top of the ramp (height 63/64, level 0) doesn't reach a
/// Guard on level 0 whose edge is 288 su (4.5 grid units) away; at the top (height exactly 1, level 1) it does, and
/// a step further too.
#[test]
fn higher_ground_adds_one_grid_unit_from_the_exact_level() {
    let guard = (ObjectKind::Guard, Vec2::new(512.0 - 288.0 - 18.0, 96.0));
    let at = |x: f32| (ObjectKind::Peacekeeper, Vec2::new(x, 96.0));
    let (t, ticks) = attacks(RAMP, at(511.0), guard);
    assert_eq!(t.sim().map().elevation_level(Vec2::new(511.0, 96.0)), 0);
    assert_eq!(ticks, [] as [Tick; 0], "level 0: 4 grid units");
    let (t, ticks) = attacks(RAMP, at(512.0), guard);
    assert_eq!(t.sim().map().elevation_level(Vec2::new(512.0, 96.0)), 1);
    assert!(can_attack_now(&t));
    assert_eq!(ticks, [4], "level 1: 5 grid units");
    let (_, ticks) = attacks(RAMP, at(513.0), guard);
    assert_eq!(ticks, [4]);
}

/// The other way round: a Guard (Range 3) on level 0 reaches a Peacekeeper 160 su (2.5 grid units) away on the ramp
/// while it is on level 0, and not once it stands on level 1 (2 grid units).
#[test]
fn lower_ground_takes_one_grid_unit_off_from_the_exact_level() {
    let guard = (ObjectKind::Guard, Vec2::new(512.0 - 12.0 - 160.0, 96.0));
    let at = |x: f32| (ObjectKind::Peacekeeper, Vec2::new(x, 96.0));
    let (_, ticks) = attacks(RAMP, guard, at(511.0));
    assert_eq!(ticks, [4], "equal levels: 3 grid units");
    let (t, ticks) = attacks(RAMP, guard, at(512.0));
    assert!(!can_attack_now(&t));
    assert_eq!(ticks, [] as [Tick; 0], "uphill: 2 grid units");
}

/// `systems/combat.md > FullyConnected > Melee`: melee attacks don't take the modifier (no M1 unit has one).
#[test]
fn melee_ignores_elevation() {
    let mut attrs = ObjectKind::Guard.def().unit.as_ref().unwrap().attack.unwrap();
    let (high, low) = (Elevation::Level(2), Elevation::Level(0));
    assert_eq!(combat::max_range(&attrs, high, low), gu(4.0));
    attrs.attack_type = AttackType::FullyConnected(FullyConnectedSubtype::Melee);
    assert_eq!(combat::max_range(&attrs, high, low), gu(3.0));
    assert_eq!(combat::max_range(&attrs, low, high), gu(3.0));
    // Air units are exempt, as attackers and as targets.
    attrs.attack_type = AttackType::FullyConnected(FullyConnectedSubtype::Ranged);
    assert_eq!(combat::max_range(&attrs, Elevation::Exempt, low), gu(3.0));
    assert_eq!(combat::max_range(&attrs, high, Elevation::Exempt), gu(3.0));
}
