//! Tiles, tile presets and terrain heights — `core/map.md`.
//!
//! A `W × H` map has `W × H` tiles, each a [`TilePreset`], and `(W + 1) × (H + 1)` corner heights
//! (`core/map.md > Elevation`): adjacent tiles share their corners, so the terrain is continuous. Height queries
//! (interpolation, flatness, slope) are in `map/height.rs`.
//!
//! # Text formats
//!
//! Each tile preset has a one-character **symbol**. The default presets
//! (`core/map.md > DefaultTilePresets`) always use [`DEFAULT_PRESET_SYMBOLS`]:
//!
//! | Symbol | Preset |
//! |---|---|
//! | `.` | Plane |
//! | `r` | Rugged Terrain |
//! | `c` | Cliff |
//! | `M` | Mountain |
//! | `~` | Water |
//!
//! Custom presets get their own symbol: any printable ASCII character except space, `"`
//! and `\` that no other preset of the map uses (see [`Map::add_preset_with_symbol`]).
//!
//! A corner height (`core/map.md > Elevation > CornerHeight - integer (0 to 16)`) is one
//! base-17 digit: `0`–`9` for 0 to 9, then `a`–`g` for 10 to 16.
//!
//! ## ASCII maps ([`Map::from_ascii`], [`Map::to_ascii`])
//!
//! A tile block of preset symbols, one line per row from north to south, optionally
//! followed by a blank line and a **height block**: one line per corner row, north to
//! south, with one digit per corner, so it is one row and one column larger than the tile
//! block. Without a height block every corner is at height 0. Lines are trimmed, so the
//! text may be indented, and leading and trailing blank lines are ignored. `from_ascii`
//! knows the default presets only; a map with custom presets round-trips through its
//! serde form instead. Here a column of cliffs separates low ground in the west from a
//! plateau 2 levels up in the east:
//!
//! ```text
//! ..c.
//! ..c.
//! ..c.
//!
//! 00022
//! 00022
//! 00022
//! 00022
//! ```
//!
//! ## Serde form (scenario files, saves)
//!
//! `Map` serializes as an object with three fields: `custom_presets`, the presets after
//! the defaults in [`TilePresetId`] order (each with its `symbol`, `name`, flags and `look`),
//! then `tiles` and `heights`, the two ASCII blocks as arrays of row strings (`H` tile
//! rows of `W` symbols, `H + 1` corner rows of `W + 1` digits). The default presets are
//! implied, so they always match the design. When reading, `custom_presets` and `heights`
//! may be omitted (none, and every corner at height 0). Pretty-printed JSON shows one
//! map row per line:
//!
//! ```json
//! {
//!   "custom_presets": [
//!     {
//!       "symbol": "A",
//!       "name": "Ash Field",
//!       "buildable": true,
//!       "traversible": true,
//!       "rugged": false,
//!       "drillable": false,
//!       "recruitable": true,
//!       "look": "Rugged"
//!     }
//!   ],
//!   "tiles": [
//!     "..AA",
//!     "~~.."
//!   ],
//!   "heights": [
//!     "00111",
//!     "00111",
//!     "00000"
//!   ]
//! }
//! ```

mod ascii;
mod edit;
mod file;
mod height;

pub use edit::PresetEditError;

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::hash::{StateHash, StateHasher};
use crate::units::{CornerPos, TilePos};

/// `core/map.md > Elevation > CornerHeight - integer (0 to 16)`.
pub const MAX_ELEVATION: u8 = 16;

/// The longest map side, in tiles (placeholder until the design settles map sizes, `core/map_editor.md > Map
/// Creation`). Loading checks it before allocating anything for the map, so a damaged or hostile file can't make
/// it reserve huge amounts of memory; validation rejects bigger maps built in code.
pub const MAX_MAP_SIDE: u32 = 256;

/// `core/map.md > Elevation > Maximum slope`: on a Traversible tile, the four corner heights differ by at most 1
/// level. Steeper tiles must be non-Traversible (cliffs).
pub const MAX_TRAVERSIBLE_SLOPE: u8 = 1;

/// Text symbols of the default presets, in [`TilePresetId`] order (see the module docs).
pub const DEFAULT_PRESET_SYMBOLS: [char; 5] = ['.', 'r', 'c', 'M', '~'];

/// `core/map.md > TilePreset`. The sim needs only the flags; `look` says how the client draws it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TilePreset {
    pub name: String,
    pub buildable: bool,
    pub traversible: bool,
    pub rugged: bool,
    pub drillable: bool,
    pub recruitable: bool,
    /// `core/map.md > TilePreset > Texture`.
    pub look: TileLook,
}

/// `core/map.md > TilePreset > Texture`: how the client draws a preset. Every look is the texture of a default
/// preset: custom presets pick one of them until maps can bring their own textures (`core/map_editor.md > Custom Tile
/// Presets`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TileLook {
    Plane,
    Rugged,
    Cliff,
    Mountain,
    Water,
}

impl TileLook {
    /// Every look, in declaration order (the default presets' order).
    pub const ALL: [TileLook; 5] = [
        TileLook::Plane,
        TileLook::Rugged,
        TileLook::Cliff,
        TileLook::Mountain,
        TileLook::Water,
    ];

    pub fn label(self) -> &'static str {
        match self {
            TileLook::Plane => "Plane",
            TileLook::Rugged => "Rugged",
            TileLook::Cliff => "Cliff",
            TileLook::Mountain => "Mountain",
            TileLook::Water => "Water",
        }
    }
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

    /// Whether it is one of the default presets, which every map has and no map changes.
    pub fn is_default(self) -> bool {
        usize::from(self.0) < DEFAULT_PRESET_SYMBOLS.len()
    }
}

/// `core/map.md > TilePlacement`: a preset at a location. Its height comes from the map's corner heights
/// (`core/map.md > Elevation`), not from the tile.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tile {
    pub preset: TilePresetId,
}

impl Tile {
    pub const fn new(preset: TilePresetId) -> Self {
        Self { preset }
    }
}

/// The tile grid and its corner heights. Both are stored row-major: tile `(x, y)` at `y * width + x`, corner
/// `(x, y)` at `y * (width + 1) + x`. Serde uses the compact text form described in the module docs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Map {
    width: u32,
    height: u32,
    presets: Vec<TilePreset>,
    /// Text symbol of each preset, same index as `presets`.
    symbols: Vec<char>,
    tiles: Vec<Tile>,
    /// `(width + 1) × (height + 1)` corner heights, each `0..=MAX_ELEVATION` (`core/map.md > Elevation`).
    heights: Vec<u8>,
}

impl Map {
    /// A `width × height` map of Plane tiles with every corner at height 0, with the default presets.
    pub fn new(width: u32, height: u32) -> Self {
        assert!(width > 0 && height > 0, "map must not be empty");
        Self {
            width,
            height,
            presets: Self::default_presets(),
            symbols: DEFAULT_PRESET_SYMBOLS.to_vec(),
            tiles: vec![Tile::new(TilePresetId::PLANE); width as usize * height as usize],
            heights: vec![0; corner_count(width, height)],
        }
    }

    /// `core/map.md > DefaultTilePresets`, in [`TilePresetId`] order.
    pub fn default_presets() -> Vec<TilePreset> {
        let preset = |name: &str, buildable, traversible, rugged, drillable, recruitable, look| TilePreset {
            name: name.to_string(),
            buildable,
            traversible,
            rugged,
            drillable,
            recruitable,
            look,
        };
        vec![
            preset("Plane", true, true, false, true, true, TileLook::Plane),
            preset("Rugged Terrain", false, true, true, true, true, TileLook::Rugged),
            preset("Cliff", false, false, false, true, true, TileLook::Cliff),
            preset("Mountain", false, false, false, false, true, TileLook::Mountain),
            preset("Water", false, false, false, false, false, TileLook::Water),
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

    /// Whether a point (space units) lies on the map. Non-finite points (NaN, ±∞) never do.
    pub fn contains_point(&self, p: Vec2) -> bool {
        // `NaN as i32` is 0, so without this check NaN would land on tile (0, 0).
        p.is_finite() && self.in_bounds(TilePos::containing(p))
    }

    fn index(&self, p: TilePos) -> Option<usize> {
        self.in_bounds(p)
            .then(|| p.y as usize * self.width as usize + p.x as usize)
    }

    pub fn tile(&self, p: TilePos) -> Option<Tile> {
        self.index(p).map(|i| self.tiles[i])
    }

    /// Panics if `p` is out of bounds or the preset is unknown.
    pub fn set_tile(&mut self, p: TilePos, tile: Tile) {
        assert!(
            (tile.preset.0 as usize) < self.presets.len(),
            "unknown preset {:?}",
            tile.preset
        );
        let i = self.index(p).unwrap_or_else(|| panic!("tile {p:?} out of bounds"));
        self.tiles[i] = tile;
    }

    /// Whether corner `c` exists: `(0..=width, 0..=height)`.
    pub fn corner_in_bounds(&self, c: CornerPos) -> bool {
        c.x >= 0 && c.y >= 0 && c.x as u32 <= self.width && c.y as u32 <= self.height
    }

    fn corner_index(&self, c: CornerPos) -> Option<usize> {
        self.corner_in_bounds(c)
            .then(|| c.y as usize * (self.width as usize + 1) + c.x as usize)
    }

    /// The height of corner `c` (`core/map.md > Elevation > CornerHeight`), if it exists.
    pub fn corner(&self, c: CornerPos) -> Option<u8> {
        self.corner_index(c).map(|i| self.heights[i])
    }

    /// Panics if `c` is out of bounds or `height` is above [`MAX_ELEVATION`].
    pub fn set_corner(&mut self, c: CornerPos, height: u8) {
        assert!(height <= MAX_ELEVATION, "height {height} > {MAX_ELEVATION}");
        let i = self
            .corner_index(c)
            .unwrap_or_else(|| panic!("corner {c:?} out of bounds"));
        self.heights[i] = height;
    }

    pub fn presets(&self) -> &[TilePreset] {
        &self.presets
    }

    pub fn preset(&self, id: TilePresetId) -> &TilePreset {
        &self.presets[id.0 as usize]
    }

    /// Text symbol of a preset (see the module docs). Panics for an unknown id, like [`Map::preset`].
    pub fn preset_symbol(&self, id: TilePresetId) -> char {
        self.symbols[id.0 as usize]
    }

    /// Preset of the tile at `p`, if in bounds.
    pub fn tile_preset(&self, p: TilePos) -> Option<&TilePreset> {
        self.tile(p).map(|t| self.preset(t.preset))
    }

    /// Register a custom preset (`core/map.md > TilePreset`: any flag combination is valid).
    /// Its symbol is the first free one of `A`–`Z`, `a`–`z`, `0`–`9`, then punctuation.
    ///
    /// Panics if every symbol is taken (87 custom presets); [`Map::add_preset_with_symbol`]
    /// reports that as an error instead.
    pub fn add_preset(&mut self, preset: TilePreset) -> TilePresetId {
        let symbol = ('A'..='Z')
            .chain('a'..='z')
            .chain('0'..='9')
            .chain('!'..='~')
            .find(|&c| is_valid_symbol(c) && !self.symbols.contains(&c))
            .expect("every preset symbol is taken");
        self.push_preset(preset, symbol)
    }

    /// Register a custom preset with a chosen text symbol. Fails if the symbol isn't a
    /// printable ASCII character other than space, `"` and `\`, or another preset has it.
    pub fn add_preset_with_symbol(&mut self, preset: TilePreset, symbol: char) -> Result<TilePresetId, String> {
        check_new_symbol(&self.presets, &self.symbols, symbol)?;
        Ok(self.push_preset(preset, symbol))
    }

    fn push_preset(&mut self, preset: TilePreset, symbol: char) -> TilePresetId {
        self.presets.push(preset);
        self.symbols.push(symbol);
        TilePresetId((self.presets.len() - 1) as u16)
    }
}

/// Symbols that stay one aligned character in a JSON string: printable ASCII except
/// space, `"` and `\` (JSON escapes those two).
fn is_valid_symbol(c: char) -> bool {
    c.is_ascii_graphic() && c != '"' && c != '\\'
}

/// Whether `symbol` can be added to a preset table whose symbols are `symbols`.
fn check_new_symbol(presets: &[TilePreset], symbols: &[char], symbol: char) -> Result<(), String> {
    if !is_valid_symbol(symbol) {
        return Err(format!(
            "invalid preset symbol {symbol:?}: use a printable ASCII character other than space, '\"' and '\\'"
        ));
    }
    if let Some(i) = symbols.iter().position(|&s| s == symbol) {
        return Err(format!(
            "preset symbol {symbol:?} is already used by preset {:?}",
            presets[i].name
        ));
    }
    Ok(())
}

impl StateHash for Map {
    fn hash(&self, h: &mut StateHasher) {
        let Map {
            width,
            height,
            presets,
            symbols,
            tiles,
            heights,
        } = self;
        width.hash(h);
        height.hash(h);
        presets.hash(h);
        symbols.hash(h);
        tiles.hash(h);
        heights.hash(h);
    }
}

/// How many corners a `width × height` map has.
fn corner_count(width: u32, height: u32) -> usize {
    (width as usize + 1) * (height as usize + 1)
}

#[cfg(test)]
mod tests;
