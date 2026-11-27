//! Range to a turned target (`systems/combat.md > AttackAttributes > Range Measurement`, design change 0031: edge to
//! edge; design change 0033: "range measurement [...] use[s] the rotated shape"). A Seeker facing north is 40 su wide
//! and 56 su long; the box it has facing east would be 56 wide and 40 long. A Militant (Range 4 grid units, 256 su;
//! 24 su square) on Hold fires at it from 4 grid units off either side of the turned shape, and not from 6 su further,
//! where off its long side it would still reach the box facing east.

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{MoveStatus, ObjectId, ObjectKind, OrderKind, Shape, SimEvent, Target, Tick, Vec2, combat, gu};

use super::{P1, P2, SEEKER};

const MILITANT: ObjectId = ObjectId(2);
/// Where the Seeker holds, facing north.
const AT: Vec2 = Vec2::new(512.0, 704.0);

/// 24 × 24 open tiles.
fn field() -> String {
    format!("{}\n", ".".repeat(24)).repeat(24)
}

/// The Seeker (#1) comes in facing east, 64 su west and 364 su south of [`AT`], and is sent there. (M5-V2) It can't
/// turn in place (`systems/units.md > FixedTurnRadiusMovement`): it turns left on its 64 su circle as it sets off and
/// drives north onto the point, where it stands facing north to within 0.01 rad.
fn a_seeker_facing_north() -> TestSim {
    let mut t = ScenarioBuilder::new(&field()).start();
    t.spawn_unit_at(1, ObjectKind::Seeker, P1, AT + Vec2::new(-64.0, 364.0))
        .order_at(2, P1, &[SEEKER], OrderKind::Move(Target::Point(AT)), false)
        .run_until(3);
    while t.status(SEEKER.0) == MoveStatus::Moving {
        t.step();
        assert!(t.sim().tick() < 200, "{:?}", t.mover(SEEKER.0));
    }
    let seeker = t.sim().object(SEEKER).unwrap();
    assert_eq!(seeker.pos, AT);
    assert!(
        (seeker.facing + core::f32::consts::FRAC_PI_2).abs() < 0.01,
        "facing north: {}",
        seeker.facing
    );
    t
}

/// [`a_seeker_facing_north`], then a Militant of player 2 (#2) comes in where `place` puts it, given the Seeker's
/// shape, and both hold for 18 ticks. The ticks of the Militant's attacks.
fn attacks_on_a_seeker_facing_north(place: impl Fn(&Shape) -> Vec2) -> (TestSim, Vec<Tick>) {
    let mut t = a_seeker_facing_north();
    let militant = place(&Shape::of(t.sim().object(SEEKER).unwrap()));
    let tick = t.sim().tick() + 1;
    t.order_at(tick, P1, &[SEEKER], OrderKind::Hold, false)
        .spawn_unit_at(tick, ObjectKind::Militant, P2, militant)
        .order_at(tick + 1, P2, &[MILITANT], OrderKind::Hold, false)
        .run_until(tick + 19);
    let ticks = t
        .events()
        .iter()
        .filter(|(_, e)| matches!(e, SimEvent::Attacked { attacker, .. } if *attacker == MILITANT))
        .map(|&(tick, _)| tick)
        .collect();
    (t, ticks)
}

/// Where a Militant stands `gap` off `seeker`, on the line from its centre along `away`: found by halving, the gap
/// between their shapes growing as the Militant goes out.
fn off(seeker: &Shape, away: Vec2, gap: f32) -> Vec2 {
    let at = |d: f32| AT + away * d;
    let (mut near, mut far) = (0.0, 2.0 * gu(4.0));
    for _ in 0..60 {
        let mid = (near + far) * 0.5;
        if Shape::silhouette(ObjectKind::Militant, at(mid), 0.0).gap(seeker) < gap {
            near = mid;
        } else {
            far = mid;
        }
    }
    at(far)
}

/// The gap between the Militant's shape and the Seeker's, and whether the Militant may attack it now.
fn measured(t: &TestSim) -> (f32, bool) {
    let sim = t.sim();
    let (militant, seeker) = (sim.object(MILITANT).unwrap(), sim.object(SEEKER).unwrap());
    let gap = Shape::of(militant).gap(&Shape::of(seeker));
    (gap, combat::can_attack_now(sim.map(), sim.vision(), militant, seeker))
}

/// Off its long side, east of it: the side runs 20 su east of its centre. 4 grid units off it the Militant fires; 6 su
/// further it doesn't, though the box the Seeker has facing east, 8 su wider on that side, would be in range.
#[test]
fn a_militant_fires_at_a_turned_seeker_from_4_grid_units_off_its_long_side() {
    let east = Vec2::new(1.0, 0.0);
    let (t, ticks) = attacks_on_a_seeker_facing_north(|seeker| off(seeker, east, gu(4.0) - 0.5));
    let (gap, may) = measured(&t);
    assert!((gap - (gu(4.0) - 0.5)).abs() < 1e-3, "{gap}");
    assert!(may && !ticks.is_empty(), "{ticks:?}");
    let (t, ticks) = attacks_on_a_seeker_facing_north(|seeker| off(seeker, east, gu(4.0) + 6.0));
    let (gap, may) = measured(&t);
    assert!((gap - (gu(4.0) + 6.0)).abs() < 1e-3, "{gap}");
    let facing_east = Shape::rect(AT, Vec2::new(28.0, 20.0));
    let militant = Shape::of(t.sim().object(MILITANT).unwrap());
    assert!(
        facing_east.gap(&militant) < gu(4.0),
        "the unturned box would be in range"
    );
    assert!(!may && ticks.is_empty(), "{ticks:?}");
}

/// Off its short side, north of it: the side runs 28 su north of its centre. 4 grid units off it the Militant fires,
/// though from the box the Seeker has facing east, 8 su shorter on that side, it would be out of range; 6 su further
/// it doesn't.
#[test]
fn a_militant_fires_at_a_turned_seeker_from_4_grid_units_off_its_short_side() {
    let north = Vec2::new(0.0, -1.0);
    let (t, ticks) = attacks_on_a_seeker_facing_north(|seeker| off(seeker, north, gu(4.0) - 0.5));
    let (gap, may) = measured(&t);
    assert!((gap - (gu(4.0) - 0.5)).abs() < 1e-3, "{gap}");
    let facing_east = Shape::rect(AT, Vec2::new(28.0, 20.0));
    let militant = Shape::of(t.sim().object(MILITANT).unwrap());
    assert!(
        facing_east.gap(&militant) > gu(4.0),
        "the unturned box would be out of range"
    );
    assert!(may && !ticks.is_empty(), "{ticks:?}");
    let (t, ticks) = attacks_on_a_seeker_facing_north(|seeker| off(seeker, north, gu(4.0) + 6.0));
    let (gap, may) = measured(&t);
    assert!((gap - (gu(4.0) + 6.0)).abs() < 1e-3, "{gap}");
    assert!(!may && ticks.is_empty(), "{ticks:?}");
}
