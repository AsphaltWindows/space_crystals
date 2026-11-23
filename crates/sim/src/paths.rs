//! The sim's navigation state and path requests — `docs/movement.md > 4` (phases 5b and 5c), `> 5.5`, `> 5.7`.
//!
//! [`NavState`] holds the `nav` grids built from the map and the blocking footprints. It is derived data: never
//! hashed, serialized or compared, rebuilt when a `Sim` is created or loaded. Phase 5b ([`sync`]) rebuilds it when
//! the blocking footprints change and bumps the sim's `nav_version`. Phase 5c ([`serve`]) turns the movers'
//! `PathState::Wanted` into paths through `nav::serve`, within `nav::PATH_BUDGET`.
//!
//! Invariant: between ticks the grids match the objects, so a loaded save plans exactly like the running game. A
//! phase after 5b that adds or removes a blocking footprint must call [`sync`] again (`Sim::step` checks this in
//! debug builds).

use std::collections::BTreeMap;
use std::fmt;

use glam::Vec2;

use crate::defs::{Category, ObjectKind};
use crate::events::SimEvent;
use crate::ids::ObjectId;
use crate::map::Map;
use crate::mover::{MoveGoal, MoveStatus, PathState};
use crate::nav::{self, BlockerKind, Nav, NavClass, NavCounters, NavRules, PathRequest, Search, TileRect};
use crate::object::{Object, Placement};
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
        }
    }

    pub(crate) fn nav(&self) -> &Nav {
        &self.nav
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

/// Phase 5c (`docs/movement.md > 4`, `> 5.5`): serve the units' path requests in `(since, id)` order within
/// `nav::PATH_BUDGET`, units of one order group sharing a field. Units before `first_new` only. A unit whose path
/// was planned before the obstacles changed (`nav_version`) plans again. A unit that can't get a path fails its
/// move (`SimEvent::MoveFailed`).
pub(crate) fn serve(
    state: &mut NavState,
    objects: &mut BTreeMap<ObjectId, Object>,
    first_new: ObjectId,
    nav_version: u32,
    tick: Tick,
    events: &mut Vec<SimEvent>,
) {
    // TODO(slice D): check line of sight along the remaining path on the unit's stagger tick, and repath only if a
    // segment is now blocked (`docs/movement.md > 5.6`).
    for o in objects.range_mut(..first_new).map(|(_, o)| o) {
        if let Some(mover) = &mut o.mover
            && mover.path_state == PathState::Ready
            && mover.nav_version != nav_version
        {
            mover.repath(tick);
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
            // `docs/movement.md > 5.3`: no path at all (the unit stands on a blocked tile, or its avoid tiles cut
            // every route). `systems/controls/behaviors.md > MovingToLocation > Failure`.
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
