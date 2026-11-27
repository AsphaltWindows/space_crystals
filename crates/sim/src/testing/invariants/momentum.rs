//! (M3-G, M5-V2, M5-V3) The movement models with momentum (`movement::model`): DragMovement,
//! FixedTurnRadiusMovement and SpeedTurnRadiusMovement carry a unit's velocity from tick to tick (`Mover::momentum`),
//! and a unit of them without a goal brakes off what it has left, moving meanwhile (`Locomotion::Stopping`;
//! `docs/movement.md > 6.7`, `> 6.11`, `> 6.12`). What I3, I5 and I7 allow it, (M5-V2) the wheeled invariant W1, that
//! a FixedTurnRadiusMovement unit turns by its speed alone, and (M5-V3) the tracked invariant S1, that a
//! SpeedTurnRadiusMovement unit turns by the tread law.

use glam::Vec2;

use super::Seen;
use crate::defs::{MovementAttrs, MovementModel};
use crate::math;
use crate::mover::{Locomotion, Mover};

/// Whether a unit with these movement numbers carries momentum. No other model has any, ever (a Glider's is its speed:
/// `super::gliders`).
pub(super) fn carries(movement: &MovementAttrs) -> bool {
    matches!(
        movement.model(),
        MovementModel::Drag | MovementModel::FixedTurnRadius | MovementModel::SpeedTurnRadius
    )
}

/// (M5-N10) I5 and I7: how far a unit with these movement numbers that turned in place toward its aim in a tick,
/// without a goal (`movement::turn_in_place`, the hull aim), may move in it at `speed_factor`. A
/// SpeedTurnRadiusMovement unit whose spin a contact cut short backs off it, or creeps toward its aim, by one tick's
/// Acceleration at most, and no farther than its MaxSpeed (`Tracked::spin`, design change 0035). The other models turn
/// in place without moving.
pub(super) fn spin_creep(movement: &MovementAttrs, speed_factor: f32) -> f32 {
    match *movement {
        MovementAttrs::SpeedTurnRadius {
            acceleration,
            max_speed,
            ..
        } => acceleration.unwrap_or(f32::INFINITY).min(max_speed * speed_factor),
        _ => 0.0,
    }
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
/// change 0035). The violation, if any, and its invariant; units of the models that turn at a fixed rate have none.
///
/// (M5-V3) S1: a SpeedTurnRadiusMovement unit turns in a tick by at most the tread law's rate at the slowest speed it
/// may have had in it, `SpinRate × (1 − s ÷ MaxSpeed)` for `s` the momentum it carried in less one tick's braking or
/// acceleration, at least 0 (`systems/units.md > SpeedTurnRadiusMovement > The turning law`, design change 0417): at
/// most SpinRate in place, nothing at MaxSpeed.
pub(super) fn turns_by_its_speed(
    movement: &MovementAttrs,
    facing: f32,
    before: &Seen,
) -> Option<(&'static str, String)> {
    let turned = math::wrap_angle(facing - before.facing).abs();
    if let MovementAttrs::SpeedTurnRadius {
        acceleration,
        deceleration,
        max_speed,
        ..
    } = *movement
    {
        let change = [acceleration, deceleration]
            .into_iter()
            .map(|a| a.unwrap_or(f32::INFINITY))
            .fold(0.0, f32::max);
        let slowest = (before.momentum.length() - change).max(0.0);
        let rate = movement.spin_rate().unwrap_or(0.0) * (1.0 - slowest / max_speed).max(0.0);
        return (turned > rate * 1.0001 + 1e-5).then(|| {
            let message = format!(
                "turned {turned} rad in one tick, more than the {rate} its speed of at least {slowest} su allows"
            );
            ("S1", message)
        });
    }
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
    (turned > speed / minimum_turn_radius * 1.0001 + 1e-5).then(|| {
        let message = format!("turned {turned} rad in one tick, more than its speed of at most {speed} su allows");
        ("W1", message)
    })
}
