//! Travelling behaviours — `systems/controls/behaviors.md > MovingToObject` (with Follow), `>
//! AttackMovingToLocation` and `> Patrolling`.

use glam::Vec2;

use crate::combat::{inside_min_range, within_range};
use crate::defs::AttackAttrs;
use crate::ids::ObjectId;
use crate::mover::{MoveGoal, MoveStatus, Mover, PathState};
use crate::nav::PathGroup;
use crate::tuning::orders::{PATROL_FAILED_LEGS, PROXIMITY_SU};

use super::engage::{Closing, close_in};
use super::reach::{self, Reach, Verdict};
use super::state::{Approach, Judged, Leg, Patrol, Pursuit};
use super::world::{Candidate, Known, Scan, Unit, View, World};

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
/// - the target **goes out of the owner's vision** (design change 0200, answering question Q0051, "same as
///   AttackingObject > Target lost from vision"; it replaces design change 0034's walk to the last seen position):
///   the order completes at once and the unit "stops where it is", or goes on to its next queued order. The fog
///   delay of the tiles that just left sight counts as seen (`World::sees`). A unit out of sight as the order
///   begins (queued behind others) ends it at once the same way. The player's own units are always seen.
///
/// **Itself** (design change 0168, `systems/controls/commands.md > Move`: "Following itself = Stop"): a unit whose
/// Move names itself, one of the units a right-click on it ordered, stops: "its order ends at once, as if it had
/// arrived". Queued, the step ends as soon as it starts, and the next one follows. The others follow it.
pub(super) fn approach(unit: &mut Unit, id: ObjectId, state: &mut Approach, world: &World) -> bool {
    if id == unit.id {
        unit.halt();
        return true;
    }
    let target = match world.known(unit.owner, id) {
        Known::Seen(target) => target,
        Known::Remembered { position } => {
            state.moving = false;
            return walk_to_remembered(unit, &mut state.remembered, position, world);
        }
        // A followed unit out of the owner's sight (design change 0200), or an object it knows nothing of.
        Known::Gone => {
            state.moving = false;
            unit.halt();
            return true;
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
            walking: Vec::new(),
            resumed: false,
            rejected: Vec::new(),
            chase: None,
        }
    }

    /// Design change 0180, PathReference: what the path of the leg's own move, `mover`'s, makes of the reference
    /// ([`Leg::reference`]). The first path of the leg is the reference ("the path planned when the AttackMove
    /// started"); the path planned when the unit resumes after a fight is only the walking path ([`Leg::walking`]):
    /// "the walking path is re-planned, the leash line isn't"; and a path that then takes the place of the walking
    /// path, while the move goes on, is a re-route: the move found its way blocked (stuck behind units, or an obstacle
    /// its owner didn't know of, `paths`), and the new path, the detour, is the reference from now on ("replaced on a
    /// blockage re-route"). So is the path a debug teleport makes the unit plan again. A new reference undoes the
    /// rejections judged by the old one ([`Leg::set_reference`], design change 0230).
    fn follow_path(&mut self, mover: &Mover) {
        if mover.path_state != PathState::Ready {
            return;
        }
        let path = &mover.path;
        if !self.planned {
            self.set_reference(path);
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
                self.set_reference(path);
                self.walking.clear();
            }
        }
    }
}

/// `> AttackMovingToLocation`: one tick of a leg. `group` is the order group of a leg ordered as one (for the
/// shared path field and for yielding, `docs/movement.md > 5.5`, `docs/movement.md > 6.6`).
///
/// 1. MovingToLocation towards the leg's destination.
/// 2. While moving, the unit scans for ValidTarget enemies within SightRange ([`World::pick`]) and switches to
///    AttackingObject on the one it acquires ([`close_in`]), **but only if it is a chase the unit can finish**
///    (design change 0230, the reach rule, [`acquire`]): an enemy within weapon range is always taken; one out of
///    range only if the path to a firing position stays within the leash of the PathReference and keeps the target
///    within the unit's SightRange ([`reach::judge`]). Otherwise the unit doesn't take it and walks on.
/// 3. While engaged ([`pursue`]; design change 0180, "the leash limits chasing only"): a target within weapon range
///    is always fought. Out of range, the unit goes after it only within AttackMoveLeashDistance of the
///    PathReference ([`Leg::reference`], [`Leg::follow_path`]); a target it could only reach beyond the leash is
///    let go. Let go, destroyed or no longer a ValidTarget (out of sight: its AttackingObject completes, design
///    change 0200): the unit disengages and resumes MovingToLocation from where it is, with a new walking path; the
///    PathReference stays.
///    **There is no memory of the targets it let go** (0230): each "is judged again like any other enemy by the
///    reach rule in step 2, so it is not re-taken while the chase still can't be finished".
/// 4. Arrived: the leg is over. `> Failure`: as MovingToLocation.
///
/// Further:
/// - **A chase the unit gives up** (design change 0235, answering question Q0073: "A chase given up by the Failure
///   rule puts the enemy in the reach rule's rejected set"; [`Closing::GaveUp`]: its move failed by `>
///   MovingToLocation > Failure`, "the way is blocked by own units, which the path check can't see", design change
///   0223; or it stands as near as it gets). The enemy "is judged again only on 0230's change triggers"
///   (`orders::reach`), and fought as always once within weapon range. It takes the place of the 32-tick retry an
///   idle unit has (design change 0181), by which units behind their own side in a corridor took the target again
///   and again and never finished their leg.
/// - **A target inside the unit's MinRange** is let go at once, too close to hit (design change 0182): the scan
///   picks a hittable one instead.
pub(super) fn march(
    unit: &mut Unit,
    leg: &mut Leg,
    group: Option<PathGroup>,
    world: &World,
    reach: &mut Reach,
) -> LegStatus {
    if let Some(mut pursuit) = leg.engaged {
        if pursue(unit, leg, &mut pursuit, world, reach) {
            leg.engaged = Some(pursuit);
            return LegStatus::Running;
        }
        unit.engage(None);
        leg.engaged = None;
        leg.chase = None;
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
    let Some(attrs) = unit.attrs() else {
        return LegStatus::Running;
    };
    if let Some((view, chase)) = acquire(unit, attrs, leg, world, reach) {
        unit.engage(Some(view.id));
        let mut pursuit = Pursuit {
            target: view.id,
            chasing: false,
        };
        // The leg's own move ends here: the unit stops in range or goes after the target.
        close_in(unit, &mut pursuit.chasing, view, world);
        leg.engaged = Some(pursuit);
        leg.chase = chase;
        leg.marching = false;
        // An engagement before the path came keeps the straight line as the reference.
        leg.planned = true;
    }
    LegStatus::Running
}

/// `> AttackMovingToLocation` 2 (design change 0230): the enemy the unit takes this tick, if any, and the judgement
/// it goes after it by (`None` for one within its weapon range). The candidates are judged in target-priority order
/// ([`World::pick`]): the best one that is within weapon range, or whose chase the unit can finish, is taken.
/// - One whose rejection stands ([`Leg::rejects`]), or that the PathReference rules out by itself
///   ([`reach::near_route`], or the unit standing beyond the leash), is passed over at no cost.
/// - Any other out of range is judged by a path ([`reach::judge`]), and rejected if the unit can't finish the chase:
///   a rejection its own movement re-checks too, from where it stands (design change 0243, [`Judged::by_reach`]).
/// - Once the unit's paths for the tick are spent ([`Verdict::Later`]), the candidates not yet judged wait for the
///   next tick, and the best within weapon range is taken now ("an enemy within weapon range is always taken").
fn acquire<'w>(
    unit: &Unit,
    attrs: &AttackAttrs,
    leg: &mut Leg,
    world: &'w World,
    reach: &mut Reach,
) -> Option<(&'w View, Option<Judged>)> {
    let version = world.knowledge_version(unit.owner);
    let on_leash = reach::within_leash(unit.pos, &leg.reference);
    loop {
        let admit = |c: &Candidate| {
            c.in_range
                || (on_leash
                    && !leg.rejects(c.view, version, unit.pos)
                    && reach::near_route(unit, attrs, c.view, &leg.reference))
        };
        let view = world.get(world.pick(unit, Scan::Sight, false, admit)?)?;
        if in_range(unit, attrs, view, world) {
            return Some((view, None));
        }
        match reach::judge(unit, attrs, &leg.reference, view, world, reach) {
            Verdict::Reachable => return Some((view, Some(Judged::of(view, version)))),
            Verdict::Unreachable => reject(unit, attrs, leg, Judged::by_reach(view, version, unit.pos), world),
            Verdict::Later => {
                let id = world.pick(unit, Scan::Sight, false, |c| c.in_range)?;
                return Some((world.get(id)?, None));
            }
        }
    }
}

/// Whether the unit's attack reaches `target` from where the unit stands (`combat::within_range`).
fn in_range(unit: &Unit, attrs: &AttackAttrs, target: &View, world: &World) -> bool {
    let gap = unit.shape().gap(&target.shape);
    within_range(attrs, gap, world.elevation(unit), target.elevation)
}

/// The unit can't finish a chase of the enemy of `judged`: the rejection is kept ([`Leg::reject`]).
fn reject(unit: &Unit, attrs: &AttackAttrs, leg: &mut Leg, judged: Judged, world: &World) {
    leg.reject(judged, |judged| {
        world
            .get(judged.target)
            .is_some_and(|t| world.valid_target(unit, attrs, t) && world.within(unit, attrs, Scan::Sight, t))
    });
}

/// `> AttackMovingToLocation` 3 (design changes 0180 and 0230): one tick of the leg's fight with the target of
/// `pursuit`. Returns whether the unit keeps it ([`march`]).
/// - Within weapon range it is always fought.
/// - Out of range, the unit goes after it while it stands within the leash itself and the chase is one it can
///   finish: by the judgement it took the target by ([`Leg::chase`]) while that stands, else by a new one (the
///   target moved on, the owner's knowledge of the obstacles changed, or the unit had only fought it within range).
///   A chase it can't finish is a rejection, as in step 2, and so is one that took the unit beyond the leash: both
///   by the reach rule, so its own movement re-checks them (design change 0243). One it gives up ([`march`]) is a
///   rejection on 0230's triggers alone (design change 0235; 0243: "Failure-rule and fog rejections keep 0230's
///   triggers").
/// - A target that is no longer a ValidTarget (out of the owner's sight, design change 0200) is let go. If the unit
///   was going after it, that chase was one it couldn't finish without losing sight of the target, whatever its
///   judgement had made of the path (the judgement measures sight as the scan does, edge to edge, which is a little
///   farther than the unit's eyes see tiles): the judgement becomes a rejection, where the target stood when it was
///   judged (never where it is now, out of sight), on 0230's triggers alone. Seen again, it is judged like any other
///   enemy once it has moved.
fn pursue(unit: &mut Unit, leg: &mut Leg, pursuit: &mut Pursuit, world: &World, reach: &mut Reach) -> bool {
    let id = pursuit.target;
    // Its Riot Shield is up (it can't attack now), or the target is destroyed.
    let (Some(attrs), Some(target)) = (unit.attrs(), world.get(id)) else {
        return false;
    };
    if !world.valid_target(unit, attrs, target) {
        if let Some(judged) = leg.chase.filter(|judged| judged.target == id) {
            reject(unit, attrs, leg, judged, world);
        }
        return false;
    }
    let gap = unit.shape().gap(&target.shape);
    if inside_min_range(attrs, gap) {
        return false;
    }
    let version = world.knowledge_version(unit.owner);
    if !within_range(attrs, gap, world.elevation(unit), target.elevation) {
        if !leg.chase.is_some_and(|judged| judged.stands(target, version, unit.pos)) {
            match reach::judge(unit, attrs, &leg.reference, target, world, reach) {
                Verdict::Reachable => leg.chase = Some(Judged::of(target, version)),
                Verdict::Unreachable => {
                    reject(unit, attrs, leg, Judged::by_reach(target, version, unit.pos), world);
                    return false;
                }
                // No path to judge by this tick: the chase goes on as it was.
                Verdict::Later => {}
            }
        } else if !reach::within_leash(unit.pos, &leg.reference) {
            // The chase took it beyond the leash after all (pushed aside, or after a target that crept on).
            reject(unit, attrs, leg, Judged::by_reach(target, version, unit.pos), world);
            return false;
        }
    }
    match close_in(unit, &mut pursuit.chasing, target, world) {
        Closing::InRange | Closing::Approaching => true,
        // The Failure rule, or as near as the unit gets (design change 0235): 0230's triggers alone.
        Closing::GaveUp => {
            reject(unit, attrs, leg, Judged::of(target, version), world);
            false
        }
    }
}

/// `> Patrolling`: one tick. Returns whether the Patrol is over.
///
/// The unit attack-moves ([`march`]) to and fro between the PatrolOrigin, where it was when the Patrol started
/// executing (design change 0036), and the PatrolDestination, and never completes on its own, except by `> Failed
/// legs` (design change 0037): a leg that fails counts as arrived and the unit turns around; after
/// [`PATROL_FAILED_LEGS`] failed legs in a row the Patrol ends and the unit goes idle. Unreachable or crowded
/// endpoints end a leg as arrived, and fighting on the way isn't a failure. Each leg takes its targets by
/// AttackMove's reach rule (design change 0230: "and Patrol, which uses AttackMove's scanning"), and starts with no
/// rejections of the leg before.
pub(super) fn patrol(unit: &mut Unit, state: &mut Patrol, group: PathGroup, world: &World, reach: &mut Reach) -> bool {
    // A leg that ends starts the next one in the same tick.
    for _ in 0..2 {
        // The way back is each unit's own: the group's path field is anchored on the destination's click.
        let group = state.outbound.then_some(group);
        match march(unit, &mut state.leg, group, world, reach) {
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
