//! Travelling behaviours — `systems/controls/behaviors.md > MovingToObject` (with Follow), `>
//! AttackMovingToLocation` and `> Patrolling`.

use glam::Vec2;

use crate::combat::{inside_min_range, max_range, within_range};
use crate::defs::AttackAttrs;
use crate::ids::ObjectId;
use crate::mover::{MoveGoal, MoveStatus, Mover, PathState};
use crate::nav::PathGroup;
use crate::units::gu;

use super::engage::{Closing, close_in};
use super::state::{Approach, Dropped, Leg, Patrol, Pursuit};
use super::world::{Candidate, Known, Scan, Unit, View, World};
use super::{ATTACK_MOVE_LEASH_GU, GIVE_UP_RETRY_TICKS, PATROL_FAILED_LEGS, PROXIMITY_SU};

/// The most targets one leg lets go of at a time ([`Leg::dropped`]); beyond it, the oldest goes. Technical: enough for
/// every enemy a unit sees at once in practice.
pub(super) const MAX_DROPPED: usize = 8;

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
            walking: Vec::new(),
            resumed: false,
            dropped: Vec::new(),
        }
    }

    /// Whether the unit leaves `id` alone for now: it gave it up ([`Leg::ignored`]) or let go of it
    /// ([`Leg::dropped`]).
    fn leaves_alone(&self, id: ObjectId) -> bool {
        self.ignored.is_some_and(|(ignored, _)| ignored == id) || self.dropped.iter().any(|d| d.target == id)
    }

    /// The unit lets go of `target`, which still lives ([`march`]); `seen`: the unit's scan reaches it now.
    fn drop_target(&mut self, target: ObjectId, seen: bool) {
        self.dropped.retain(|d| d.target != target);
        if self.dropped.len() >= MAX_DROPPED {
            self.dropped.remove(0);
        }
        self.dropped.push(Dropped { target, seen });
    }

    /// The tick's look at the targets the unit let go of ([`Leg::dropped`]): one its scan reaches has been seen;
    /// one it has seen since, and doesn't reach any more, is forgotten: the unit may take it again if it comes back.
    fn look_at_dropped(&mut self, unit: &Unit, attrs: &AttackAttrs, world: &World) {
        self.dropped.retain_mut(|dropped| {
            let reached = world
                .get(dropped.target)
                .is_some_and(|view| scan_reaches(unit, attrs, view, world));
            if reached {
                dropped.seen = true;
            }
            reached || !dropped.seen
        });
    }

    /// Design change 0180, PathReference: what the path of the leg's own move, `mover`'s, makes of the reference
    /// ([`Leg::reference`]). The first path of the leg is the reference ("the path planned when the AttackMove
    /// started"); the path planned when the unit resumes after a fight is only the walking path ([`Leg::walking`]):
    /// "the walking path is re-planned, the leash line isn't"; and a path that then takes the place of the walking
    /// path, while the move goes on, is a re-route: the move found its way blocked (stuck behind units, or an obstacle
    /// its owner didn't know of, `paths`), and the new path, the detour, is the reference from now on ("replaced on a
    /// blockage re-route"). So is the path a debug teleport makes the unit plan again.
    fn follow_path(&mut self, mover: &Mover) {
        if mover.path_state != PathState::Ready {
            return;
        }
        let path = &mover.path;
        if !self.planned {
            self.reference.clone_from(path);
            self.planned = true;
        } else if self.resumed {
            self.walking.clone_from(path);
            self.resumed = false;
        } else {
            let walking = if self.walking.is_empty() {
                &self.reference
            } else {
                &self.walking
            };
            if path != walking {
                self.reference.clone_from(path);
                self.walking.clear();
            }
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
///    AttackingObject on the one it acquires ([`close_in`]): one its weapon reaches from where it stands, wherever
///    that is, or one it may go after within the leash ([`leashed`]).
/// 3. While engaged ([`pursue`]; design change 0180, "the leash limits chasing only"): a target within weapon range
///    is always fought. Out of range, the unit goes after it only within AttackMoveLeashDistance of the
///    PathReference ([`Leg::reference`], [`Leg::follow_path`]); a target it could only reach beyond the leash is
///    dropped. Dropped, destroyed or no longer a ValidTarget (out of sight, say): the unit disengages and resumes
///    MovingToLocation from where it is, with a new walking path; the PathReference stays.
/// 4. Arrived: the leg is over. `> Failure`: as MovingToLocation.
///
/// Placeholder readings, where the design is silent:
/// - **Letting go until the unit has passed**: a target the unit let go of while it still lived, out of reach
///   within the leash or lost from sight on the way, is left alone ([`Leg::dropped`]) until it has left the unit's
///   scan after the scan has reached it again ([`Leg::look_at_dropped`]): then the unit may take it again if it
///   comes back. Meanwhile it is fought only while within weapon range, and never chased. Otherwise a unit whose way
///   to an enemy at the edge of its sight leads out of that sight would let go, walk on, see it again, take it again
///   and turn back, for good (the soak's seed 1); and taken again after a fixed while, it would be when the unit had
///   walked on less than the stretch of its route from which it sees the enemy. One the unit can't get to
///   ([`Closing::GaveUp`]) is left alone for [`GIVE_UP_RETRY_TICKS`] ([`Leg::ignored`], design change 0181: "retries
///   after 32 ticks").
/// - **A target inside the unit's MinRange** is dropped at once, too close to hit (design change 0182): the scan
///   picks a hittable one instead.
pub(super) fn march(unit: &mut Unit, leg: &mut Leg, group: Option<PathGroup>, world: &World) -> LegStatus {
    if let Some(mut pursuit) = leg.engaged {
        if pursue(unit, leg, &mut pursuit, world) {
            leg.engaged = Some(pursuit);
            return LegStatus::Running;
        }
        unit.engage(None);
        leg.engaged = None;
    }
    if !leg.marching {
        unit.mover.go_to(MoveGoal::Point(leg.to), group, world.tick);
        leg.marching = true;
        // After a fight the walking path is planned again, the PathReference isn't.
        leg.resumed = leg.planned;
        leg.walking.clear();
    }
    match unit.mover.status {
        MoveStatus::Moving => {}
        MoveStatus::Arrived => return LegStatus::Arrived,
        MoveStatus::Failed | MoveStatus::TargetLost | MoveStatus::Idle => return LegStatus::Failed,
    }
    leg.follow_path(unit.mover);
    let tick = world.tick;
    leg.ignored = leg.ignored.filter(|&(_, until)| tick < until);
    let Some(attrs) = unit.attrs() else {
        return LegStatus::Running;
    };
    leg.look_at_dropped(unit, attrs, world);
    let admit =
        |c: &Candidate| c.in_range || (!leg.leaves_alone(c.view.id) && leashed(unit, attrs, leg, c.view, c.gap, world));
    if let Some(target) = world.pick(unit, Scan::Sight, false, admit)
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

/// `> AttackMovingToLocation` 3 (design change 0180): one tick of the leg's fight with the target of `pursuit`.
/// Returns whether the unit keeps it ([`march`]).
fn pursue(unit: &mut Unit, leg: &mut Leg, pursuit: &mut Pursuit, world: &World) -> bool {
    let id = pursuit.target;
    // Its Riot Shield is up (it can't attack now), or the target is destroyed.
    let (Some(attrs), Some(target)) = (unit.attrs(), world.get(id)) else {
        return false;
    };
    if !world.valid_target(unit, attrs, target) {
        // Out of sight, say.
        leg.drop_target(id, false);
        return false;
    }
    let gap = unit.shape().gap(&target.shape);
    if inside_min_range(attrs, gap) {
        return false;
    }
    if !within_range(attrs, gap, world.elevation(unit), target.elevation) {
        // One the unit leaves alone is fought only while in range.
        if leg.leaves_alone(id) {
            return false;
        }
        if !leashed(unit, attrs, leg, target, gap, world) {
            leg.drop_target(id, scan_reaches(unit, attrs, target, world));
            return false;
        }
    }
    match close_in(unit, &mut pursuit.chasing, target, world) {
        Closing::InRange | Closing::Approaching => true,
        Closing::GaveUp => {
            leg.ignored = Some((id, world.tick.saturating_add(GIVE_UP_RETRY_TICKS)));
            false
        }
    }
}

/// Design change 0180 (`> AttackMovingToLocation` 3: "The unit does not move to chase a target if that would take it
/// beyond AttackMoveLeashDistance (perpendicular distance from PathReference)"): whether the unit may go after
/// `target`, `gap` away and out of its weapon range, within the leash: it stands within it, and so does the spot it
/// would attack from. That spot is an estimate: `gap` less the Range (with the ElevationModifier) along the straight
/// line to the target. The gap shrinks no faster than the unit walks and a path may bend, so the unit may have to go
/// a little farther; once it stands beyond the leash, it lets go ([`pursue`]).
fn leashed(unit: &Unit, attrs: &AttackAttrs, leg: &Leg, target: &View, gap: f32, world: &World) -> bool {
    let leash = gu(ATTACK_MOVE_LEASH_GU);
    let reach = max_range(attrs, world.elevation(unit), target.elevation);
    let toward = (target.pos - unit.pos).normalize_or_zero();
    let spot = unit.pos + toward * (gap - reach).max(0.0);
    distance_to_route(unit.pos, &leg.reference) <= leash && distance_to_route(spot, &leg.reference) <= leash
}

/// Whether `unit`'s scan for targets reaches `view` ([`Scan::Sight`]): it is a ValidTarget within the unit's
/// SightRange.
fn scan_reaches(unit: &Unit, attrs: &AttackAttrs, view: &View, world: &World) -> bool {
    world.valid_target(unit, attrs, view) && world.within(unit, attrs, Scan::Sight, view)
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
