//! Design change 0179's steps of rotation (`orders::world::scan::rotation_step`); the picks themselves are tested through
//! the game in `tests/target_priority.rs`.

use glam::Vec2;

use crate::math;
use crate::orders::world::scan::rotation_step;
const ROTATION_STEP: f32 = crate::tuning::orders::BUILTIN.rotation_step;

/// "Required rotation (0–180°, either direction) is bucketed into 45° steps": a step reaches up to its upper bound
/// (the placeholder reading at the shared bounds), and a target exactly 45° off, by the game's own angle, is in the
/// first step.
#[test]
fn rotations_fall_into_steps_of_45_degrees() {
    let degrees = |d: f32| math::degrees_to_radians(d);
    assert_eq!(rotation_step(0.0), 0);
    assert_eq!(rotation_step(degrees(5.0)), 0);
    assert_eq!(rotation_step(ROTATION_STEP), 0, "45°: the first step's upper bound");
    assert_eq!(rotation_step(ROTATION_STEP.next_up()), 1);
    assert_eq!(rotation_step(degrees(60.0)), 1);
    assert_eq!(rotation_step(2.0 * ROTATION_STEP), 1);
    assert_eq!(rotation_step(degrees(100.0)), 2);
    assert_eq!(rotation_step(degrees(170.0)), 3);
    assert_eq!(rotation_step(math::PI), 3);
    // The angle the game measures to a target diagonally off.
    assert_eq!(math::vector_angle(Vec2::new(100.0, 100.0)), ROTATION_STEP);
    assert_eq!(rotation_step(math::vector_angle(Vec2::new(100.0, 100.0))), 0);
}
