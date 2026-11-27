//! (M3-G, M5-V2) The movement models with momentum (`movement::model`): DragMovement and FixedTurnRadiusMovement carry
//! a unit's velocity from tick to tick (`Mover::momentum`), and a unit of them without a goal brakes off what it has
//! left, moving meanwhile (`Locomotion::Stopping`; `docs/movement.md > 6.7`, `> 6.11`). What I3, I5 and I7 allow it,
//! and (M5-V2) the wheeled invariant W1, that a FixedTurnRadiusMovement unit turns by its speed alone.

use glam::Vec2;

use super::Seen;
use crate::defs::{MovementAttrs, MovementModel};
use crate::math;
use crate::mover::{Locomotion, Mover};

/// Whether a unit with these movement numbers carries momentum. No other model has any, ever (a Glider's is its speed:
/// `super::gliders`).
pub(super) fn carries(movement: &MovementAttrs) -> bool {
    matches!(movement.model(), MovementModel::Drag | MovementModel::FixedTurnRadius)
}

/// I7: a unit with momentum braking it off without a goal (`before`, as it was a tick before): it is Stopping, its
/// velocity is that momentum, or it met an obstacle in this tick, which left it none (design change 0035), and it is
/// slower than a tick before, every tick, so it does come to rest.
pub(super) fn braking(mover: &Mover, before: Option<&Seen>) -> bool {
    mover.locomotion == Locomotion::Stopping
        && (mover.velocity == mover.momentum || mover.momentum == Vec2::ZERO)
        && before.is_some_and(|b| mover.velocity.length_squared() < b.momentum.length_squared())
}

/// W1 (slice M5-V2): a FixedTurnRadiusMovement unit never turns in place (`systems/units.md >
/// FixedTurnRadiusMovement`: "Unit cannot turn in place"). In a tick it turns by at most its speed in it over its
/// MinimumTurnRadius, and that speed is at most the momentum it carried in (`before`) and one tick's acceleration or
/// braking: a unit pressed against a wall, which carried none in, turns by its single-tick acceleration's (design
/// change 0035). The violation, if any; units of the other models have none.
pub(super) fn turns_by_its_speed(movement: &MovementAttrs, facing: f32, before: &Seen) -> Option<String> {
    let MovementAttrs::FixedTurnRadius {
        minimum_turn_radius,
        forward_acceleration,
        reverse_acceleration,
        deceleration,
        ..
    } = *movement
    else {
        return None;
    };
    let change = [forward_acceleration, reverse_acceleration, deceleration]
        .into_iter()
        .map(|a| a.unwrap_or(f32::INFINITY))
        .fold(0.0, f32::max);
    let speed = before.momentum.length() + change;
    let turned = math::wrap_angle(facing - before.facing).abs();
    (turned > speed / minimum_turn_radius * 1.0001 + 1e-5)
        .then(|| format!("turned {turned} rad in one tick, more than its speed of at most {speed} su allows"))
}
