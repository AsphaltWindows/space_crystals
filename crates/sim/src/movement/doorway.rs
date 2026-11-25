//! Leaving an exit doorway — `core/entities.md > Structure Type > Exit Doorway` (design changes 0080 and 0086), and
//! (M3-D2) a portal's energy doorway (design change 0145, `factions/colonists/structures/README.md > Portals > Portal
//! Exits (energy doorway)`). The doorways' geometry and their rules are `production::ExitDoor`; the exception itself is
//! applied where every move is tested, in `collide` (`World::door`).

use glam::Vec2;

use super::collide::{Moved, World};
use super::drag::Drag;
use super::{Chased, stand, stuck, turn_rate, yielding};
use crate::defs::MovementAttrs;
use crate::ids::ObjectId;
use crate::math;
use crate::mover::{Locomotion, MoveGoal, MoveStatus, Mover, Orientation, PathState};
use crate::production::{Doorway, EnergyDoorway};
use crate::shape::Shape;
use crate::units::Tick;

/// (M3-D2) How short of where it comes clear of a portal's footprint a unit's step may end and still take it out
/// ([`emerge`]): a sixty-fourth of a space unit, a technical margin against rounding.
const CLEAR_SU: f32 = 1.0 / 64.0;

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
            aside: false,
            slid: false,
        }
    } else {
        Moved {
            pos: world.shuffle(*pos, d, touched),
            forward: false,
            blocker: touched.first().copied(),
            aside: false,
            slid: false,
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

/// (M3-D2) The walk out of a portal's energy doorway (design change 0145, points 1 and 4): the unit moves **straight
/// along its heading** (`Mover::exit_heading`, chosen when it was released: `EnergyDoorway::heading`) until its box
/// is clear of the footprint, through the footprint's own tiles and nothing else (`World::door`). It is its only
/// move while it is in the doorway, with or without a goal: no sidestep, no way round, no lane, so it never leaves
/// the ray, and the ray leaves through an open side. Returns whether its move failed this tick.
///
/// - **How it moves** is its movement model's: a unit that stops within the tick turns to its heading first and then
///   steps its speed; a DragMovement unit (the Drone) turns as it goes and thrusts along the heading, its speed
///   carried from tick to tick ([`Drag::along`]).
/// - **Without a goal**, or while its path is still to come, it walks to where it is just clear of the footprint
///   ([`EnergyDoorway::outside`]) and stands there: the last step lands exactly on that point, a DragMovement unit
///   braking to rest on it ([`Drag::approach`]). Once its path is there it doesn't stop at the edge: it keeps its
///   speed out of the footprint, its path starting where the walk comes clear (`paths::serve::doorway_start`), and
///   the step after next takes it on like any unit (`super::follow`).
/// - **In its way**: it goes up to a unit and no farther, and as far as it can towards a blocked tile; it asks the
///   idle units of its owner to step aside ([`yielding::ask`]) and tries again every tick. Its stuck clock runs
///   meanwhile if it has a path (`stuck::after_move`), so a move that can't begin fails as a Barracks unit's does; a
///   unit that can't get out stays where it got to, "half out at the edge of the footprint", as a normal unit
///   (point 4), until the way is free.
/// - **An object goal** ends as in `super::follow`: the object is gone, or the unit is within its reach. The unit
///   walks on out.
#[expect(
    clippy::too_many_arguments,
    reason = "the unit's state, split into the borrows the step holds"
)]
pub(super) fn emerge(
    pos: &mut Vec2,
    facing: &mut f32,
    turned: &mut f32,
    mover: &mut Mover,
    movement: &MovementAttrs,
    rugged: bool,
    portal: &EnergyDoorway,
    world: &mut World,
    touched: &mut Vec<usize>,
    requests: &mut Vec<yielding::Request>,
    chased: Option<&Chased>,
    engaged: &[Option<ObjectId>],
    tick: Tick,
) -> bool {
    mover.sidestep = None;
    let half = world.body().half;
    if let MoveGoal::Object { reach, .. } = mover.goal {
        match chased {
            None => mover.finish(MoveStatus::TargetLost),
            Some(chased) if Shape::rect(*pos, half).gap(&chased.shape) <= reach => mover.finish(MoveStatus::Arrived),
            Some(_) => {}
        }
    }
    // On its way: its path is there.
    let mut on_its_way = mover.goal != MoveGoal::None && mover.path_state == PathState::Ready;
    if on_its_way && mover.path.get(usize::from(mover.next)).is_none() {
        // An empty path: the unit is where it ends (as in `super::follow`).
        mover.finish(MoveStatus::Arrived);
        on_its_way = false;
    }
    let heading = portal.heading_of(*pos, mover.exit_heading);
    let out = portal.outside(*pos, heading, half);
    let dist = (out - *pos).length();
    let step = movement.max_speed() * mover.speed_factor;
    // What is left of the way: to the end of its path, else to where it is clear of the footprint.
    let left = if on_its_way {
        stuck::remaining(*pos, &mover.path, usize::from(mover.next))
    } else {
        dist
    };
    let model = Drag::of(movement, mover.speed_factor, rugged);
    // How far it goes along its heading this tick.
    let speed = match &model {
        None => {
            let turn = math::turn_toward(
                facing,
                math::wrap_angle(math::vector_angle(heading)),
                turn_rate(movement),
            );
            *turned = turn.angle;
            if !turn.done {
                stand(mover);
                mover.orientation = Orientation::Turning(out);
                return false;
            }
            step.min(left)
        }
        Some(model) => {
            let (speed, turn) = model.along(
                heading,
                model.approach(left),
                facing,
                turn_rate(movement),
                mover.momentum,
            );
            *turned = turn;
            speed
        }
    };
    // Clear of the footprint in this tick: exactly where it comes clear (a hair short of it counts, so that rounding
    // never has it creep out by ever smaller steps), and on along its heading by the rest of its speed if its path
    // is there; without one it lands there.
    let leaves = dist <= step && speed + CLEAR_SU >= dist;
    let lands = leaves && !on_its_way;
    let to = if leaves && on_its_way {
        out + heading * (speed - dist).max(0.0)
    } else if leaves {
        out
    } else {
        *pos + heading * speed
    };
    touched.clear();
    let at = world.straight(*pos, to, touched);
    let whole = at == to;
    let moved = Moved {
        pos: at,
        forward: whole && at != *pos,
        blocker: touched.first().copied(),
        aside: false,
        slid: false,
    };
    yielding::ask(world, *pos, to - *pos, &moved, false, mover, requests);
    mover.velocity = at - *pos;
    *pos = at;
    mover.locomotion = if mover.velocity == Vec2::ZERO {
        Locomotion::Stationary
    } else {
        Locomotion::Moving
    };
    mover.orientation = if *turned != 0.0 {
        Orientation::Turning(out)
    } else {
        Orientation::Maintaining
    };
    mover.blocked_ticks = if whole {
        0
    } else {
        mover.blocked_ticks.saturating_add(1)
    };
    // (M3-G) What it carries into the next tick: all of a whole move, nothing after a landing or a move that met
    // something (design change 0035).
    if model.is_some() {
        mover.momentum = if whole && !lands { mover.velocity } else { Vec2::ZERO };
    }
    if on_its_way {
        world.counters.steered += 1;
        return stuck::after_move(*pos, mover, step, &moved, world, chased, engaged, tick);
    }
    // Waiting for its path: the stuck clock doesn't run (`super::wait`).
    if mover.goal != MoveGoal::None {
        mover.last_progress = mover.last_progress.saturating_add(1).min(tick);
    }
    false
}
