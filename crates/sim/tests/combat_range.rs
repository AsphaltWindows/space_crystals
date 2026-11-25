//! What an attack reaches (`systems/combat.md > AttackAttributes`, `systems/combat.md > ValidTarget`): Range measured
//! edge to edge, the gap between the attacker's shape and the target's (`systems/combat.md > Range Measurement`, design
//! change 0031), MinRange, TargetDomain, and the ElevationModifier (`core/vision.md > ElevationModifier`) at the exact
//! edge of a level.
//!
//! Both units stand on Hold Position, so neither moves: the attacker engages what its weapon reaches from where it
//! stands (`systems/controls/behaviors.md > HoldingPosition`), which is the range check alone.

use sc_sim::combat;
use sc_sim::defs::{AttackType, Domain, FullyConnectedSubtype, TargetDomain};
use sc_sim::elevation::Elevation;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{ObjectId, ObjectKind, OrderKind, PlayerId, Shape, SimEvent, Tick, Vec2, gu};

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
/// both are ordered to hold their position in tick 2, and the game runs to tick 10. The ticks of the attacker's
/// attacks.
fn attacks(map: &str, attacker: (ObjectKind, Vec2), target: (ObjectKind, Vec2)) -> (TestSim, Vec<Tick>) {
    let mut t = ScenarioBuilder::new(map).start();
    t.spawn_unit_at(1, attacker.0, P1, attacker.1)
        .spawn_unit_at(1, target.0, P2, target.1)
        .order_at(2, P1, &[ATTACKER], OrderKind::Hold, false)
        .order_at(2, P2, &[TARGET], OrderKind::Hold, false)
        .run_until(10);
    assert!(t.sim().object(TARGET).is_some(), "both spawned");
    let ticks = t
        .events()
        .iter()
        .filter(|(_, e)| matches!(e, SimEvent::Attacked { attacker, .. } if *attacker == ATTACKER))
        .map(|&(tick, _)| tick)
        .collect();
    (t, ticks)
}

fn can_attack_now(t: &TestSim) -> bool {
    let sim = t.sim();
    combat::can_attack_now(
        sim.map(),
        sim.vision(),
        sim.object(ATTACKER).unwrap(),
        sim.object(TARGET).unwrap(),
    )
}

/// Design change 0031: Range is the gap between the two shapes. A Peacekeeper (Range 4 grid units, 256 su; 24 su
/// wide) reaches a Guard (36 su wide) when the gap between their edges is exactly 256 su, and not 1 su further.
#[test]
fn range_is_the_gap_between_the_edges() {
    let flat = RAMP.split("\n\n").next().unwrap();
    let pk = (ObjectKind::Peacekeeper, Vec2::new(100.0, 96.0));
    let guard_at = |gap: f32| (ObjectKind::Guard, Vec2::new(100.0 + 12.0 + gap + 18.0, 96.0));
    let (t, ticks) = attacks(flat, pk, guard_at(256.0));
    assert!(can_attack_now(&t));
    assert_eq!(ticks, [4]);
    let (t, ticks) = attacks(flat, pk, guard_at(257.0));
    assert!(!can_attack_now(&t));
    assert_eq!(ticks, [] as [Tick; 0]);
    // Diagonally, between the two nearest corners: 3-4-5.
    let shape = |(kind, at): (ObjectKind, Vec2)| Shape::silhouette(kind, at, 0.0);
    let diagonal = (
        ObjectKind::Guard,
        pk.1 + Vec2::new(12.0 + 153.6 + 18.0, 12.0 + 204.8 + 18.0),
    );
    assert!((shape(pk).gap(&shape(diagonal)) - 256.0).abs() < 1e-3);
}

/// Design change 0031's Verification, as far as today's units go: an attacker's own body doesn't eat into its
/// range. The wide Guard (Range 3, 36 su) and the narrow Peacekeeper (Range 4, 24 su) each start firing when the
/// gap between the edges reaches their Range, whatever their own width. (The Guard aims for 4 ticks, design change
/// 0186: its first hit lands in tick 6.)
#[test]
fn an_attackers_own_size_doesnt_change_its_range() {
    let flat = RAMP.split("\n\n").next().unwrap();
    // The Guard attacks; the Peacekeeper on Hold would fire back, which changes nothing here.
    let guard = (ObjectKind::Guard, Vec2::new(100.0, 96.0));
    let pk_at = |gap: f32| (ObjectKind::Peacekeeper, Vec2::new(100.0 + 18.0 + gap + 12.0, 96.0));
    let (_, ticks) = attacks(flat, guard, pk_at(192.0));
    assert_eq!(ticks, [6], "Range 3 from its edge");
    let (_, ticks) = attacks(flat, guard, pk_at(193.0));
    assert_eq!(ticks, [] as [Tick; 0]);
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
/// Air units, Universal attacks anything. Every M1 unit is a ground unit; the Peacekeeper's attack is Ground, the
/// Guard's laser rifle Universal (design change 0186).
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
    for (kind, domain) in [
        (ObjectKind::Peacekeeper, TargetDomain::Ground),
        (ObjectKind::Guard, TargetDomain::Universal),
    ] {
        let unit = kind.def().unit.as_ref().unwrap();
        assert_eq!(unit.attack.as_ref().unwrap().target_domain, domain, "{kind:?}");
        assert_eq!(unit.base_def().domain, Ground);
    }
}

/// `core/vision.md > ElevationModifier`: +1 grid unit from higher ground, −1 from lower ground, by elevation level,
/// the height rounded down. A Peacekeeper one su below the top of the ramp (height 63/64, level 0) doesn't reach a
/// Guard on level 0 across a gap of 275 su (4.3 grid units); at the top (height exactly 1, level 1) it does, and a
/// step further too.
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

/// The other way round: a Guard (Range 3) on level 0 reaches a Peacekeeper across a gap of 141 su (2.2 grid units)
/// on the ramp while it is on level 0, and not once it stands on level 1 (2 grid units). (Its hit lands after its Aim
/// of 4 ticks.)
#[test]
fn lower_ground_takes_one_grid_unit_off_from_the_exact_level() {
    let guard = (ObjectKind::Guard, Vec2::new(512.0 - 12.0 - 160.0, 96.0));
    let at = |x: f32| (ObjectKind::Peacekeeper, Vec2::new(x, 96.0));
    let (_, ticks) = attacks(RAMP, guard, at(511.0));
    assert_eq!(ticks, [6], "equal levels: 3 grid units");
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
