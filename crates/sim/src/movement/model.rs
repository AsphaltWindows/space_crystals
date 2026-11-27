//! The movement models with momentum, which carry a unit's velocity from tick to tick (`Mover::momentum`): DragMovement
//! (`super::drag`, slice M3-G) and (M5-V2) FixedTurnRadiusMovement (`super::wheeled`). Each replaces only the "desired
//! direction → facing and velocity" part of a unit's tick (`docs/movement.md > 6.3`, `> 6.7`): the shared rules give
//! it the step a unit without momentum would take (`super::follow`), and the model drives towards it. A unit of
//! TurnRateMovement stops within the tick and has none, and a Glider flies by its own model (`super::glider`).

use glam::Vec2;

use super::collide::World;
use super::drag::Drag;
use super::wheeled::Wheeled;
use crate::defs::MovementAttrs;
use crate::mover::Mover;

/// One unit's model with momentum for this tick.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) enum Model {
    Drag(Drag),
    Wheeled(Wheeled),
}

impl Model {
    /// The model of a unit with these movement numbers and this mover, `None` for one that stops within the tick.
    /// `rugged` is whether the tile under its centre is Rugged terrain, which only DragMovement asks
    /// (`systems/units.md > HoverVehicle > RuggedTerrain`).
    pub(super) fn of(movement: &MovementAttrs, mover: &Mover, rugged: bool) -> Option<Self> {
        Drag::of(movement, mover.speed_factor, rugged)
            .map(Model::Drag)
            .or_else(|| Wheeled::of(movement, mover).map(Model::Wheeled))
    }

    /// The speed the unit approaches a goal `dist` space units along its path at, from which it stops on it.
    pub(super) fn approach(&self, dist: f32) -> f32 {
        match self {
            Model::Drag(drag) => drag.approach(dist),
            Model::Wheeled(wheeled) => wheeled.approach(dist),
        }
    }

    /// For a unit held to the line along the unit vector `dir` (a doorway's walk): its speed along `dir` this tick,
    /// towards `want`, and the angle it turned towards `dir` (a DragMovement unit at most `turn_rate`).
    pub(super) fn along(&self, dir: Vec2, want: f32, facing: &mut f32, turn_rate: f32, momentum: Vec2) -> (f32, f32) {
        match self {
            Model::Drag(drag) => drag.along(dir, want, facing, turn_rate, momentum),
            Model::Wheeled(wheeled) => wheeled.along(dir, want, facing, momentum),
        }
    }

    /// A unit that isn't steering this tick brakes off the momentum it still has (`Locomotion::Stopping`).
    pub(super) fn coast(
        &self,
        pos: &mut Vec2,
        facing: f32,
        mover: &mut Mover,
        world: &mut World,
        touched: &mut Vec<usize>,
    ) {
        match self {
            Model::Drag(drag) => drag.coast(pos, facing, mover, world, touched),
            Model::Wheeled(wheeled) => wheeled.coast(pos, facing, mover, world, touched),
        }
    }
}
