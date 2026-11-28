//! Phase 5c (`docs/movement.md > 4`, `docs/movement.md > 5.5`): the staggered path checks, then the tick's path
//! requests. Every unit plans on what its owner knows (`systems/controls/behaviors.md > PathKnowledge`,
//! [`super::knowledge`]).

use std::collections::BTreeMap;

use glam::Vec2;

use super::NavState;
use super::checks::{chase_check, staggered_check};
use super::knowledge::PathKnowledge;
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::map::Map;
use crate::mover::{MoveGoal, MoveStatus, PathState};
use crate::nav::{self, PathFail, PathRequest};
use crate::object::Object;
use crate::production::ExitDoor;
use crate::ruleset::rules;
use crate::units::Tick;

/// Where the object a unit heads for was when its path was planned, else `goal`.
fn planned_goal_of_direct(aimed_at: &[(ObjectId, Vec2)], id: ObjectId, goal: Vec2) -> Vec2 {
    aimed_at
        .binary_search_by_key(&id, |&(chaser, _)| chaser)
        .map_or(goal, |i| aimed_at[i].1)
}

/// `core/entities.md > Structure Type > Exit Doorway` (design change 0080): where the path of unit `o` starts if it
/// is still in the exit doorway of the building that released it (`Mover::exit`): the first of the doorway's ways
/// out (`Doorway::ways_out`: straight out, then in front of each tile of the exit side) where its box is clear on
/// the grid it plans on. (M3-D2) Out of a portal's energy doorway (design change 0145), the one way out: where its
/// walk along its heading (`Mover::exit_heading`) comes clear of the footprint (`EnergyDoorway::outside`). `None`
/// for any other unit, and for one with no way out that its owner knows of: its request then fails like any from a
/// blocked tile, and the unit waits in the doorway (`movement::doorway::leave`, `movement::doorway::emerge`).
fn doorway_start(
    state: &NavState,
    objects: &BTreeMap<ObjectId, Object>,
    o: &Object,
    class: nav::NavClass,
    size: u8,
    half: Vec2,
) -> Option<Vec2> {
    let mover = o.mover.as_ref()?;
    let building = objects.get(&mover.exit?)?;
    let clear = |way: Vec2| {
        state
            .plan_grid(o.owner, class, size, way)
            .is_some_and(|(_, grid)| nav::box_clear(grid, half, way))
    };
    match ExitDoor::of(building).filter(|door| door.holds(o.pos, half))? {
        ExitDoor::Side(door) => door.ways_out(o.pos, half).into_iter().find(|&way| clear(way)),
        ExitDoor::Centre(portal) => {
            let heading = portal.heading_of(o.pos, mover.exit_heading);
            Some(portal.outside(o.pos, heading, half)).filter(|&way| clear(way))
        }
    }
}

/// How a wanted path turned out without a search.
enum Direct {
    /// Nothing to plan: the goal is gone.
    NoGoal,
    /// The goal object no longer exists.
    TargetLost,
    /// Air units fly straight (question Q0011, gap G14).
    Straight { path: Vec<Vec2>, goal: Vec2 },
}

/// Whether `count` is a multiple of `period` (`count % period == 0`; with a `period` of 0, whether `count` is 0): by a
/// mask when `period` is a power of two, as the built-in rules' are, rather than a division for every unit.
#[inline]
fn one_in(count: Tick, period: Tick) -> bool {
    match period.is_power_of_two() {
        true => count & (period - 1) == 0,
        false => count.is_multiple_of(period),
    }
}

/// Phase 5c (`docs/movement.md > 4`, `docs/movement.md > 5.5`): run the staggered path checks ([`staggered_check`]),
/// then serve the units' path requests in `(since, id)` order within `tuning::paths::Paths::path_budget`, units of one
/// order group sharing a field. Units before `first_new` only. A unit that can't get a path fails its move
/// (`SimEvent::MoveFailed`).
///
/// **Whose grids** (`systems/controls/behaviors.md > PathKnowledge`): each unit's checks and plans run on its
/// owner's known grids (`NavState::plan_grid`), and its path notes the version of that knowledge
/// (`PlayerKnowledge::version`), so the owner seeing something new on the way sends the unit's next check along
/// its path. A unit without an owner plans on the true grids, by `nav_version`.
///
/// **Object goals** (`MoveGoal::Object`): the path is planned to the point nearest the object's centre that the
/// unit can stand on (`nav::reachable_point`: right at the wall of a structure or map resource, which blocks its
/// own tiles), and `Mover::planned_target` notes where the object was. [`chase_check`] keeps the path up to date
/// as the object moves. (M5-H1) An object is where it counts as being on `map` (`Object::map_pos`): a Glider past the
/// map's edge at the nearest edge tile.
#[expect(
    clippy::too_many_arguments,
    reason = "phase 5c's state, in the parts of the sim it reads and writes"
)]
pub(crate) fn serve(
    state: &mut NavState,
    map: &Map,
    objects: &mut BTreeMap<ObjectId, Object>,
    first_new: ObjectId,
    knowledge: &PathKnowledge,
    nav_version: u32,
    tick: Tick,
    events: &mut Vec<SimEvent>,
) {
    let tuning_movement = &rules().tuning.movement;
    let tuning_paths = &rules().tuning.paths;
    let version_of = |owner: Option<PlayerId>| {
        owner
            .and_then(|player| knowledge.version(player))
            .unwrap_or(nav_version)
    };
    let staggered = |id: ObjectId| one_in(tick.wrapping_add(id.0), tuning_movement.stagger);
    // Where the objects being chased are now, by chaser, in id order.
    let chased: Vec<(ObjectId, Vec2)> = objects
        .range(..first_new)
        .filter_map(|(_, o)| {
            let mover = o.mover.as_ref().filter(|m| m.path_state == PathState::Ready)?;
            let MoveGoal::Object { id, .. } = mover.goal else {
                return None;
            };
            Some((o.id, objects.get(&id).filter(|_| staggered(o.id))?.map_pos(map)))
        })
        .collect();
    for o in objects.range_mut(..first_new).map(|(_, o)| o) {
        let Some(mover) = &mut o.mover else {
            continue;
        };
        if mover.path_state != PathState::Ready || !staggered(o.id) {
            continue;
        }
        let def = o.kind.def();
        let grid = def.unit.as_ref().zip(def.silhouette()).and_then(|(unit, silhouette)| {
            let class = unit.nav_class()?;
            let (_, grid) = state.plan_grid(o.owner, class, nav::size_class(silhouette), o.pos)?;
            Some((grid, silhouette / 2.0))
        });
        if let Ok(i) = chased.binary_search_by_key(&o.id, |&(chaser, _)| chaser) {
            let half = def.half_extents().unwrap_or(Vec2::ZERO);
            let due = one_in(tick.wrapping_add(o.id.0), tuning_paths.chase_replan_ticks);
            chase_check(grid.map(|(grid, _)| grid), o.pos, half, mover, chased[i].1, due, tick);
            if mover.path_state != PathState::Ready {
                continue;
            }
        }
        let version = version_of(o.owner);
        match grid {
            Some((grid, half)) => staggered_check(grid, o.pos, half, mover, version, tick),
            // Air units fly straight: nothing on the ground changes their path.
            None => mover.nav_version = version,
        }
    }

    let mut requests: Vec<PathRequest> = Vec::new();
    // The set of grids each request plans on (`super::navs`).
    let mut on: Vec<usize> = Vec::new();
    let mut direct: Vec<(ObjectId, Direct)> = Vec::new();
    // For the units heading for an object: where it is now, in id order.
    let mut aimed_at: Vec<(ObjectId, Vec2)> = Vec::new();
    for o in objects.range(..first_new).map(|(_, o)| o) {
        let Some(mover) = &o.mover else {
            continue;
        };
        let PathState::Wanted { since, group } = mover.path_state else {
            continue;
        };
        let def = o.def();
        let (Some(unit), Some(silhouette)) = (&def.unit, def.silhouette()) else {
            continue;
        };
        let class = unit.nav_class();
        let size = nav::size_class(silhouette);
        // A unit still in the exit doorway of the building that released it stands on that building's tiles,
        // which no path starts from: its path starts where it comes out (`doorway_start`, design change 0080).
        let start = class
            .and_then(|class| doorway_start(state, objects, o, class, size, silhouette / 2.0))
            .unwrap_or(o.pos);
        let plan = class.and_then(|class| state.plan_grid(o.owner, class, size, start));
        let goal = match mover.goal {
            MoveGoal::None => {
                direct.push((o.id, Direct::NoGoal));
                continue;
            }
            MoveGoal::Point(p) => p,
            MoveGoal::Object { id, .. } => match objects.get(&id) {
                Some(target) => {
                    let at = target.map_pos(map);
                    aimed_at.push((o.id, at));
                    // The nearest point to it the unit can stand on: a structure's own tiles are blocked.
                    plan.and_then(|(_, grid)| {
                        let component = grid.component(grid.tile_of(start))?;
                        nav::reachable_point(grid, component, silhouette / 2.0, at)
                    })
                    .unwrap_or(at)
                }
                None => {
                    direct.push((o.id, Direct::TargetLost));
                    continue;
                }
            },
        };
        match class {
            Some(class) => {
                requests.push(PathRequest {
                    id: o.id,
                    since,
                    class,
                    size,
                    half: silhouette / 2.0,
                    start,
                    goal,
                    group,
                    avoid: mover.avoid_tiles.clone(),
                });
                // Without a grid of its class the request fails on any set (`PathFail::NoGrid`).
                on.push(plan.map_or(0, |(index, _)| index));
            }
            // (M6-GK) A HoverCraft's path ends at the nearest point at which its box lies on the map (`Map::clamp_box`),
            // as a ground unit's ends at the nearest point it can reach: its own move is held there
            // (`movement::step`; `systems/units.md > Air Collision > Map edge`: "HoverCraft stay inside the map"), so it
            // lands on that end and arrives. A Glider may fly past the edge.
            None => {
                let end = match unit.glides() {
                    true => goal,
                    false => map.clamp_box(goal, silhouette / 2.0),
                };
                direct.push((
                    o.id,
                    Direct::Straight {
                        path: vec![o.pos, end],
                        goal,
                    },
                ))
            }
        }
    }

    let navs = super::navs(&state.nav, &state.known);
    let output = nav::serve_on(&navs, &on, &mut state.search, &requests, tuning_paths.path_budget);
    state.counters = output.counters;
    // `requests` is in id order. A path to an object notes where the object was, not the point beside it.
    let planned_goal = |id: ObjectId| {
        if let Ok(i) = aimed_at.binary_search_by_key(&id, |&(chaser, _)| chaser) {
            return aimed_at[i].1;
        }
        let i = requests
            .binary_search_by_key(&id, |r| r.id)
            .expect("served ids were requested");
        requests[i].goal
    };
    for served in output.served {
        let Some(o) = objects.get_mut(&served.id) else {
            continue;
        };
        let version = version_of(o.owner);
        let Some(mover) = o.mover.as_mut() else {
            continue;
        };
        match served.result {
            Ok(path) => {
                mover.follow(path.waypoints, planned_goal(served.id), version);
                // The path of a unit in a doorway starts outside it: the unit heads for that start first.
                if mover.exit.is_some() {
                    mover.next = 0;
                }
            }
            // `docs/movement.md > 5.3`: the avoid tiles cut every route. Tiles are coarse next to units, so that
            // doesn't prove there is no way: the unit drops them and plans again in the next tick.
            Err(PathFail::Avoided) if !mover.avoid_tiles.is_empty() => {
                mover.avoid_tiles.clear();
                mover.repath(tick);
            }
            // No path at all: the unit stands on a blocked tile, or has no grid.
            // `systems/controls/command_behaviors.md > MovingToLocation > Failure`.
            Err(_) => {
                mover.finish(MoveStatus::Failed);
                events.push(SimEvent::MoveFailed { unit: served.id });
            }
        }
    }
    for (id, outcome) in direct {
        let Some(o) = objects.get_mut(&id) else {
            continue;
        };
        let version = version_of(o.owner);
        let Some(mover) = o.mover.as_mut() else {
            continue;
        };
        match outcome {
            Direct::NoGoal => mover.path_state = PathState::None,
            Direct::TargetLost => mover.finish(MoveStatus::TargetLost),
            Direct::Straight { path, goal } => mover.follow(path, planned_goal_of_direct(&aimed_at, id, goal), version),
        }
    }
}
