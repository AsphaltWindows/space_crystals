//! One unit's tick along its path — `docs/movement.md > 6.2`, `> 6.3`: the steering waypoint, the turn, the step
//! by the shared rules (`super::collide`), arrival, and the stuck clock (`super::stuck`). Shared by every movement
//! model; the model decides how fast the unit approaches and what becomes of the step: a unit that stops within
//! the tick takes it as it is, a DragMovement unit thrusts towards it (`super::drag`, slice M3-G).

use glam::Vec2;

use super::collide::World;
use super::drag::Drag;
use super::{Chased, stand, stuck, turn_rate, wait, yielding};
use crate::defs::MovementAttrs;
use crate::ids::ObjectId;
use crate::math;
use crate::mover::{Locomotion, MoveGoal, MoveStatus, Mover, Orientation, PathState};
use crate::shape::Shape;
use crate::units::Tick;

/// One unit's tick (`docs/movement.md > 6.2`, `> 6.3`) on open ground: [`step_unit_on`] off Rugged terrain.
/// Returns whether its move failed this tick. For the unit tests of one unit's tick.
#[cfg(test)]
#[expect(
    clippy::too_many_arguments,
    reason = "the unit's state, split into the borrows the step holds"
)]
pub(super) fn step_unit(
    pos: &mut Vec2,
    facing: &mut f32,
    turned: &mut f32,
    mover: &mut Mover,
    movement: &MovementAttrs,
    world: &mut World,
    touched: &mut Vec<usize>,
    requests: &mut Vec<yielding::Request>,
    chased: Option<&Chased>,
    engaged: &[Option<ObjectId>],
    tick: Tick,
) -> bool {
    step_unit_on(
        pos, facing, turned, mover, movement, false, world, touched, requests, chased, engaged, tick,
    )
}

/// One unit's tick (`docs/movement.md > 6.2`, `> 6.3`). Returns whether its move failed this tick. `rugged` is
/// whether the tile under its centre is Rugged terrain, which only a DragMovement unit's model asks
/// (`systems/units.md > HoverVehicle > RuggedTerrain`).
///
/// A DragMovement unit that didn't steer in this tick (no goal, no path yet, its move ended before it stepped)
/// brakes off the momentum it still has ([`Drag::coast`]); every other unit then stands.
#[expect(
    clippy::too_many_arguments,
    reason = "the unit's state, split into the borrows the step holds"
)]
pub(super) fn step_unit_on(
    pos: &mut Vec2,
    facing: &mut f32,
    turned: &mut f32,
    mover: &mut Mover,
    movement: &MovementAttrs,
    rugged: bool,
    world: &mut World,
    touched: &mut Vec<usize>,
    requests: &mut Vec<yielding::Request>,
    chased: Option<&Chased>,
    engaged: &[Option<ObjectId>],
    tick: Tick,
) -> bool {
    let model = Drag::of(movement, mover.speed_factor, rugged);
    let mut steered = false;
    let failed = follow(
        pos,
        facing,
        turned,
        mover,
        movement,
        model.as_ref(),
        &mut steered,
        world,
        touched,
        requests,
        chased,
        engaged,
        tick,
    );
    if let Some(model) = model.filter(|_| !steered) {
        model.coast(pos, *facing, mover, world, touched);
    }
    failed
}

/// [`step_unit_on`] for a unit of movement model `model` (`None`: one that stops within the tick). `steered` is
/// set once the model has integrated the unit's velocity for this tick.
///
/// - Without a goal, or while waiting for its path, the unit stands still (`> 5.5`). Only its own sidestep moves
///   an idle unit (`yielding`); nothing pushes it.
/// - **Steering target**: the path's waypoint `Mover::next`. A waypoint within one step is passed without being
///   touched; the staggered check in phase 5c moves on to the farthest waypoint in sight.
/// - **Facing** (`systems/combat.md > LocomotionOrientationConstraints[TurnRateMovement]`): the unit turns towards
///   the steering target, at most TurnRate per tick, and moves only once it faces it. Every M1 unit turns 180°
///   per tick, so its turn always completes. The facing follows the path, not a sidestep. The angle turned goes to
///   `turned`: the aim in phase 5e has only the rest of the tick's TurnRate ([`super::Turns`]). A DragMovement
///   unit (`> LocomotionOrientationConstraints[DragMovement]`) turns as it moves and never stands to turn.
/// - **Arrival**: when the goal is within one step and the way is free, the unit stands exactly on it and stops
///   with `status = Arrived`. The arrival radius is one tick of travel, so it never overshoots. A DragMovement
///   unit's step is its approach speed ([`Drag::approach`]), which brings it onto the goal at a speed it stops
///   from within the tick.
/// - **An object goal** (`chased`; `systems/controls/behaviors.md > MovingToObject`): the unit has `Arrived` as soon
///   as the gap between its shape and the object's is within the goal's `reach`. Its path leads to where the
///   object was when it was planned (phase 5c keeps it up to date, `paths::chase_check`): at its end the unit has
///   `Arrived` too if the object is still there (it stands on the reachable point nearest to it, design change
///   0025), and plans again if the object has moved on. An object that is gone ends the move `TargetLost`.
/// - Otherwise it moves `step = MaxSpeed × speed_factor` towards the target: a step towards a lane clear of the
///   units it has to pass, if there are some ahead ([`World::lane_step`]), else by the rules of
///   [`World::resolve`]. It asks the idle units of its owner in its way to step aside ([`yielding::ask`], and
///   [`yielding::ask_ahead`] for a group it walks through), then its progress is measured ([`stuck`]). For a
///   DragMovement unit that step is what it thrusts towards ([`Drag::drive`]).
#[expect(
    clippy::too_many_arguments,
    reason = "the unit's state, split into the borrows the step holds"
)]
fn follow(
    pos: &mut Vec2,
    facing: &mut f32,
    turned: &mut f32,
    mover: &mut Mover,
    movement: &MovementAttrs,
    model: Option<&Drag>,
    steered: &mut bool,
    world: &mut World,
    touched: &mut Vec<usize>,
    requests: &mut Vec<yielding::Request>,
    chased: Option<&Chased>,
    engaged: &[Option<ObjectId>],
    tick: Tick,
) -> bool {
    if let MoveGoal::Object { reach, .. } = mover.goal {
        let Some(chased) = chased else {
            mover.finish(MoveStatus::TargetLost);
            return false;
        };
        if Shape::rect(*pos, world.body().half).gap(&chased.shape) <= reach {
            mover.finish(MoveStatus::Arrived);
            return false;
        }
    }
    if mover.goal == MoveGoal::None || mover.path_state != PathState::Ready {
        wait(mover, tick);
        return false;
    }
    world.counters.steered += 1;
    let step = movement.max_speed() * mover.speed_factor;
    let last_index = mover.path.len().saturating_sub(1);
    while usize::from(mover.next) < last_index && pos.distance(mover.path[usize::from(mover.next)]) <= step {
        mover.next += 1;
    }
    // (M3-G2) Past the last waypoint of a detour round standing units: the rest is the path as it was planned.
    if mover.detour_end <= mover.next {
        mover.detour_end = 0;
    }
    let Some(&target) = mover.path.get(usize::from(mover.next)) else {
        // An empty path: the unit is where it ends.
        mover.finish(MoveStatus::Arrived);
        return false;
    };
    let to = target - *pos;
    let dist = to.length();
    // TODO(movement models): TurnRateMovement and DragMovement are modelled; the others turn without limit here.
    let mut turning = false;
    // A unit with momentum turns towards the step it takes, once the rules have given it (`Drag::drive`).
    if dist > 0.0 && model.is_none() {
        let wanted = math::wrap_angle(math::vector_angle(to));
        // Phase 5d turns a unit first in the tick: the whole budget is left.
        let turn = math::turn_toward(facing, wanted, turn_rate(movement));
        *turned = turn.angle;
        if !turn.done {
            // Turning in place for the whole tick.
            stand(mover);
            mover.orientation = Orientation::Turning(target);
            return false;
        }
        turning = turn.angle != 0.0;
    }
    // How far the unit would go this tick: a whole step, or the speed a unit with momentum still stops from at
    // the end of its path.
    let reach = model.map_or(step, |model| {
        model.approach(stuck::remaining(*pos, &mover.path, usize::from(mover.next)))
    });
    let arriving = usize::from(mover.next) == last_index && dist <= reach;
    let d = if dist > reach { to * (reach / dist) } else { to };
    touched.clear();
    // (G) First a step towards a lane clear of the units it has to pass, if there are some ahead; else the rules
    // after the sweep (`docs/movement.md > 6.3`). The idle units it is about to walk into are asked to part.
    // (M3-G2) A unit that waits in its queue makes no step aside: no lane, no way round (`World::hold`).
    let waits = stuck::waits_in_queue(mover, world.me, chased, engaged, tick);
    let (shift, lane) = if arriving || waits {
        (None, None)
    } else {
        // The end of its way, if it is steering at it.
        let to_goal = if usize::from(mover.next) == last_index {
            dist
        } else {
            f32::INFINITY
        };
        world.lane_step(*pos, d, to_goal, chased.map(|chased| chased.id), mover, touched)
    };
    if let Some(shift) = shift {
        yielding::ask_ahead(world, d, shift, mover, requests);
    }
    let moved = lane.unwrap_or_else(|| {
        touched.clear();
        if waits {
            return world.hold(*pos, d, arriving.then_some(target), mover, tick, touched);
        }
        // A unit with momentum goes round a standing unit on the near side (`World::around`).
        let near_side = model.is_some();
        world.resolve_for(*pos, d, arriving.then_some(target), mover, tick, touched, near_side)
    });
    let from = *pos;
    // The unit it heads for isn't in its way: it is where the unit is going.
    let blocked_by_goal = chased
        .zip(moved.blocker)
        .is_some_and(|(chased, blocker)| world.bodies[blocker].id == chased.id);
    if !blocked_by_goal {
        yielding::ask(world, from, d, &moved, shift.is_some(), mover, requests);
    }
    // (M3-G) A unit with momentum thrusts towards that step and goes where its velocity takes it.
    let moved = match model {
        Some(model) => {
            *steered = true;
            let turn = model.drive(from, facing, turn_rate(movement), d, moved, mover, world, touched);
            *turned = turn.1;
            turning = turn.1 != 0.0;
            turn.0
        }
        None => moved,
    };
    *pos = moved.pos;
    if arriving && moved.pos == target {
        // It landed at a speed that one tick of braking takes away (`Drag::approach`): it is at rest.
        mover.momentum = Vec2::ZERO;
        match chased {
            // The object moved on while the unit walked to where it was: plan again (not a counted repath).
            Some(chased) if chased.pos != mover.planned_target => {
                stand(mover);
                mover.repath(tick);
            }
            _ => mover.finish(MoveStatus::Arrived),
        }
        return false;
    }
    mover.velocity = *pos - from;
    mover.locomotion = if mover.velocity == Vec2::ZERO {
        Locomotion::Stationary
    } else {
        Locomotion::Moving
    };
    mover.orientation = if turning {
        Orientation::Turning(target)
    } else {
        Orientation::Maintaining
    };
    // (M3-G) A unit with momentum takes a dozen ticks over a step aside that the others make in a few: while
    // it moves, it is getting round its blocker, not standing behind it.
    let getting_round = model.is_some() && moved.aside && moved.pos != from;
    mover.blocked_ticks = if moved.forward || getting_round {
        0
    } else {
        mover.blocked_ticks.saturating_add(1)
    };
    stuck::after_move(*pos, mover, step, &moved, world, chased, engaged, tick)
}

#[cfg(test)]
mod tests;
