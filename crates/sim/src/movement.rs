//! Phase 5d, the one movement pipeline — `docs/movement.md > 4`, `> 6.3`. The only code that writes a unit's
//! `pos` (besides a debug teleport).
//!
//! This is the **naive** mover of slice B: each unit follows its path's waypoints at its top speed, turns by the
//! TurnRate rule and snaps exactly onto its goal. Units don't collide with each other yet; slice D replaces this
//! step with steering and collision (`docs/movement.md > 6`), keeping its interface: goal and path in; position,
//! facing, velocity, channels and status out.

use std::collections::BTreeMap;

use glam::Vec2;

use crate::defs::MovementAttrs;
use crate::ids::ObjectId;
use crate::math;
use crate::mover::{Locomotion, MoveGoal, MoveStatus, Mover, Orientation, PathState};
use crate::object::Object;

/// Phase 5d: move every unit before `first_new` one tick along its path, in id order.
pub(crate) fn step(objects: &mut BTreeMap<ObjectId, Object>, first_new: ObjectId) {
    for o in objects.range_mut(..first_new).map(|(_, o)| o) {
        let Some(unit) = &o.kind.def().unit else {
            continue;
        };
        let Some(mover) = &mut o.mover else {
            continue;
        };
        step_unit(&mut o.pos, &mut o.facing, mover, &unit.movement);
    }
}

/// One unit's tick (`docs/movement.md > 6.3`, without collision).
///
/// - Without a goal, or while waiting for its path, the unit stands still (G2; `> 5.5`).
/// - It walks `step = MaxSpeed × speed_factor` along the path, through as many waypoints as the step reaches.
/// - **Facing** (`systems/combat.md > LocomotionOrientationConstraints[TurnRateMovement]`): before each leg the
///   unit turns to face it, at most TurnRate per tick in all. If the turn doesn't complete, the unit turns in
///   place for the rest of the tick. Every M1 unit turns 180° per tick, so it always completes.
/// - **Arrival**: when the last waypoint is within the step, the unit stands exactly on it and stops, with
///   `status = Arrived`. The arrival radius is one tick of travel, so it never overshoots.
fn step_unit(pos: &mut Vec2, facing: &mut f32, mover: &mut Mover, movement: &MovementAttrs) {
    if mover.goal == MoveGoal::None || mover.path_state != PathState::Ready {
        mover.velocity = Vec2::ZERO;
        mover.locomotion = Locomotion::Stationary;
        mover.orientation = Orientation::Maintaining;
        return;
    }
    let start = *pos;
    let mut budget = movement.max_speed() * mover.speed_factor;
    // TODO(movement models): only TurnRateMovement (every M1 unit) is modelled; the others turn without limit here.
    let mut turn_left = movement.turn_rate().unwrap_or(math::PI);
    let mut turning_toward = None;
    let mut arrived = false;
    loop {
        let index = usize::from(mover.next);
        let Some(&target) = mover.path.get(index) else {
            // Past the last waypoint (an empty path): the unit is where the path ends.
            arrived = true;
            break;
        };
        let last = index + 1 >= mover.path.len();
        let to = target - *pos;
        let dist = to.length();
        if dist > 0.0 {
            let wanted = math::wrap_angle(math::vector_angle(to));
            let turn = math::wrap_angle(wanted - *facing);
            if turn.abs() > turn_left {
                *facing = math::wrap_angle(*facing + turn_left.copysign(turn));
                turning_toward = Some(target);
                break;
            }
            if turn != 0.0 {
                turn_left -= turn.abs();
                *facing = wanted;
                turning_toward = Some(target);
            }
        }
        if dist > budget {
            *pos += to * (budget / dist);
            break;
        }
        *pos = target;
        budget -= dist;
        if last {
            arrived = true;
            break;
        }
        mover.next = mover.next.saturating_add(1);
        if budget <= 0.0 {
            break;
        }
    }
    if arrived {
        mover.finish(MoveStatus::Arrived);
        return;
    }
    mover.velocity = *pos - start;
    mover.locomotion = if mover.velocity == Vec2::ZERO {
        Locomotion::Stationary
    } else {
        Locomotion::Moving
    };
    mover.orientation = turning_toward.map_or(Orientation::Maintaining, Orientation::Turning);
}

#[cfg(test)]
mod tests {
    use super::*;

    const WALKER: MovementAttrs = MovementAttrs::TurnRate {
        turn_rate: math::PI,
        acceleration: None,
        deceleration: None,
        max_speed: 4.0,
    };

    fn walking(path: &[Vec2]) -> Mover {
        let mut mover = Mover::default();
        mover.go_to(MoveGoal::Point(*path.last().unwrap()), None, 1);
        mover.follow(path.to_vec(), *path.last().unwrap(), 0);
        mover
    }

    #[test]
    fn walks_through_waypoints_and_snaps_onto_the_goal() {
        let path = [Vec2::new(0.0, 0.0), Vec2::new(6.0, 0.0), Vec2::new(6.0, 5.0)];
        let mut mover = walking(&path);
        let (mut pos, mut facing) = (path[0], 0.0);
        step_unit(&mut pos, &mut facing, &mut mover, &WALKER);
        assert_eq!(pos, Vec2::new(4.0, 0.0));
        assert_eq!(
            (facing, mover.velocity, mover.locomotion),
            (0.0, Vec2::new(4.0, 0.0), Locomotion::Moving)
        );
        assert_eq!(mover.orientation, Orientation::Maintaining);
        // Round the corner within one tick: 2 su east, then 2 su south, facing south.
        step_unit(&mut pos, &mut facing, &mut mover, &WALKER);
        assert_eq!(pos, Vec2::new(6.0, 2.0));
        assert_eq!(facing, math::PI / 2.0);
        assert_eq!(mover.orientation, Orientation::Turning(path[2]));
        // 3 su left: arrive exactly.
        step_unit(&mut pos, &mut facing, &mut mover, &WALKER);
        assert_eq!(pos, path[2]);
        assert_eq!((mover.status, mover.goal), (MoveStatus::Arrived, MoveGoal::None));
        assert_eq!((mover.velocity, mover.locomotion), (Vec2::ZERO, Locomotion::Stationary));
        // And stays.
        step_unit(&mut pos, &mut facing, &mut mover, &WALKER);
        assert_eq!(pos, path[2]);
    }

    #[test]
    fn a_slow_turn_is_made_in_place() {
        let slow = MovementAttrs::TurnRate {
            turn_rate: 0.5,
            acceleration: None,
            deceleration: None,
            max_speed: 4.0,
        };
        let path = [Vec2::ZERO, Vec2::new(0.0, 100.0)];
        let mut mover = walking(&path);
        let (mut pos, mut facing) = (path[0], 0.0);
        step_unit(&mut pos, &mut facing, &mut mover, &slow);
        assert_eq!((pos, facing), (Vec2::ZERO, 0.5));
        assert_eq!(mover.locomotion, Locomotion::Stationary);
        assert_eq!(mover.orientation, Orientation::Turning(path[1]));
        step_unit(&mut pos, &mut facing, &mut mover, &slow);
        step_unit(&mut pos, &mut facing, &mut mover, &slow);
        assert_eq!(pos, Vec2::ZERO);
        // The last 0.07 rad complete within the tick, and the unit walks.
        step_unit(&mut pos, &mut facing, &mut mover, &slow);
        assert_eq!((pos, facing), (Vec2::new(0.0, 4.0), math::PI / 2.0));
    }

    #[test]
    fn without_a_path_the_unit_waits() {
        let mut mover = Mover::default();
        mover.go_to(MoveGoal::Point(Vec2::new(50.0, 0.0)), None, 1);
        let (mut pos, mut facing) = (Vec2::ZERO, 1.0);
        step_unit(&mut pos, &mut facing, &mut mover, &WALKER);
        assert_eq!((pos, facing, mover.status), (Vec2::ZERO, 1.0, MoveStatus::Moving));
    }
}
