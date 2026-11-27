//! Path requests, results and the per-tick budget — `docs/movement.md > 5.5`.

use std::collections::BTreeMap;

use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::los::clamp_to_clear;
use super::{Nav, NavClass, NavCounters, NavGrid, Search, los, smooth_avoiding};
use crate::hash::{StateHash, StateHasher};
use crate::ids::ObjectId;
use crate::ruleset::rules;
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

/// [`find_path`] without avoid tiles that gives up once the cheapest tile path is known to cost more than `max_cost`
/// ([`Search::astar_within`]): `Ok(None)` then, and otherwise the very path [`find_path`] returns. For a caller that
/// has no use for a longer path (`orders::reach`), at a bounded cost.
pub fn find_path_within(
    grid: &NavGrid,
    search: &mut Search,
    half: Vec2,
    start: Vec2,
    goal: Vec2,
    max_cost: u32,
) -> Result<Option<Path>, PathFail> {
    let component = start_component(grid, start).ok_or(PathFail::StartBlocked)?;
    let (goal_tile, goal, retargeted) = resolve_goal(grid, component, half, goal);
    let tiles = search.astar_within(grid, grid.tile_of(start), goal_tile, &[], max_cost);
    Ok(tiles.map(|path| Path {
        waypoints: smooth_avoiding(grid, half, start, &path.tiles, goal, &[]),
        retargeted,
    }))
}

/// `docs/movement.md > 5.5`: serves one tick's path requests in `(since, id)` order within `budget` node
/// expansions (normally [`crate::tuning::paths::Paths::path_budget`]).
///
/// - A job starts while the expansions of this call are below the budget, and always finishes. So the oldest
///   request is served every tick, and every request within bounded ticks.
/// - A job is one request's A\*, or a **group field**: when at least `Paths::group_field_min` pending requests share
///   a group id, class, size class and component, have no avoid tiles, and their goals can be seen from the
///   group's anchor tile (`field_members`), one Dijkstra from the anchor serves them all, each with the parent
///   chain from its own tile, smoothed, ending on its own goal.
/// - The budget counts expansions, never time, so the outcome is deterministic. `requests` may come in any
///   order; ids must be unique.
pub fn serve(nav: &Nav, search: &mut Search, requests: &[PathRequest], budget: u32) -> ServeOutput {
    serve_on(&[nav], &vec![0; requests.len()], search, requests, budget)
}

/// [`serve`] for requests that plan on different sets of grids: request `i` on `navs[on[i]]`, all within the one
/// budget and in the one `(since, id)` order. The sim plans each unit on what its owner knows
/// (`systems/controls/behaviors.md > PathKnowledge`), so a tick's requests are spread over the players' grids.
/// A group field serves only members of one set of grids. `on` has an index into `navs` for every request.
pub fn serve_on(
    navs: &[&Nav],
    on: &[usize],
    search: &mut Search,
    requests: &[PathRequest],
    budget: u32,
) -> ServeOutput {
    let tuning = &rules().tuning.paths;
    assert_eq!(on.len(), requests.len(), "one set of grids per request");
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
        let Some(grid) = navs[on[i]].grid(request.class, request.size.max(1)) else {
            done[i] = true;
            served.push(Served {
                id: request.id,
                result: Err(PathFail::NoGrid),
                via_field: false,
            });
            continue;
        };
        let field = field_members(grid, requests, on, &order, &done, i);
        if let Some((anchor, members)) = field.filter(|(_, members)| members.len() >= tuning.group_field_min) {
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
/// one. The members share a group id, their set of grids (`on`), class, size class and component and have no avoid
/// tiles, and the anchor tile's centre sees each one's goal for its box.
///
/// A member whose goal the anchor can't see plans alone. With the magic box (`docs/movement.md > 6.4`) a
/// unit's destination is its own place in the group's shape, which may lie behind an obstacle from the click, for
/// example on the near side of a wall the click is beyond. The way through the anchor would then be a detour, and
/// only that unit's own search finds the direct one.
fn field_members(
    grid: &NavGrid,
    requests: &[PathRequest],
    on: &[usize],
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
            && on[j] == on[i]
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
///
/// (G) The field is built only as far as the members stand ([`Search::field_to`]), and the members share the
/// smoothing of the chains they share ([`smooth_shared`]).
fn field_paths(
    grid: &NavGrid,
    search: &mut Search,
    requests: &[PathRequest],
    anchor: TilePos,
    members: &[FieldMember],
) -> Vec<Path> {
    let starts: Vec<TilePos> = members
        .iter()
        .map(|member| grid.tile_of(requests[member.index].start))
        .collect();
    search.field_to(grid, anchor, &starts);
    let mut legs = BTreeMap::new();
    members
        .iter()
        .zip(&starts)
        .map(|(member, &from)| {
            let request = &requests[member.index];
            let tiles = search
                .field_path(grid, from)
                .expect("the field covers the members' component")
                .tiles;
            Path {
                waypoints: smooth_shared(grid, request.half, request.start, &tiles, member.goal, &mut legs),
                retargeted: member.retargeted,
            }
        })
        .collect()
}

/// (G) [`super::smooth`] for the members of one field, with the same result. The paths of a field are chains of one
/// tree, so from a tile on, two members walk the same tiles; and the waypoint that follows a tile's centre, the
/// farthest centre of its chain that it sees, depends on that chain alone as long as the member's own goal is
/// out of sight. `legs` remembers it per tile (how many tiles ahead it lies) for boxes of `half`, so each leg is
/// searched once for the whole group instead of once per member. A member's first leg, from its exact position,
/// and its last, to its own goal, are its own. Use one `legs` per field and box size.
fn smooth_shared(
    grid: &NavGrid,
    half: Vec2,
    start: Vec2,
    tiles: &[TilePos],
    goal: Vec2,
    legs: &mut BTreeMap<(u32, u32, u32), usize>,
) -> Vec<Vec2> {
    let sees = |a: Vec2, b: Vec2| los(grid, half, a, b);
    // The candidates of `smooth`: the start, the centre of every tile, the goal.
    let last = tiles.len() + 1;
    let point = |i: usize| match i {
        0 => start,
        i if i == last => goal,
        i => grid.center(tiles[i - 1]),
    };
    let mut out = vec![start];
    let mut at = 0;
    while at < last {
        let from = point(at);
        let next = if sees(from, goal) {
            last
        } else {
            let key = (at > 0)
                .then(|| grid.index(tiles[at - 1]))
                .flatten()
                .map(|tile| (tile, half.x.to_bits(), half.y.to_bits()));
            let known = key.and_then(|key| legs.get(&key).copied());
            let ahead = known.unwrap_or_else(|| {
                let (mut visible, mut hidden) = (at + 1, last);
                while hidden - visible > 1 {
                    let mid = visible + (hidden - visible) / 2;
                    if sees(from, point(mid)) {
                        visible = mid;
                    } else {
                        hidden = mid;
                    }
                }
                visible - at
            });
            if let (Some(key), None) = (key, known) {
                legs.insert(key, ahead);
            }
            at + ahead
        };
        if next == last || point(next) != out[out.len() - 1] {
            out.push(point(next));
        }
        at = next;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::Map;
    use crate::nav::{NavRules, smooth};
    use crate::rng::Rng;

    /// (G) Sharing the legs between the members of a field changes no path: on seeded random maps every member's
    /// waypoints are those of [`smooth`].
    #[test]
    fn shared_smoothing_gives_the_waypoints_of_smooth() {
        let half = Vec2::splat(12.0);
        for seed in 1..=6 {
            let mut rng = Rng::from_seed(seed);
            let rows: Vec<String> = (0..24)
                .map(|_| (0..24).map(|_| if rng.below(100) < 18 { 'c' } else { '.' }).collect())
                .collect();
            let map = Map::from_ascii(&rows.join("\n")).unwrap();
            let grid = NavGrid::build(&map, NavClass::Ground, 1, &[], NavRules::default());
            let free = |rng: &mut Rng| loop {
                let tile = TilePos::new(rng.below(24) as i32, rng.below(24) as i32);
                if !grid.is_blocked(tile) {
                    break tile;
                }
            };
            let anchor = free(&mut rng);
            let froms: Vec<TilePos> = (0..60)
                .map(|_| free(&mut rng))
                .filter(|&from| grid.component(from) == grid.component(anchor))
                .collect();
            let mut search = Search::new();
            assert!(search.field_to(&grid, anchor, &froms));
            let mut legs = BTreeMap::new();
            let mut paths = 0;
            for from in froms {
                let tiles = search.field_path(&grid, from).unwrap().tiles;
                let start = clamp_to_clear(&grid, half, from, grid.center(from) + Vec2::new(9.0, -7.0));
                let goal = clamp_to_clear(&grid, half, anchor, grid.center(anchor) + Vec2::new(-5.0, 11.0));
                assert_eq!(
                    smooth_shared(&grid, half, start, &tiles, goal, &mut legs),
                    smooth(&grid, half, start, &tiles, goal),
                    "seed {seed}: {from:?} -> {anchor:?}"
                );
                paths += 1;
            }
            assert!(paths > 10 && !legs.is_empty(), "seed {seed}");
        }
    }
}
