//! Tiles and tile presets — `core/map.md`.

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::units::TilePos;

/// `core/map.md > Elevation - integer (0 to 16)`.
pub const MAX_ELEVATION: u8 = 16;

/// `core/map.md > TilePreset`. The texture is client-side; the sim only needs the flags.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TilePreset {
    pub name: String,
    pub buildable: bool,
    pub traversible: bool,
    pub rugged: bool,
    pub drillable: bool,
    pub recruitable: bool,
}

/// Index into a map's preset table.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TilePresetId(pub u16);

impl TilePresetId {
    // The default presets (`core/map.md > DefaultTilePresets`) always occupy these
    // indices, in this order. Custom presets are appended after them.
    pub const PLANE: Self = Self(0);
    pub const RUGGED: Self = Self(1);
    pub const CLIFF: Self = Self(2);
    pub const MOUNTAIN: Self = Self(3);
    pub const WATER: Self = Self(4);
}

/// `core/map.md > TilePlacement`: a preset at an elevation.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tile {
    pub preset: TilePresetId,
    pub elevation: u8,
}

impl Tile {
    pub const fn new(preset: TilePresetId, elevation: u8) -> Self {
        Self { preset, elevation }
    }
}

/// The tile grid. Row-major storage, `index = y * width + x`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Map {
    width: u32,
    height: u32,
    presets: Vec<TilePreset>,
    tiles: Vec<Tile>,
}

impl Map {
    /// A `width × height` map of Plane tiles at elevation 0, with the default presets.
    pub fn new(width: u32, height: u32) -> Self {
        assert!(width > 0 && height > 0, "map must not be empty");
        Self {
            width,
            height,
            presets: Self::default_presets(),
            tiles: vec![Tile::new(TilePresetId::PLANE, 0); (width * height) as usize],
        }
    }

    /// `core/map.md > DefaultTilePresets`, in [`TilePresetId`] order.
    pub fn default_presets() -> Vec<TilePreset> {
        let preset = |name: &str, buildable, traversible, rugged, drillable, recruitable| TilePreset {
            name: name.to_string(),
            buildable,
            traversible,
            rugged,
            drillable,
            recruitable,
        };
        vec![
            preset("Plane", true, true, false, true, true),
            preset("Rugged Terrain", false, true, true, true, true),
            preset("Cliff", false, false, false, true, true),
            preset("Mountain", false, false, false, false, true),
            preset("Water", false, false, false, false, false),
        ]
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn in_bounds(&self, p: TilePos) -> bool {
        p.x >= 0 && p.y >= 0 && (p.x as u32) < self.width && (p.y as u32) < self.height
    }

    /// Whether a point (space units) lies on the map.
    pub fn contains_point(&self, p: Vec2) -> bool {
        self.in_bounds(TilePos::containing(p))
    }

    fn index(&self, p: TilePos) -> Option<usize> {
        self.in_bounds(p)
            .then(|| p.y as usize * self.width as usize + p.x as usize)
    }

    pub fn tile(&self, p: TilePos) -> Option<Tile> {
        self.index(p).map(|i| self.tiles[i])
    }

    /// Panics if `p` is out of bounds or the tile's preset/elevation is invalid.
    pub fn set_tile(&mut self, p: TilePos, tile: Tile) {
        assert!(
            (tile.preset.0 as usize) < self.presets.len(),
            "unknown preset {:?}",
            tile.preset
        );
        assert!(
            tile.elevation <= MAX_ELEVATION,
            "elevation {} > {MAX_ELEVATION}",
            tile.elevation
        );
        let i = self.index(p).unwrap_or_else(|| panic!("tile {p:?} out of bounds"));
        self.tiles[i] = tile;
    }

    pub fn presets(&self) -> &[TilePreset] {
        &self.presets
    }

    pub fn preset(&self, id: TilePresetId) -> &TilePreset {
        &self.presets[id.0 as usize]
    }

    /// Preset of the tile at `p`, if in bounds.
    pub fn tile_preset(&self, p: TilePos) -> Option<&TilePreset> {
        self.tile(p).map(|t| self.preset(t.preset))
    }

    /// Register a custom preset (`core/map.md > TilePreset`: any flag combination is valid).
    pub fn add_preset(&mut self, preset: TilePreset) -> TilePresetId {
        self.presets.push(preset);
        TilePresetId((self.presets.len() - 1) as u16)
    }
}
