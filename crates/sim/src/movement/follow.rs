//! One unit's tick along its path — `docs/movement.md > 6.2`, `> 6.3`: the steering waypoint, the turn, the step
//! by the shared rules (`super::collide`), arrival, and the stuck clock (`super::stuck`). Shared by every movement
//! model; the model decides how fast the unit approaches and what becomes of the step: a unit that stops within
//! the tick takes it as it is, a DragMovement unit thrusts towards it (`super::drag`, slice M3-G), (M5-V2) a
//! FixedTurnRadiusMovement unit turns to head along it and drives (`super::wheeled`), and (M5-V3) a
//! SpeedTurnRadiusMovement unit does so by the tread law, at the speed its corner choice gives each turn
//! (`super::tracked`).

use glam::Vec2;

use super::collide::{Moved, World};
use super::model::Model;
use super::{Chased, stand, stuck, turn_rate, wait, yielding};
use crate::defs::MovementAttrs;
use crate::ids::ObjectId;
use crate::math;
use crate::mover::{Locomotion, MoveGoal, MoveStatus, Mover, Orientation, PathState};
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
/// A unit with momentum that didn't steer in this tick (no goal, no path yet, its move ended before it stepped)
/// brakes off the momentum it still has ([`Model::coast`]); every other unit then stands.
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
    let model = Model::of(movement, mover, rugged);
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
///   unit (`systems/combat.md > LocomotionOrientationConstraints[DragMovement]`) turns as it moves and never stands to
///   turn. (M5-V1) A Silhouette that turns with the facing turns only as far as it stays free (`World::turn`), and
///   then moves in its turned shape; a turn cut short is a blocked move, and the unit creeps on ([`creep_on`]).
///   (M5-V2) A FixedTurnRadiusMovement unit turns by its speed as it drives (`super::wheeled`), and steers straight
///   on while its waypoint lies inside its turning circle: the forward loop (`Wheeled::steer`, design change 0353).
///   (M5-V3) A SpeedTurnRadiusMovement unit turns by the tread law as it drives (`super::tracked`), slowing for each
///   turn to the speed of its corner choice, and for the turn at its steering waypoint in time (`Tracked::ahead`). A
///   unit that spins in place stands to turn, as a TurnRate unit does, and the stuck rules see it as they see a
///   blocked unit.
/// - **Arrival**: when the goal is within one step and the way is free, the unit stands exactly on it and stops
///   with `status = Arrived`. The arrival radius is one tick of travel, so it never overshoots. A DragMovement
///   unit's step is its approach speed ([`Drag::approach`]), which brings it onto the goal at a speed it stops
///   from within the tick.
/// - **An object goal** (`chased`; `systems/controls/command_behaviors.md > MovingToObject`): the unit has `Arrived` as
///   soon as the gap between its shape and the object's is within the goal's `reach`. Its path leads to where the
///   object was when it was planned (phase 5c keeps it up to date, `paths::chase_check`): at its end the unit has
///   `Arrived` too if the object is still there (it stands on the reachable point nearest to it, design change
///   0025), and plans again if the object has moved on. An object that is gone ends the move `TargetLost`.
/// - **A contact step under way** (`Mover::step_aside`, design change 0323) comes before everything below: the
///   unit takes its next part ([`World::step_aside`]) and nothing else.
/// - Otherwise it moves `step = MaxSpeed × speed_factor` towards the target: a step towards a lane clear of the
///   units it has to pass, if there are some ahead ([`World::lane_step`]), else by the rules of
///   [`World::resolve`]. It asks the idle units of its owner in its way to step aside ([`yielding::ask`], and
///   [`yielding::ask_ahead`] for a group it walks through), then its progress is measured ([`stuck`]). For a
///   DragMovement unit that step is what it thrusts towards (`Drag::drive`); for a FixedTurnRadiusMovement unit,
///   what it heads along (`Wheeled::drive`), and a unit its own move meets is asked to step aside as well.
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
    model: Option<&Model>,
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
        if world.body().shape_at(*pos).gap(&chased.shape) <= reach {
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
    // (M5-V2) A wheeled unit passes a waypoint on its way, not its goal, that lies inside its turning circle: no turn
    // reaches it from where the unit is, and its path goes on from there (`Wheeled::inside`). Only its goal is
    // reached by a forward loop (design change 0353).
    if let Some(Model::Wheeled(wheeled)) = model {
        while usize::from(mover.next) < last_index
            && wheeled.inside(*facing, mover.path[usize::from(mover.next)] - *pos)
        {
            mover.next += 1;
        }
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
    // (M5-V2) The forward loop: a wheeled unit steers straight on while its waypoint lies inside its turning circle.
    let aim = to;
    let (to, looping) = match model {
        Some(Model::Wheeled(wheeled)) if dist > 0.0 => wheeled.steer(*facing, to),
        _ => (to, false),
    };
    let mut turning = false;
    // The locomotion a model drove by, where it isn't Moving.
    let mut locomotion = None;
    // A unit with momentum turns towards the step it takes, once the rules have given it (`Drag::drive`).
    if dist > 0.0 && model.is_none() {
        let wanted = math::wrap_angle(math::vector_angle(to));
        // Phase 5d turns a unit first in the tick: the whole budget is left.
        let from = *facing;
        let turn = math::turn_toward(facing, wanted, turn_rate(movement));
        *turned = turn.angle;
        // (M5-V1) A Silhouette that turns with the facing turns only as far as it stays free (`World::turn`).
        let turn_made = world.turn(*pos, from, *facing);
        if turn_made.cut {
            *facing = turn_made.facing;
            *turned = math::wrap_angle(turn_made.facing - from).abs();
            creep_on(*pos, to, turn_made.blocker, mover, world, requests, chased);
            turning = true;
        } else if !turn.done {
            // Turning in place for the whole tick.
            stand(mover);
            mover.orientation = Orientation::Turning(target);
            return false;
        } else {
            turning = turn.angle != 0.0;
        }
    }
    // How far the unit would go this tick: a whole step, or the speed a unit with momentum still stops from at
    // the end of its path.
    let reach = model.map_or(step, |model| {
        model.approach(stuck::remaining(*pos, &mover.path, usize::from(mover.next)))
    });
    let arriving = !looping && usize::from(mover.next) == last_index && dist <= reach;
    // (M5-V3) A tracked unit slows in time for the turn at its steering waypoint, onto the path's next leg.
    let ahead = match model {
        Some(Model::Tracked(tracked)) if usize::from(mover.next) < last_index => {
            let beyond = mover.path[usize::from(mover.next) + 1];
            tracked.ahead(world, *pos, target, beyond, step)
        }
        _ => f32::INFINITY,
    };
    // (M5-V3) It spins in place this tick: it stands to turn.
    let mut spinning = false;
    let d = if dist > reach { to * (reach / dist) } else { to };
    touched.clear();
    // (G) First a step towards a lane clear of the units it has to pass, if there are some ahead; else the rules
    // after the sweep (`docs/movement.md > 6.3`). The idle units it is about to walk into are asked to part.
    // (0323) A unit taking a contact step takes its next part, and nothing else, until the step is made.
    let stepping = world.step_aside(*pos, d, reach, mover, touched);
    let (shift, lane) = if arriving || stepping.is_some() {
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
    let moved = stepping.or(lane).unwrap_or_else(|| {
        touched.clear();
        // A unit with momentum goes round a standing unit on the near side (`World::around`).
        let near_side = model.is_some();
        let snap = arriving.then_some(target);
        world.resolve_for(*pos, d, reach, snap, mover, tick, touched, near_side)
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
        Some(Model::Drag(drag)) => {
            *steered = true;
            let turn = drag.drive(from, facing, turn_rate(movement), d, moved, mover, world, touched);
            *turned = turn.1;
            turning = turn.1 != 0.0;
            turn.0
        }
        // (M5-V2) A wheeled unit heads along that step and drives at its speed.
        Some(Model::Wheeled(wheeled)) => {
            *steered = true;
            let lands = arriving && moved.pos == target;
            // A loop the way ahead blocks can't be driven: the unit turns towards its waypoint as it presses on, its
            // turns' blocked parts dropped (design change 0035), rather than into what blocks it.
            let asked = if looping { aim * (d.length() / dist) } else { d };
            let driven = wheeled.drive(from, facing, asked, moved, lands, mover, world, touched);
            *turned = driven.turned;
            turning = driven.turned != 0.0;
            locomotion = Some(driven.locomotion);
            if let Some(blocker) = driven.cut {
                creep_on(from, to, blocker, mover, world, requests, chased);
            }
            // The unit its own move met, where the step it heads along didn't (it turns round, or loops), is asked
            // to step aside as well (design change 0353: "Infantry idle in its path step aside").
            if driven.moved.blocker != moved.blocker {
                yielding::ask(world, from, driven.velocity, &driven.moved, false, mover, requests);
            }
            driven.moved
        }
        // (M5-V3) A tracked unit takes a contact step as it is, sideways, its facing kept and no speed carried; else it
        // heads along that step by the tread law, at the speed of the corner choice.
        Some(Model::Tracked(_)) if mover.step_aside.is_some() => {
            *steered = true;
            mover.momentum = Vec2::ZERO;
            moved
        }
        Some(Model::Tracked(tracked)) => {
            *steered = true;
            let lands = arriving && moved.pos == target;
            let driven = tracked.drive(from, facing, d, moved, lands, ahead, dist, mover, world, touched);
            *turned = driven.turned;
            turning = driven.turned != 0.0;
            locomotion = Some(driven.locomotion);
            spinning = driven.velocity == Vec2::ZERO && turning && driven.cut.is_none();
            if let Some(blocker) = driven.cut {
                creep_on(from, to, blocker, mover, world, requests, chased);
            }
            if driven.moved.blocker != moved.blocker {
                yielding::ask(world, from, driven.velocity, &driven.moved, false, mover, requests);
            }
            driven.moved
        }
        None => moved,
    };
    if spinning {
        // Turning in place for the whole tick, as a TurnRate unit stands to turn: it doesn't get on, so its stuck clock
        // runs, and the stuck rules see it, as they see a blocked unit (a spin that never ended would be a unit that
        // never got on).
        mover.momentum = Vec2::ZERO;
        stand(mover);
        mover.orientation = Orientation::Turning(target);
        return stuck::after_move(*pos, mover, step, &moved, world, chased, engaged, tick);
    }
    *pos = moved.pos;
    // (0323) The contact step is made once the unit is on its end, or past it.
    if let Some(aside) = mover.step_aside
        && aside.is_done(from, *pos)
    {
        mover.step_aside = None;
    }
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
        locomotion.unwrap_or(Locomotion::Moving)
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

/// (M5-V1) `systems/units.md > UnitCollision > Collision Response (Un-sticking)` (design change 0035): the unit at
/// `pos`, its steering target `to` away, turned towards it, and its Silhouette, which turns with its facing, met an
/// obstacle on the way (`World::turn`, plan decision T78). A turn cut short is a blocked move: "the blocked portion of
/// the movement is dropped (the unit stops at contact instead of overlapping) and its velocity is set to 0". The unit
/// doesn't stand to wait for the turn ("Blocked units don't freeze; they creep and turn out of the contact"): it takes
/// its step all the same, in the shape it turned to, and turns on from where that leaves it in the next tick, so it
/// scrapes out of the contact as a unit pressed against a wall creeps along it. The unit that stopped the turn, if one
/// did (`blocker`), is asked to make way (`yielding::ask`), unless it is the unit it heads for.
fn creep_on(
    pos: Vec2,
    to: Vec2,
    blocker: Option<usize>,
    mover: &mut Mover,
    world: &World,
    requests: &mut Vec<yielding::Request>,
    chased: Option<&Chased>,
) {
    mover.momentum = Vec2::ZERO;
    let stopped = Moved {
        pos,
        forward: false,
        blocker,
        aside: false,
        slid: false,
    };
    let blocked_by_goal = chased
        .zip(blocker)
        .is_some_and(|(chased, blocker)| world.bodies[blocker].id == chased.id);
    if !blocked_by_goal {
        yielding::ask(world, pos, to, &stopped, false, mover, requests);
    }
}

#[cfg(test)]
mod tests;
