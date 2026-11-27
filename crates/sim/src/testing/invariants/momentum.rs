//! (M3-G, M5-V2) The movement models with momentum (`movement::model`): DragMovement and FixedTurnRadiusMovement carry
//! a unit's velocity from tick to tick (`Mover::momentum`), and a unit of them without a goal brakes off what it has
//! left, moving meanwhile (`Locomotion::Stopping`; `docs/movement.md > 6.7`, `> 6.11`). What I3, I5 and I7 allow it.

use glam::Vec2;

use super::Seen;
use crate::defs::{MovementAttrs, MovementModel};
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
