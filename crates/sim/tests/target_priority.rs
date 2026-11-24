//! Design change 0179 (`systems/controls/behaviors.md > TurretAutonomousScanning > Target Selection Priority`, which
//! BaseAutoTargeting uses too): the rotation a target needs counts in steps of 45°; within the lowest step the closest
//! target wins, then the fixed order of ids. The steps themselves: `orders::tests::priority`.

use sc_sim::math;
use sc_sim::testing::ScenarioBuilder;
use sc_sim::{ObjectId, ObjectKind, OrderKind, PlayerId, Vec2, gu};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const PK: ObjectKind = ObjectKind::Peacekeeper; // 24 × 24 su, Range 4

/// 16 × 10 open tiles.
fn open() -> String {
    "................\n".repeat(10)
}

/// The target a Peacekeeper of player 1 (id 1), facing east and holding its position, picks among Peacekeepers of
/// player 2 (ids 2, 3, …) placed at `enemies` from it, which hold theirs.
fn pick(enemies: &[Vec2]) -> Option<ObjectId> {
    let at = Vec2::new(300.0, 320.0);
    let mut t = ScenarioBuilder::new(&open()).start();
    t.spawn_unit_at(1, PK, P1, at);
    for &enemy in enemies {
        t.spawn_unit_at(1, PK, P2, at + enemy);
    }
    let theirs: Vec<ObjectId> = (2..=enemies.len() as u32 + 1).map(ObjectId).collect();
    t.order_at(2, P2, &theirs, OrderKind::Hold, false)
        .order_at(2, P1, &[ObjectId(1)], OrderKind::Hold, false)
        .run_until(1);
    // It picks in tick 2, facing east still (and turns to its target as it aims).
    assert_eq!(t.sim().object(ObjectId(1)).unwrap().facing, 0.0, "east");
    t.run_until(2);
    t.sim().object(ObjectId(1)).unwrap().attack.as_ref().unwrap().target
}

/// `d` su from the unit's centre, `degrees` off its facing.
fn off(degrees: f32, d: f32) -> Vec2 {
    math::facing_vector(math::degrees_to_radians(degrees)) * d
}

/// Dead ahead, the gap between the two Peacekeepers exactly their Range.
fn at_max_range() -> Vec2 {
    Vec2::new(gu(4.0) + 24.0, 0.0)
}

/// Design change 0179's Verification: "Enemy A at max range dead ahead, enemy B adjacent at 5°: the unit shoots B
/// first. Enemy C adjacent at 60° vs A ahead: A first."
#[test]
fn the_closest_target_within_the_least_step_of_rotation_comes_first() {
    let (a, b, c) = (at_max_range(), off(5.0, 40.0), off(60.0, 40.0));
    assert_eq!(
        pick(&[a, b]),
        Some(ObjectId(3)),
        "B: both in the first step, B the closer"
    );
    assert_eq!(pick(&[b, a]), Some(ObjectId(2)), "B, whichever id it has");
    assert_eq!(pick(&[a, c]), Some(ObjectId(2)), "A: C is in the second step");
    assert_eq!(pick(&[c, a]), Some(ObjectId(3)), "A, whichever id it has");
}

/// The first step reaches up to 45° (`orders::world::rotation_step`, a placeholder reading of the shared bounds): an
/// adjacent enemy exactly 45° off comes before one at max range dead ahead; one a little more than 45° off doesn't.
#[test]
fn an_enemy_exactly_45_degrees_off_is_in_the_first_step() {
    let diagonal = Vec2::splat(30.0);
    assert_eq!(math::vector_angle(diagonal), math::degrees_to_radians(45.0));
    assert_eq!(pick(&[at_max_range(), diagonal]), Some(ObjectId(3)));
    assert_eq!(pick(&[at_max_range(), off(46.0, 40.0)]), Some(ObjectId(2)));
}

/// "Then a fixed deterministic order": two enemies alike, as far off and as many degrees to either side, go by
/// their ids; the lower first.
#[test]
fn equal_targets_go_by_their_ids() {
    let (left, right) = (off(-20.0, 60.0), off(20.0, 60.0));
    assert_eq!(pick(&[left, right]), Some(ObjectId(2)));
    assert_eq!(pick(&[right, left]), Some(ObjectId(2)));
}
