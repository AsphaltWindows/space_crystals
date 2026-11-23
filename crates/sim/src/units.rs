//! Units of measure — `core/scale.md`.
//!
//! World positions are `Vec2` in **space units** (su). Origin = the map's top-left
//! corner, +x = east, **+y = south** (same direction as tile rows). Tile `(x, y)`
//! covers `[x*64, (x+1)*64) × [y*64, (y+1)*64)`.

use glam::Vec2;
use serde::{Deserialize, Serialize};

/// `core/scale.md > SimulationFrame > FramesPerSecond`.
pub const TICKS_PER_SECOND: u32 = 16;

/// `core/scale.md > SpaceUnit > SpaceUnitsPerGridUnit`.
pub const SU_PER_GU: f32 = 64.0;

/// A simulation tick (SimulationFrame) number. Tick 0 is the initial state;
/// each [`crate::Sim::step`] advances it by one.
pub type Tick = u32;

/// Grid units → space units.
pub fn gu(grid_units: f32) -> f32 {
    grid_units * SU_PER_GU
}

/// Seconds → ticks, rounded to the nearest whole tick.
pub fn seconds(s: f32) -> u32 {
    (s * TICKS_PER_SECOND as f32).round() as u32
}

/// Integer tile (grid cell) coordinate. Tile `(0, 0)` is the map's top-left corner.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TilePos {
    pub x: i32,
    pub y: i32,
}

impl TilePos {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// The tile containing a world position (space units).
    pub fn containing(p: Vec2) -> Self {
        Self {
            x: (p.x / SU_PER_GU).floor() as i32,
            y: (p.y / SU_PER_GU).floor() as i32,
        }
    }

    /// Top-left corner of this tile, in space units.
    pub fn corner(self) -> Vec2 {
        Vec2::new(self.x as f32 * SU_PER_GU, self.y as f32 * SU_PER_GU)
    }

    /// Center of this tile, in space units.
    pub fn center(self) -> Vec2 {
        self.corner() + Vec2::splat(SU_PER_GU / 2.0)
    }

    /// The tile's four corners (`core/map.md > Elevation`): north-west, north-east, south-west, south-east.
    pub fn corners(self) -> [CornerPos; 4] {
        let (x, y) = (self.x, self.y);
        [
            CornerPos::new(x, y),
            CornerPos::new(x + 1, y),
            CornerPos::new(x, y + 1),
            CornerPos::new(x + 1, y + 1),
        ]
    }
}

/// Integer coordinate of a tile corner, where terrain heights live (`core/map.md > Elevation`). Corner `(x, y)` is
/// the north-west corner of tile `(x, y)`, at `(64x, 64y)` su, so a `W × H` map has corners `(0..=W, 0..=H)`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CornerPos {
    pub x: i32,
    pub y: i32,
}

impl CornerPos {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// The corner's position, in space units.
    pub fn pos(self) -> Vec2 {
        Vec2::new(self.x as f32 * SU_PER_GU, self.y as f32 * SU_PER_GU)
    }
}
