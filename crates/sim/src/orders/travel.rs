//! Travelling behaviours — `systems/controls/behaviors.md > MovingToObject` (with Follow), `>
//! AttackMovingToLocation` and `> Patrolling`.

use glam::Vec2;

use crate::ids::ObjectId;
use crate::mover::{MoveGoal, MoveStatus, PathState};
use crate::nav::PathGroup;
use crate::units::gu;

use super::engage::{Closing, close_in};
use super::state::{Approach, Leg, Patrol, Pursuit};
use super::world::{Known, Scan, Unit, World};
use super::{ATTACK_MOVE_LEASH_GU, GIVE_UP_RETRY_TICKS, PATROL_FAILED_LEGS, PROXIMITY_SU};

/// Design change 0139 (`core/vision.md > Explored > Targeting remembered objects`: "The order goes to the object's
/// **last-known position**"): one tick of a unit's walk to `spot`, the last-known position of an object its owner
/// remembers but doesn't see. `walking` says whether the unit's current move is this walk. The walk is a move to
/// the point (`MoveGoal::Point`): the centre of the remembered footprint, which the owner's path knowledge blocks
/// (design change 0040), so the path ends at its edge, as near as the unit gets. It never names the object, so
/// neither the path serving nor the movement reads anything of it, and the walk is the same whether or not the
/// object still stands. It starts once the unit's attack sequence lets it move. Returns whether it is over without
/// the owner having seen the object: arrived as near as it gets, or failed; the order then ends.
pub(super) fn walk_to_remembered(unit: &mut Unit, walking: &mut bool, spot: Vec2, world: &World) -> bool {
    if !*walking {
        if unit.may_move() {
            unit.mover.go_to(MoveGoal::Point(spot), None, world.tick);
            *walking = true;
        }
        return false;
    }
    unit.mover.status != MoveStatus::Moving
}

/// `> MovingToObject`: one tick of a Move to the object `id`, as the unit's owner knows it ([`World::known`]).
/// Returns whether the behaviour is complete.
///
/// **Proximity** (`> Proximity`, design change 0032): a gap of at most [`PROXIMITY_SU`] between the unit's
/// Silhouette and the object's shape, edge to edge.
///
/// **A static object** (`> Completion`: a structure, a map resource): complete within proximity. The move also
/// ends, and the order with it, at the reachable point nearest to an object the unit can't get to (`>
/// MovingToLocation > Unreachable target location`), when the unit is stuck within 3 grid units of it behind units
/// that aren't moving ("Blocked near the target", design change 0033; `movement::stuck`), and when it fails
/// (`> Failure`). Out of the owner's sight, it is where the owner remembers it (design change 0139): the unit walks
/// to its last-known position ([`walk_to_remembered`]) and on to the object once the owner sees it; once the owner
/// sees its place without it, the order ends ("the way it would for a vanished target": `Sim::follow_targets`).
///
/// **A unit: Follow** (design change 0032). The behaviour never completes on reaching it: within proximity the
/// unit stops and waits, and when the target moves out of proximity it follows again. Stuck within 3 grid units
/// behind units that aren't moving, it waits there, with no re-route, until the target moves on. It ends when
/// - the target dies (the removal path completes the order) or **leaves the map**: entering a transport, a
///   building or the Tunnel Network doesn't exist yet. **The hook**: such an object will be gone from
///   `World::get`, or carry a flag this function reads where it looks the target up;
/// - the move fails (`> Failure`);
/// - the target leaves the owner's vision (design change 0034, extended to Follow): the unit goes to its last seen
///   position and the order completes there. The player's own units are always seen.
///
/// **Itself** (design change 0168, `systems/controls/commands.md > Move`: "Following itself = Stop"): a unit whose
/// Move names itself, one of the units a right-click on it ordered, stops: "its order ends at once, as if it had
/// arrived". Queued, the step ends as soon as it starts, and the next one follows. The others follow it.
pub(super) fn approach(unit: &mut Unit, id: ObjectId, state: &mut Approach, world: &World) -> bool {
    if id == unit.id {
        unit.halt();
        return true;
    }
    if state.lost {
        return unit.mover.status != MoveStatus::Moving;
    }
    let target = match world.known(unit.owner, id) {
        Known::Seen(target) => target,
        Known::Remembered { position, .. } => {
            state.moving = false;
            return walk_to_remembered(unit, &mut state.remembered, position, world);
        }
        // A followed unit out of the owner's sight (below), or an object it knows nothing of.
        Known::Gone => {
            state.moving = false;
            return match state.last_seen {
                Some(at) => {
                    unit.mover.go_to(MoveGoal::Point(at), None, world.tick);
                    state.lost = true;
                    false
                }
                None => {
                    unit.halt();
                    true
                }
            };
        }
    };
    // Seen at last: the move goes on to the object itself.
    if std::mem::take(&mut state.remembered) {
        state.moving = false;
    }
    let goal = MoveGoal::Object {
        id,
        reach: PROXIMITY_SU,
    };
    let within = unit.shape().gap(&target.shape) <= PROXIMITY_SU;
    if !target.is_unit {
        if within {
            unit.halt();
            return true;
        }
        if !state.moving {
            unit.mover.go_to(goal, None, world.tick);
            state.moving = true;
            return false;
        }
        return unit.mover.status != MoveStatus::Moving;
    }
    state.last_seen = Some(target.pos);
    if within {
        if state.moving {
            unit.mover.stop();
            state.moving = false;
        }
        state.parked = Some(target.pos);
        return false;
    }
    if state.moving {
        match unit.mover.status {
            MoveStatus::Moving => return false,
            MoveStatus::Failed | MoveStatus::TargetLost | MoveStatus::Idle => return true,
            // It reached the target as far as it can: blocked near it, or at the nearest point it can stand on.
            // It waits there until the target moves.
            MoveStatus::Arrived => {
                state.moving = false;
                state.parked = Some(target.pos);
                return false;
            }
        }
    }
    if state.parked != Some(target.pos) {
        unit.mover.go_to(goal, None, world.tick);
        state.moving = true;
        state.parked = None;
    }
    false
}

/// How a leg stands after [`march`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum LegStatus {
    Running,
    /// The unit arrived (also at the reachable point nearest to the destination, or behind the crowd on it:
    /// `> MovingToLocation > Unreachable target location`, `> Occupied target location`).
    Arrived,
    /// The move gave up (`> MovingToLocation > Failure`).
    Failed,
}

impl Leg {
    /// A leg from `from`, where the unit stands, to `to`.
    pub(super) fn new(from: Vec2, to: Vec2) -> Self {
        Self {
            to,
            reference: vec![from, to],
            planned: false,
            marching: false,
            engaged: None,
            ignored: None,
        }
    }
}

/// The distance from `p` to the nearest point of the route `route`, a chain of legs.
fn distance_to_route(p: Vec2, route: &[Vec2]) -> f32 {
    let to_leg = |a: Vec2, b: Vec2| {
        let along = b - a;
        let len2 = along.length_squared();
        let t = if len2 > 0.0 {
            ((p - a).dot(along) / len2).clamp(0.0, 1.0)
        } else {
            0.0
        };
        p.distance(a + along * t)
    };
    match route {
        [] => 0.0,
        [only] => p.distance(*only),
        _ => route
            .windows(2)
            .map(|leg| to_leg(leg[0], leg[1]))
            .fold(f32::MAX, f32::min),
    }
}

/// `> AttackMovingToLocation`: one tick of a leg. `group` is the order group of a leg ordered as one (for the
/// shared path field and for yielding, `docs/movement.md > 5.5`, `> 6.6`).
///
/// 1. MovingToLocation towards the leg's destination.
/// 2. While moving, the unit scans for ValidTarget enemies within SightRange ([`World::pick`]) and switches to
///    AttackingObject on the one it acquires ([`close_in`]).
/// 3. While engaged: when the unit's distance from the PathReference exceeds AttackMoveLeashDistance, or the target
///    is destroyed (or no longer a ValidTarget: out of sight, say), it disengages and resumes MovingToLocation from
///    where it is, with a new path.
/// 4. Arrived: the leg is over. `> Failure`: as MovingToLocation.
///
/// Placeholder readings, where the design is silent:
/// - **PathReference** is the route first planned for the leg and stays that for the whole leg, also after the
///   unit resumed from somewhere else. Recomputed on every resume it would start where the unit stands, the leash
///   would measure nothing, and a unit could be drawn away for any distance, 6 grid units at a time.
/// - **The leash holds for acquiring too**: a unit farther than the leash from its PathReference doesn't acquire.
///   Otherwise a unit that just broke off would acquire the same enemy again in the next tick.
/// - A target the unit can't get to ([`Closing::GaveUp`]) is left alone for [`GIVE_UP_RETRY_TICKS`].
pub(super) fn march(unit: &mut Unit, leg: &mut Leg, group: Option<PathGroup>, world: &World) -> LegStatus {
    let leash = gu(ATTACK_MOVE_LEASH_GU);
    if let Some(mut pursuit) = leg.engaged {
        let target = unit
            .attrs()
            .and_then(|attrs| world.get(pursuit.target).filter(|t| world.valid_target(unit, attrs, t)));
        let on_leash = distance_to_route(unit.pos, &leg.reference) <= leash;
        let keeps = match target {
            Some(target) if on_leash => match close_in(unit, &mut pursuit.chasing, target, world) {
                Closing::InRange | Closing::Approaching => true,
                Closing::GaveUp => {
                    leg.ignored = Some((pursuit.target, world.tick.saturating_add(GIVE_UP_RETRY_TICKS)));
                    false
                }
            },
            Some(_) | None => false,
        };
        if keeps {
            leg.engaged = Some(pursuit);
            return LegStatus::Running;
        }
        unit.engage(None);
        leg.engaged = None;
    }
    if !leg.marching {
        unit.mover.go_to(MoveGoal::Point(leg.to), group, world.tick);
        leg.marching = true;
    }
    match unit.mover.status {
        MoveStatus::Moving => {}
        MoveStatus::Arrived => return LegStatus::Arrived,
        MoveStatus::Failed | MoveStatus::TargetLost | MoveStatus::Idle => return LegStatus::Failed,
    }
    if !leg.planned && unit.mover.path_state == PathState::Ready {
        leg.reference.clone_from(&unit.mover.path);
        leg.planned = true;
    }
    if distance_to_route(unit.pos, &leg.reference) > leash {
        return LegStatus::Running;
    }
    leg.ignored = leg.ignored.filter(|&(_, until)| world.tick < until);
    let skip = leg.ignored.map(|(id, _)| id);
    if let Some(target) = world.pick(unit, Scan::Sight, skip, false)
        && let Some(view) = world.get(target)
    {
        unit.engage(Some(target));
        let mut pursuit = Pursuit { target, chasing: false };
        // The leg's own move ends here: the unit stops in range or goes after the target.
        close_in(unit, &mut pursuit.chasing, view, world);
        leg.engaged = Some(pursuit);
        leg.marching = false;
        // An engagement before the path came keeps the straight line as the reference.
        leg.planned = true;
    }
    LegStatus::Running
}

/// `> Patrolling`: one tick. Returns whether the Patrol is over.
///
/// The unit attack-moves ([`march`]) to and fro between the PatrolOrigin, where it was when the Patrol started
/// executing (design change 0036), and the PatrolDestination, and never completes on its own, except by `> Failed
/// legs` (design change 0037): a leg that fails counts as arrived and the unit turns around; after
/// [`PATROL_FAILED_LEGS`] failed legs in a row the Patrol ends and the unit goes idle. Unreachable or crowded
/// endpoints end a leg as arrived, and fighting on the way isn't a failure.
pub(super) fn patrol(unit: &mut Unit, state: &mut Patrol, group: PathGroup, world: &World) -> bool {
    // A leg that ends starts the next one in the same tick.
    for _ in 0..2 {
        // The way back is each unit's own: the group's path field is anchored on the destination's click.
        let group = state.outbound.then_some(group);
        match march(unit, &mut state.leg, group, world) {
            LegStatus::Running => return false,
            LegStatus::Arrived => state.failed_legs = 0,
            LegStatus::Failed => {
                state.failed_legs = state.failed_legs.saturating_add(1);
                if state.failed_legs >= PATROL_FAILED_LEGS {
                    return true;
                }
            }
        }
        state.outbound = !state.outbound;
        let to = if state.outbound {
            state.destination
        } else {
            state.origin
        };
        state.leg = Leg::new(unit.pos, to);
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_leash_is_measured_to_the_nearest_point_of_the_route() {
        let route = [Vec2::ZERO, Vec2::new(100.0, 0.0), Vec2::new(100.0, 100.0)];
        assert_eq!(distance_to_route(Vec2::new(50.0, 30.0), &route), 30.0);
        assert_eq!(distance_to_route(Vec2::new(130.0, 50.0), &route), 30.0);
        assert_eq!(
            distance_to_route(Vec2::new(-30.0, -40.0), &route),
            50.0,
            "before its start"
        );
        assert_eq!(distance_to_route(Vec2::new(100.0, 110.0), &route), 10.0, "past its end");
        assert_eq!(distance_to_route(Vec2::new(3.0, 4.0), &[Vec2::ZERO]), 5.0);
        assert_eq!(distance_to_route(Vec2::new(3.0, 4.0), &[Vec2::ZERO, Vec2::ZERO]), 5.0);
        assert_eq!(distance_to_route(Vec2::ONE, &[]), 0.0);
    }
}
