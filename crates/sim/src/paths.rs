//! The sim's navigation state and path requests — `docs/movement.md > 4` (phases 5b and 5c), `> 5.5`, `> 5.7`.
//!
//! [`NavState`] holds the `nav` grids built from the map and the blocking footprints. It is derived data: never
//! hashed, serialized or compared, rebuilt when a `Sim` is created or loaded. Phase 5b ([`sync`]) rebuilds it when
//! the blocking footprints change and bumps the sim's `nav_version`. Phase 5c ([`serve`]) runs the units' staggered
//! path checks (`docs/movement.md > 5.6`, `> 6.2`), then turns the movers' `PathState::Wanted` into paths through
//! `nav::serve`, within `nav::PATH_BUDGET`.
//!
//! Invariant: between ticks the grids match the objects, so a loaded save plans exactly like the running game. A
//! phase after 5b that adds or removes a blocking footprint must call [`sync`] again (`Sim::step` checks this in
//! debug builds).

use std::collections::BTreeMap;
use std::fmt;

use glam::Vec2;

use crate::defs::{Category, Domain, ObjectKind};
use crate::events::SimEvent;
use crate::ids::ObjectId;
use crate::map::Map;
use crate::movement;
use crate::mover::{HALF_STUCK_TICKS, MoveGoal, MoveStatus, Mover, PathState, STAGGER};
use crate::nav::{
    self, BlockerKind, Nav, NavClass, NavCounters, NavGrid, NavRules, PathFail, PathRequest, Search, TileRect,
};
use crate::object::{Object, Placement};
use crate::scenario::SpawnProblem;
use crate::units::{SU_PER_GU, Tick};

// `nav::ROUTE_ON_TRUE_MAP` (question Q0011, gap G12): routes are planned on the true map, so the blocking
// footprints are every blocking object's, whoever has seen it. Planning on what each player has seen needs grids
// per player, which this module doesn't build.
const _: () = assert!(nav::ROUTE_ON_TRUE_MAP, "per-player grids aren't built (question Q0011)");

/// The navigation grids and the search scratch (`docs/movement.md > 3`: derived, and scratch).
pub(crate) struct NavState {
    nav: Nav,
    /// The blocking footprints `nav` was built from, in object id order.
    blockers: Vec<TileRect>,
    search: Search,
    /// The work of the latest phase 5c: output, never state.
    counters: NavCounters,
    /// The scratch of phase 5d (`movement::step`): spatial grids, rebuilt every tick.
    moves: movement::Scratch,
}

impl NavState {
    /// No grids yet: a stand-in while a `Sim` is put together, replaced by [`NavState::build`].
    pub(crate) fn unbuilt(map: &Map) -> Self {
        let rules = NavRules::default();
        Self {
            nav: Nav::build(map, &[], &[], rules),
            blockers: Vec::new(),
            search: Search::new(),
            counters: NavCounters::default(),
            moves: movement::Scratch::default(),
        }
    }

    /// The grids of every unit type's class and size class, for `map` and the footprints of `objects`.
    pub(crate) fn build(map: &Map, objects: &BTreeMap<ObjectId, Object>) -> Self {
        let rules = NavRules::default();
        let blockers: Vec<TileRect> = blockers(objects, rules).collect();
        Self {
            nav: Nav::build(map, &blockers, &grid_keys(), rules),
            blockers,
            search: Search::new(),
            counters: NavCounters::default(),
            moves: movement::Scratch::default(),
        }
    }

    pub(crate) fn nav(&self) -> &Nav {
        &self.nav
    }

    /// The grids, and the scratch of the movement step that reads them.
    pub(crate) fn for_movement(&mut self) -> (&Nav, &mut movement::Scratch) {
        (&self.nav, &mut self.moves)
    }

    /// The work of the latest phase 5d.
    pub(crate) fn move_counters(&self) -> movement::MoveCounters {
        self.moves.counters()
    }

    pub(crate) fn counters(&self) -> NavCounters {
        self.counters
    }

    /// Whether the grids were built from the footprints the objects have now.
    pub(crate) fn matches(&self, objects: &BTreeMap<ObjectId, Object>) -> bool {
        blockers(objects, self.nav.rules()).eq(self.blockers.iter().copied())
    }
}

impl Clone for NavState {
    /// The grids and the footprints; the scratch starts empty.
    fn clone(&self) -> Self {
        Self {
            nav: self.nav.clone(),
            blockers: self.blockers.clone(),
            search: Search::new(),
            counters: self.counters,
            moves: movement::Scratch::default(),
        }
    }
}

impl PartialEq for NavState {
    /// Derived data: two sims with equal state have equal grids, so the grids never make them differ.
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl fmt::Debug for NavState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NavState")
            .field("grids", &self.nav.grids().len())
            .field("blockers", &self.blockers)
            .finish_non_exhaustive()
    }
}

/// The `(class, size class)` of every unit type that walks on a grid (`docs/movement.md > 5.2`), from the type
/// definitions, so the grids don't depend on which units exist.
fn grid_keys() -> Vec<(NavClass, u8)> {
    ObjectKind::ALL
        .iter()
        .filter_map(|kind| {
            let def = kind.def();
            let class = def.unit.as_ref()?.base_def().nav_class()?;
            Some((class, nav::size_class(def.silhouette()?)))
        })
        .collect()
}

/// The footprint of an object of `kind` placed at `placement`, if it blocks surface units
/// (`docs/movement.md > 5.2`; question Q0011, gap G10: map resources block).
pub(crate) fn blocking_footprint(kind: ObjectKind, placement: &Placement, rules: NavRules) -> Option<TileRect> {
    let blocker = match kind.def().category {
        Category::Structure => BlockerKind::Structure,
        Category::Resource => BlockerKind::Resource,
        Category::Unit => return None,
    };
    rules
        .footprint_blocks(blocker)
        .then(|| TileRect::new(placement.origin, placement.w, placement.h))
}

/// The footprints that block surface units, in object id order.
fn blockers(objects: &BTreeMap<ObjectId, Object>, rules: NavRules) -> impl Iterator<Item = TileRect> + '_ {
    objects
        .values()
        .filter_map(move |o| blocking_footprint(o.kind, o.placement.as_ref()?, rules))
}

/// Whether a box centred on `pos` with half extents `half` overlaps the tiles of `rect`. Touching is free.
pub(crate) fn box_overlaps_rect(pos: Vec2, half: Vec2, rect: TileRect) -> bool {
    let lo = rect.origin.corner();
    let hi = lo + Vec2::new(rect.w as f32, rect.h as f32) * SU_PER_GU;
    pos.x - half.x < hi.x && pos.x + half.x > lo.x && pos.y - half.y < hi.y && pos.y + half.y > lo.y
}

/// Whether two boxes overlap (`docs/movement.md > 3`): strictly, so touching is free.
pub(crate) fn boxes_overlap(a: Vec2, a_half: Vec2, b: Vec2, b_half: Vec2) -> bool {
    (a.x - b.x).abs() < a_half.x + b_half.x && (a.y - b.y).abs() < a_half.y + b_half.y
}

/// `docs/movement.md > 3` for the objects a game starts with, which skip the command gate: each object of
/// `objects`, in order, against the map and the objects before it. A unit's box must be clear of impassable
/// terrain, the map's edge and the blocking footprints, and overlap no unit of its layer; a blocking footprint must
/// not cover a ground unit. Lists the position and problem of each object that breaks this, so of two objects in
/// conflict, the later one. The same test as the sim's grids (`nav::box_clear`), on grids of the terrain alone,
/// built only for the unit classes present.
pub(crate) fn misplaced<'a>(
    map: &Map,
    objects: impl IntoIterator<Item = (ObjectKind, Vec2, Option<&'a Placement>)>,
) -> Vec<(usize, SpawnProblem)> {
    let rules = NavRules::default();
    let mut grids: BTreeMap<(NavClass, u8), NavGrid> = BTreeMap::new();
    let mut units: Vec<(Vec2, Vec2, Domain)> = Vec::new();
    let mut footprints: Vec<TileRect> = Vec::new();
    let mut problems = Vec::new();
    for (i, (kind, pos, placement)) in objects.into_iter().enumerate() {
        if let Some(rect) = placement.and_then(|p| blocking_footprint(kind, p, rules)) {
            if units
                .iter()
                .any(|&(at, half, domain)| domain == Domain::Ground && box_overlaps_rect(at, half, rect))
            {
                problems.push((i, SpawnProblem::CoversUnit));
            }
            footprints.push(rect);
            continue;
        }
        let def = kind.def();
        let (Some(unit), Some(silhouette)) = (&def.unit, def.silhouette()) else {
            continue;
        };
        let half = silhouette / 2.0;
        let base = unit.base_def();
        let on_impassable = base.nav_class().is_some_and(|class| {
            let size = nav::size_class(silhouette);
            let grid = grids
                .entry((class, size))
                .or_insert_with(|| NavGrid::build(map, class, size, &[], rules));
            !nav::box_clear(grid, half, pos)
        });
        // Surface footprints block ground units only (`nav::NavGrid::build`).
        let on_footprint =
            base.domain == Domain::Ground && footprints.iter().any(|&rect| box_overlaps_rect(pos, half, rect));
        if on_impassable || on_footprint {
            problems.push((i, SpawnProblem::Blocked));
        } else if base.domain.hard_collision()
            && units
                .iter()
                .any(|&(at, other, domain)| domain == base.domain && boxes_overlap(pos, half, at, other))
        {
            problems.push((i, SpawnProblem::OverlapsUnit));
        }
        units.push((pos, half, base.domain));
    }
    problems
}

/// Phase 5b (`docs/movement.md > 4`, `> 5.7`): if a blocking footprint appeared, moved or went away, rebuild the
/// grids in full and bump `nav_version`.
pub(crate) fn sync(state: &mut NavState, map: &Map, objects: &BTreeMap<ObjectId, Object>, nav_version: &mut u32) {
    if state.matches(objects) {
        return;
    }
    state.blockers = blockers(objects, state.nav.rules()).collect();
    state.nav.rebuild(map, &state.blockers);
    *nav_version = nav_version.wrapping_add(1);
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

/// At most this many waypoints are passed in one staggered check (a technical bound on its line-of-sight tests).
const MAX_WAYPOINTS_PER_CHECK: usize = 8;

/// The staggered path checks of one unit with a path (`docs/movement.md > 5.6`, `> 6.2`), run in the ticks where
/// `(tick + id) % STAGGER == 0`, on the grid the unit walks on:
///
/// 1. **The ground changed** (`nav_version`): line of sight along the remaining path. If every leg is still clear
///    the path stays, else the unit plans again. Not counted against the repath cap.
/// 2. **Steering target**: the unit heads for the farthest waypoint it sees, one after another from the current
///    one, so it need not touch any of them.
/// 3. **Lost sight** of the steering waypoint (pushed off the path by sidesteps) **and** no progress for
///    `HALF_STUCK_TICKS`: a repath that counts against the cap and waits for its backoff. A unit that is still
///    getting on finds its own way back by sliding, which saves the leg's few repaths for when it needs them.
fn staggered_check(grid: &NavGrid, pos: Vec2, half: Vec2, mover: &mut Mover, nav_version: u32, tick: Tick) {
    let sees = |from: Vec2, to: Vec2| nav::los(grid, half, from, to);
    let next = usize::from(mover.next);
    let Some(&target) = mover.path.get(next) else {
        return;
    };
    if mover.nav_version != nav_version {
        let ahead = &mover.path[next..];
        if sees(pos, target) && ahead.windows(2).all(|leg| sees(leg[0], leg[1])) {
            mover.nav_version = nav_version;
        } else {
            mover.repath(tick);
            return;
        }
    }
    // The steering also stays out of the unit's avoid tiles, as its path does.
    let mut at = next;
    while at + 1 < mover.path.len()
        && at - next < MAX_WAYPOINTS_PER_CHECK
        && sees(pos, mover.path[at + 1])
        && nav::segment_avoids(pos, mover.path[at + 1], &mover.avoid_tiles)
    {
        at += 1;
    }
    if at != next {
        mover.next = at as u16;
    } else if tick.saturating_sub(mover.stuck_clock()) >= HALF_STUCK_TICKS && !sees(pos, target) {
        mover.counted_repath(tick);
    }
}

/// Phase 5c (`docs/movement.md > 4`, `> 5.5`): run the staggered path checks ([`staggered_check`]), then serve the
/// units' path requests in `(since, id)` order within `nav::PATH_BUDGET`, units of one order group sharing a field.
/// Units before `first_new` only. A unit that can't get a path fails its move (`SimEvent::MoveFailed`).
pub(crate) fn serve(
    state: &mut NavState,
    objects: &mut BTreeMap<ObjectId, Object>,
    first_new: ObjectId,
    nav_version: u32,
    tick: Tick,
    events: &mut Vec<SimEvent>,
) {
    for o in objects.range_mut(..first_new).map(|(_, o)| o) {
        let Some(mover) = &mut o.mover else {
            continue;
        };
        if mover.path_state != PathState::Ready || tick.wrapping_add(o.id.0) % STAGGER != 0 {
            continue;
        }
        let def = o.kind.def();
        let grid = def.unit.as_ref().zip(def.silhouette()).and_then(|(unit, silhouette)| {
            let class = unit.base_def().nav_class()?;
            Some((state.nav.grid(class, nav::size_class(silhouette))?, silhouette / 2.0))
        });
        match grid {
            Some((grid, half)) => staggered_check(grid, o.pos, half, mover, nav_version, tick),
            // Air units fly straight: nothing on the ground changes their path.
            None => mover.nav_version = nav_version,
        }
    }

    let mut requests: Vec<PathRequest> = Vec::new();
    let mut direct: Vec<(ObjectId, Direct)> = Vec::new();
    for o in objects.range(..first_new).map(|(_, o)| o) {
        let Some(mover) = &o.mover else {
            continue;
        };
        let PathState::Wanted { since, group } = mover.path_state else {
            continue;
        };
        let goal = match mover.goal {
            MoveGoal::None => {
                direct.push((o.id, Direct::NoGoal));
                continue;
            }
            MoveGoal::Point(p) => p,
            // TODO(slice E): chasing, reach and the planned-target check (`docs/movement.md > 5.6`).
            MoveGoal::Object { id, .. } => match objects.get(&id) {
                Some(target) => target.pos,
                None => {
                    direct.push((o.id, Direct::TargetLost));
                    continue;
                }
            },
        };
        let def = o.def();
        let (Some(unit), Some(silhouette)) = (&def.unit, def.silhouette()) else {
            continue;
        };
        match unit.base_def().nav_class() {
            Some(class) => requests.push(PathRequest {
                id: o.id,
                since,
                class,
                size: nav::size_class(silhouette),
                half: silhouette / 2.0,
                start: o.pos,
                goal,
                group,
                avoid: mover.avoid_tiles.clone(),
            }),
            None => direct.push((
                o.id,
                Direct::Straight {
                    path: vec![o.pos, goal],
                    goal,
                },
            )),
        }
    }

    let output = nav::serve(&state.nav, &mut state.search, &requests, nav::PATH_BUDGET);
    state.counters = output.counters;
    // `requests` is in id order.
    let planned_goal = |id: ObjectId| {
        let i = requests
            .binary_search_by_key(&id, |r| r.id)
            .expect("served ids were requested");
        requests[i].goal
    };
    for served in output.served {
        let Some(mover) = objects.get_mut(&served.id).and_then(|o| o.mover.as_mut()) else {
            continue;
        };
        match served.result {
            Ok(path) => mover.follow(path.waypoints, planned_goal(served.id), nav_version),
            // `docs/movement.md > 5.3`: the avoid tiles cut every route. Tiles are coarse next to units, so that
            // doesn't prove there is no way: the unit drops them and plans again in the next tick.
            Err(PathFail::Avoided) if !mover.avoid_tiles.is_empty() => {
                mover.avoid_tiles.clear();
                mover.repath(tick);
            }
            // No path at all: the unit stands on a blocked tile, or has no grid.
            // `systems/controls/behaviors.md > MovingToLocation > Failure`.
            Err(_) => {
                mover.finish(MoveStatus::Failed);
                events.push(SimEvent::MoveFailed { unit: served.id });
            }
        }
    }
    for (id, outcome) in direct {
        let Some(mover) = objects.get_mut(&id).and_then(|o| o.mover.as_mut()) else {
            continue;
        };
        match outcome {
            Direct::NoGoal => mover.path_state = PathState::None,
            Direct::TargetLost => mover.finish(MoveStatus::TargetLost),
            Direct::Straight { path, goal } => mover.follow(path, goal, nav_version),
        }
    }
}
