//! `Map`'s compact serde form: custom presets with their symbols, then the tile rows and
//! the corner height rows as strings (see the `map` module docs).

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::{DEFAULT_PRESET_SYMBOLS, Map, TileLook, TilePreset, check_new_symbol};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MapFile {
    #[serde(default)]
    custom_presets: Vec<CustomPreset>,
    tiles: Vec<String>,
    /// One row per corner row (`core/map.md > Elevation`). Empty when omitted: every corner at height 0.
    #[serde(default)]
    heights: Vec<String>,
}

/// A custom preset and its symbol in the rows.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CustomPreset {
    symbol: char,
    name: String,
    buildable: bool,
    traversible: bool,
    rugged: bool,
    drillable: bool,
    recruitable: bool,
    look: TileLook,
}

impl Serialize for Map {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let defaults = DEFAULT_PRESET_SYMBOLS.len();
        let custom_presets = self.presets[defaults..]
            .iter()
            .zip(&self.symbols[defaults..])
            .map(|(p, &symbol)| CustomPreset {
                symbol,
                name: p.name.clone(),
                buildable: p.buildable,
                traversible: p.traversible,
                rugged: p.rugged,
                drillable: p.drillable,
                recruitable: p.recruitable,
                look: p.look,
            })
            .collect();
        MapFile {
            custom_presets,
            tiles: self.tile_rows(),
            heights: self.height_rows(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Map {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let file = MapFile::deserialize(deserializer)?;
        Map::try_from(file).map_err(D::Error::custom)
    }
}

impl TryFrom<MapFile> for Map {
    type Error = String;

    fn try_from(file: MapFile) -> Result<Self, String> {
        let mut presets = Map::default_presets();
        let mut symbols = DEFAULT_PRESET_SYMBOLS.to_vec();
        for p in file.custom_presets {
            check_new_symbol(&presets, &symbols, p.symbol).map_err(|e| format!("custom preset {:?}: {e}", p.name))?;
            symbols.push(p.symbol);
            presets.push(TilePreset {
                name: p.name,
                buildable: p.buildable,
                traversible: p.traversible,
                rugged: p.rugged,
                drillable: p.drillable,
                recruitable: p.recruitable,
                look: p.look,
            });
        }
        let tiles: Vec<&str> = file.tiles.iter().map(String::as_str).collect();
        let heights: Vec<&str> = file.heights.iter().map(String::as_str).collect();
        let heights = (!heights.is_empty()).then_some(heights.as_slice());
        Map::from_rows(presets, symbols, &tiles, heights)
    }
}
