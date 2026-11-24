//! Leaving an exit doorway without a goal — `core/entities.md > Structure Type > Exit Doorway` (design changes 0080
//! and 0086). The doorway's geometry and its rule are `production::Doorway`; the exception itself is applied where
//! every move is tested, in `collide` (`World::door`).

use glam::Vec2;

use super::collide::{Moved, World};
use super::{stand, turn_rate, yielding};
use crate::defs::MovementAttrs;
use crate::math;
use crate::mover::{Locomotion, Mover, Orientation};
use crate::production::Doorway;

/// `core/entities.md > Structure Type > Exit Doorway`, point 2 (design changes 0080 and 0086): a unit in a doorway
/// that has nowhere to go (no rally point, or its order ended there) "walks straight out of the exit side" and
/// stops as soon as its box is clear of the building. It is the one move a unit makes without a goal besides a
/// sidestep, and goes through the same predicates as every move ([`World::clear`], [`World::is_free`]). Straight
/// out only: up to a unit in its way and no step around it, so a unit that can't get out stands in the doorway
/// instead of pacing along the wall ("If the way is blocked, it stops in the doorway"). It asks the idle units of
/// its owner in front of the doorway to step aside (`yielding::ask`, point 4) and tries again every tick.
#[expect(
    clippy::too_many_arguments,
    reason = "the unit's state, split into the borrows the step holds"
)]
pub(super) fn leave(
    pos: &mut Vec2,
    facing: &mut f32,
    turned: &mut f32,
    mover: &mut Mover,
    movement: &MovementAttrs,
    door: &Doorway,
    world: &mut World,
    touched: &mut Vec<usize>,
    requests: &mut Vec<yielding::Request>,
) {
    let out = door.outside(*pos, world.body().half);
    let to = out - *pos;
    let dist = to.length();
    let step = movement.max_speed() * mover.speed_factor;
    if dist <= 0.0 || step <= 0.0 {
        stand(mover);
        return;
    }
    let turn = math::turn_toward(facing, math::wrap_angle(math::vector_angle(to)), turn_rate(movement));
    *turned = turn.angle;
    if !turn.done {
        stand(mover);
        mover.orientation = Orientation::Turning(out);
        return;
    }
    let d = if dist > step { to * (step / dist) } else { to };
    touched.clear();
    let moved = if world.clear(*pos, d, touched) {
        // Within a step of being out, the unit stands exactly against the exit side.
        let at = if dist > step { *pos + d } else { out };
        let free = dist > step || world.is_free(at);
        Moved {
            pos: if free { at } else { *pos },
            forward: free,
            blocker: None,
        }
    } else {
        Moved {
            pos: world.shuffle(*pos, d, touched),
            forward: false,
            blocker: touched.first().copied(),
        }
    };
    yielding::ask(world, *pos, d, &moved, false, mover, requests);
    mover.velocity = moved.pos - *pos;
    *pos = moved.pos;
    mover.locomotion = if mover.velocity == Vec2::ZERO {
        Locomotion::Stationary
    } else {
        Locomotion::Moving
    };
    mover.orientation = Orientation::Maintaining;
    mover.blocked_ticks = if moved.forward {
        0
    } else {
        mover.blocked_ticks.saturating_add(1)
    };
}
