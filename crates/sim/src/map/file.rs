//! `Map`'s compact serde form: custom presets with their symbols, then the tile and
//! elevation rows as strings (see the `map` module docs).

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::{DEFAULT_PRESET_SYMBOLS, Map, TilePreset, check_new_symbol};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MapFile {
    #[serde(default)]
    custom_presets: Vec<CustomPreset>,
    tiles: Vec<String>,
    /// Empty when omitted: every tile at elevation 0.
    #[serde(default)]
    elevation: Vec<String>,
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
            })
            .collect();
        MapFile {
            custom_presets,
            tiles: self.tile_rows(),
            elevation: self.elevation_rows(),
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
            });
        }
        let tiles: Vec<&str> = file.tiles.iter().map(String::as_str).collect();
        let elevation: Vec<&str> = file.elevation.iter().map(String::as_str).collect();
        let elevation = (!elevation.is_empty()).then_some(elevation.as_slice());
        Map::from_rows(presets, symbols, &tiles, elevation)
    }
}
