//! Path requests, results and the per-tick budget — `docs/movement.md > 5.5`.

use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::los::clamp_to_clear;
use super::{GROUP_FIELD_MIN, Nav, NavClass, NavCounters, NavGrid, Search, los, smooth, smooth_avoiding};
use crate::hash::{StateHash, StateHasher};
use crate::ids::ObjectId;
use crate::units::{Tick, TilePos};

/// A unit's wish for a path: what `PathState::Wanted { since }` on its `Mover` turns into in phase 5c
/// (`docs/movement.md > 3`, `> 4`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PathRequest {
    /// The requesting unit. Requests are served in `(since, id)` order.
    pub id: ObjectId,
    /// The tick the unit started to want the path.
    pub since: Tick,
    pub class: NavClass,
    /// The unit's size class ([`super::size_class`]).
    pub size: u8,
    /// Half the unit's silhouette, its collision box.
    pub half: Vec2,
    /// The unit's position.
    pub start: Vec2,
    /// Where it wants to go: for a group order its own destination (`docs/movement.md > 6.4`).
    pub goal: Vec2,
    /// The order group, if the order went to several units.
    pub group: Option<PathGroup>,
    /// Tiles a repath treats as blocked (`docs/movement.md > 6.5`). Such a request never shares a field.
    pub avoid: Vec<TilePos>,
}

/// The units of one command (`docs/movement.md > 3`, `OrderGroup`).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PathGroup {
    /// The group's id: requests with the same id belong together.
    pub id: u32,
    /// The point the group was sent to (the click). Its tile, retargeted into the members' component, is the
    /// anchor of the group's field; the members' own destinations lie around it (`docs/movement.md > 6.4`).
    pub anchor: Vec2,
}

/// A served path. It is unit state (`docs/movement.md > 3`: "paths are state, not cache"), so it serializes,
/// compares and hashes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Path {
    /// The exact start, tile centres, then the final point: at least two points. Consecutive waypoints have
    /// line of sight for the unit's box.
    pub waypoints: Vec<Vec2>,
    /// The goal was unreachable (blocked, or in another component), so the path ends on the nearest tile the
    /// unit can reach (`docs/movement.md > 5.3`). Otherwise it ends on the goal, or as near to it within the
    /// goal's tile as the unit's box fits.
    pub retargeted: bool,
}

impl StateHash for Path {
    fn hash(&self, h: &mut StateHasher) {
        let Path { waypoints, retargeted } = self;
        waypoints.hash(h);
        retargeted.hash(h);
    }
}

/// Why a request got no path.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PathFail {
    /// No grid was built for the request's class and size class.
    NoGrid,
    /// The unit isn't on a passable tile (or its position is off the map or not finite).
    StartBlocked,
    /// The avoid tiles cut every route (`docs/movement.md > 5.3`: the only search that can fail).
    Avoided,
}

/// One request's outcome.
#[derive(Clone, Debug, PartialEq)]
pub struct Served {
    pub id: ObjectId,
    pub result: Result<Path, PathFail>,
    /// The path came out of a shared group field.
    pub via_field: bool,
}

/// What [`serve`] did. Requests missing from `served` wait for the next tick.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ServeOutput {
    /// In the order served.
    pub served: Vec<Served>,
    /// The work of this call. Outside the state and its hash.
    pub counters: NavCounters,
}

/// A goal the unit can stand on: its tile, the point, and whether it was retargeted.
fn resolve_goal(grid: &NavGrid, component: u32, half: Vec2, goal: Vec2) -> (TilePos, Vec2, bool) {
    let wanted = if goal.is_finite() {
        grid.tile_of(goal)
    } else {
        TilePos::new(0, 0)
    };
    if grid.component(wanted) == Some(component) {
        return (wanted, clamp_to_clear(grid, half, wanted, goal), false);
    }
    let tile = grid
        .nearest_in_component(component, wanted)
        .expect("the component exists");
    (tile, grid.center(tile), true)
}

/// The component a unit at `start` is in.
fn start_component(grid: &NavGrid, start: Vec2) -> Option<u32> {
    start.is_finite().then(|| grid.component(grid.tile_of(start))).flatten()
}

/// One unit's path from `start` to `goal`: retarget an unreachable goal, A\*, then smooth
/// (`docs/movement.md > 5.3`, `> 5.4`). An unreachable goal costs no expansion more than a reachable one.
pub fn find_path(
    grid: &NavGrid,
    search: &mut Search,
    half: Vec2,
    start: Vec2,
    goal: Vec2,
    avoid: &[TilePos],
) -> Result<Path, PathFail> {
    let component = start_component(grid, start).ok_or(PathFail::StartBlocked)?;
    let (goal_tile, goal, retargeted) = resolve_goal(grid, component, half, goal);
    let tiles = search
        .astar(grid, grid.tile_of(start), goal_tile, avoid)
        .ok_or(PathFail::Avoided)?
        .tiles;
    Ok(Path {
        waypoints: smooth_avoiding(grid, half, start, &tiles, goal, avoid),
        retargeted,
    })
}

/// `docs/movement.md > 5.5`: serves one tick's path requests in `(since, id)` order within `budget` node
/// expansions (normally [`super::PATH_BUDGET`]).
///
/// - A job starts while the expansions of this call are below the budget, and always finishes. So the oldest
///   request is served every tick, and every request within bounded ticks.
/// - A job is one request's A\*, or a **group field**: when at least [`GROUP_FIELD_MIN`] pending requests share
///   a group id, class, size class and component, have no avoid tiles, and their goals can be seen from the
///   group's anchor tile (`field_members`), one Dijkstra from the anchor serves them all, each with the parent
///   chain from its own tile, smoothed, ending on its own goal.
/// - The budget counts expansions, never time, so the outcome is deterministic. `requests` may come in any
///   order; ids must be unique.
pub fn serve(nav: &Nav, search: &mut Search, requests: &[PathRequest], budget: u32) -> ServeOutput {
    let before = search.counters();
    let spent = |search: &Search| search.counters().expansions - before.expansions;
    let mut order: Vec<usize> = (0..requests.len()).collect();
    order.sort_by_key(|&i| (requests[i].since, requests[i].id));
    let mut done = vec![false; requests.len()];
    let mut served = Vec::new();
    for &i in &order {
        if done[i] {
            continue;
        }
        if spent(search) >= budget {
            break;
        }
        let request = &requests[i];
        let Some(grid) = nav.grid(request.class, request.size.max(1)) else {
            done[i] = true;
            served.push(Served {
                id: request.id,
                result: Err(PathFail::NoGrid),
                via_field: false,
            });
            continue;
        };
        let field = field_members(grid, requests, &order, &done, i);
        if let Some((anchor, members)) = field.filter(|(_, members)| members.len() >= GROUP_FIELD_MIN) {
            for (member, path) in members
                .iter()
                .zip(field_paths(grid, search, requests, anchor, &members))
            {
                done[member.index] = true;
                served.push(Served {
                    id: requests[member.index].id,
                    result: Ok(path),
                    via_field: true,
                });
            }
        } else {
            done[i] = true;
            served.push(Served {
                id: request.id,
                result: find_path(grid, search, request.half, request.start, request.goal, &request.avoid),
                via_field: false,
            });
        }
    }
    let after = search.counters();
    ServeOutput {
        served,
        counters: NavCounters {
            searches: after.searches - before.searches,
            fields: after.fields - before.fields,
            expansions: after.expansions - before.expansions,
        },
    }
}

/// A request that shares its group's field, with its goal as the field's grid resolves it.
struct FieldMember {
    /// Its index in the requests.
    index: usize,
    goal: Vec2,
    retargeted: bool,
}

/// The field request `i` would share (`docs/movement.md > 5.5`): its anchor tile, and the pending requests that
/// would share it, `i` first and the others in serving order, each with its goal resolved. `None` if `i` can't use
/// one. The members share a group id, class, size class and component and have no avoid tiles, and the anchor
/// tile's centre sees each one's goal for its box.
///
/// A member whose goal the anchor can't see plans alone. With the magic box (`docs/movement.md > 6.4`) a
/// unit's destination is its own place in the group's shape, which may lie behind an obstacle from the click, for
/// example on the near side of a wall the click is beyond. The way through the anchor would then be a detour, and
/// only that unit's own search finds the direct one.
fn field_members(
    grid: &NavGrid,
    requests: &[PathRequest],
    order: &[usize],
    done: &[bool],
    i: usize,
) -> Option<(TilePos, Vec<FieldMember>)> {
    let request = &requests[i];
    let (Some(group), true) = (request.group, request.avoid.is_empty()) else {
        return None;
    };
    let component = start_component(grid, request.start)?;
    let anchor_tile = if group.anchor.is_finite() {
        grid.tile_of(group.anchor)
    } else {
        TilePos::new(0, 0)
    };
    let anchor = grid
        .nearest_in_component(component, anchor_tile)
        .expect("the component exists");
    let member = |index: usize| {
        let request = &requests[index];
        let (goal_tile, goal, retargeted) = resolve_goal(grid, component, request.half, request.goal);
        let seen = goal_tile == anchor || los(grid, request.half, grid.center(anchor), goal);
        seen.then_some(FieldMember {
            index,
            goal,
            retargeted,
        })
    };
    // `i` is the first pending request in serving order, so it heads the list.
    let mut members = vec![member(i)?];
    let others = order.iter().copied().filter(|&j| {
        let other = &requests[j];
        j != i
            && !done[j]
            && other.group.is_some_and(|g| g.id == group.id)
            && other.avoid.is_empty()
            && (other.class, other.size.max(1)) == (grid.class(), grid.size())
            && start_component(grid, other.start) == Some(component)
    });
    members.extend(others.filter_map(member));
    Some((anchor, members))
}

/// One field from the group's `anchor`, then a path per member (`docs/movement.md > 5.5`): the parent chain from
/// its own tile to the anchor tile, whose centre sees its goal, smoothed. The members share a component.
fn field_paths(
    grid: &NavGrid,
    search: &mut Search,
    requests: &[PathRequest],
    anchor: TilePos,
    members: &[FieldMember],
) -> Vec<Path> {
    search.field(grid, anchor);
    members
        .iter()
        .map(|member| {
            let request = &requests[member.index];
            let tiles = search
                .field_path(grid, grid.tile_of(request.start))
                .expect("the field covers the members' component")
                .tiles;
            Path {
                waypoints: smooth(grid, request.half, request.start, &tiles, member.goal),
                retargeted: member.retargeted,
            }
        })
        .collect()
}
