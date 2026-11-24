//! Navigation: static-obstacle pathfinding for ground and underground units — `docs/movement.md > 5`.
//!
//! This module is pure and standalone: it knows the [`Map`] and tile rectangles, never `Object`, `Sim` or the tick.
//! The movement slices plug it in (`docs/movement.md > 4`: phase 5b rebuilds a [`Nav`], phase 5c calls [`serve`]).
//!
//! - [`NavGrid`] (5.2): a blocked bit and a component label per tile, one grid per `(NavClass, size class)`.
//! - [`Search`] (5.3): the A\* scratch arrays; [`Search::astar`] and the Dijkstra [`Search::field`] share one loop.
//! - [`los`] and [`smooth`] (5.4): line of sight for a box, and string-pulling of a tile path into waypoints
//!   ([`smooth_avoiding`] keeps a repath's detour around its avoid tiles).
//! - [`find_path`] and [`serve`] (5.5): one path, and a tick's requests within the expansion budget
//!   ([`serve_on`] when they plan on several sets of grids: the sim keeps one per player, for what each knows).
//! - [`reachable_point`] (6.4): where a group member goes when its own destination is out of its reach.
//!
//! Every discrete decision of a search (costs, heap keys, tie-breaks, retargeting) is integer arithmetic. Floats
//! appear only in line of sight, waypoints and [`reachable_point`]'s distances, with `+ - * /` and comparisons
//! (`docs/movement.md > 7`).
//!
//! Units never enter a grid (5.7). Unit positions map to tiles through [`NavGrid::tile_of`]. Size class 1 is
//! complete and tested; larger classes build true-clearance grids, but their goal clamping is only coarse.

mod grid;
mod los;
mod search;
mod serve;

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::map::Map;
use crate::units::{SU_PER_GU, TilePos};

pub use grid::NavGrid;
pub use los::{box_clear, los, reachable_point, segment_avoids, smooth, smooth_avoiding};
pub use search::{Search, TilePath};
pub use serve::{Path, PathFail, PathGroup, PathRequest, ServeOutput, Served, find_path, serve, serve_on};

/// `docs/movement.md > 5.5`: node expansions per tick within which the path server starts jobs (tuning const).
pub const PATH_BUDGET: u32 = 32_000;

/// `docs/movement.md > 5.5`: from this many requests of one order group on whose goals the group's anchor sees, they
/// share one Dijkstra field.
pub const GROUP_FIELD_MIN: usize = 4;

/// `docs/movement.md > 5.3`: cost of an orthogonal step.
pub const STEP_COST: u32 = 10;

/// `docs/movement.md > 5.3`: cost of a diagonal step.
pub const DIAGONAL_COST: u32 = 14;

/// Whether map resources (crystal patches, supply stations) block ground units. Default until question Q0011
/// (gap G10) is answered: yes.
pub const RESOURCES_BLOCK: bool = true;

/// Whether a Rugged tile is impassable for a base with RuggedTerrain = false. Default until question Q0011
/// (gap G11) is answered: yes.
pub const RUGGED_IMPASSABLE: bool = true;

/// Which tiles a unit may stand on (`docs/movement.md > 3`, `core/map.md > Tile`). Air units need no grid.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum NavClass {
    /// Ground bases with RuggedTerrain = false (`systems/units.md > UnitBase`).
    Ground,
    /// Ground bases with RuggedTerrain = true: they cross Rugged tiles.
    GroundRugged,
    /// Drill units: Drillable tiles (`systems/units.md > DrillUnit`).
    Underground,
}

/// `docs/movement.md > 3`: a unit's size class, `ceil(max(silhouette w, h) / 64 su)`, at least 1.
pub fn size_class(silhouette: Vec2) -> u8 {
    (silhouette.max_element() / SU_PER_GU).ceil().clamp(1.0, 255.0) as u8
}

/// A rectangle of tiles: a blocking footprint. Parts outside the map are ignored.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileRect {
    /// Top-left tile.
    pub origin: TilePos,
    pub w: u32,
    pub h: u32,
}

impl TileRect {
    pub const fn new(origin: TilePos, w: u32, h: u32) -> Self {
        Self { origin, w, h }
    }
}

/// What kind of object a tile footprint belongs to, for [`NavRules::footprint_blocks`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BlockerKind {
    /// A surface structure.
    Structure,
    /// A map resource (`core/map.md > SpaceCrystalsPatch`, `> SupplyDeliveryStation`).
    Resource,
    /// An underground expansion: surface units walk over it
    /// (`factions/syndicate/structures.md > Tunnel Expansions`).
    UndergroundExpansion,
}

/// The switches for the open terrain questions. [`NavRules::default`] holds the defaults of Q0011.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct NavRules {
    /// [`RESOURCES_BLOCK`] (Q0011, G10).
    pub resources_block: bool,
    /// [`RUGGED_IMPASSABLE`] (Q0011, G11).
    pub rugged_impassable: bool,
}

impl Default for NavRules {
    fn default() -> Self {
        Self {
            resources_block: RESOURCES_BLOCK,
            rugged_impassable: RUGGED_IMPASSABLE,
        }
    }
}

impl NavRules {
    /// Whether a footprint of this kind goes into the blocking footprints of the surface grids
    /// (`docs/movement.md > 5.2`). The caller filters its objects with this, then passes their rectangles.
    pub fn footprint_blocks(&self, kind: BlockerKind) -> bool {
        match kind {
            BlockerKind::Structure => true,
            BlockerKind::Resource => self.resources_block,
            BlockerKind::UndergroundExpansion => false,
        }
    }
}

/// Work done by searches: output for tests and the perf overlay, never part of the state or its hash
/// (`docs/movement.md > 4`).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct NavCounters {
    /// A\* searches started.
    pub searches: u32,
    /// Dijkstra fields built.
    pub fields: u32,
    /// Nodes expanded by both. The path budget counts these.
    pub expansions: u32,
}

/// The grids in use, one per `(NavClass, size class)`. Derived data: rebuilt from the map and the blocking
/// footprints, never hashed or serialized (`docs/movement.md > 3`).
#[derive(Clone, Debug, PartialEq)]
pub struct Nav {
    rules: NavRules,
    grids: Vec<NavGrid>,
}

impl Nav {
    /// Builds a grid for each `(class, size class)` key. `blockers` are the footprints that block surface units.
    pub fn build(map: &Map, blockers: &[TileRect], keys: &[(NavClass, u8)], rules: NavRules) -> Self {
        let mut keys: Vec<(NavClass, u8)> = keys.iter().map(|&(class, size)| (class, size.max(1))).collect();
        keys.sort_unstable();
        keys.dedup();
        let grids = keys
            .iter()
            .map(|&(class, size)| NavGrid::build(map, class, size, blockers, rules))
            .collect();
        Self { rules, grids }
    }

    /// `docs/movement.md > 5.7`: the full rebuild after the map or the blocking footprints changed. It keeps the
    /// keys and the rules. The caller bumps its `nav_version`.
    pub fn rebuild(&mut self, map: &Map, blockers: &[TileRect]) {
        for grid in &mut self.grids {
            *grid = NavGrid::build(map, grid.class(), grid.size(), blockers, self.rules);
        }
    }

    /// `docs/movement.md > 5.7`: the rebuild after the blocking footprints changed on a map that didn't
    /// ([`NavGrid::restamp`]): the grids [`Nav::rebuild`] makes on the map they were built from, without reading
    /// the terrain again.
    pub fn restamp(&mut self, blockers: &[TileRect]) {
        for grid in &mut self.grids {
            grid.restamp(blockers);
        }
    }

    pub fn rules(&self) -> NavRules {
        self.rules
    }

    /// The grid of a class and size class, if it was built.
    pub fn grid(&self, class: NavClass, size: u8) -> Option<&NavGrid> {
        self.grids.iter().find(|g| g.class() == class && g.size() == size)
    }

    /// Every grid, in `(class, size class)` order.
    pub fn grids(&self) -> &[NavGrid] {
        &self.grids
    }
}
