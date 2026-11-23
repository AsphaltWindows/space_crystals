//! The ASCII map format and the row encoding it shares with the serde form
//! (see the `map` module docs).

use super::{DEFAULT_PRESET_SYMBOLS, MAX_ELEVATION, Map, Tile, TilePreset, TilePresetId};

/// Elevation digits, base 17: index = elevation (`core/map.md > TilePlacement`).
const ELEVATION_DIGITS: &str = "0123456789abcdefg";

fn elevation_of_digit(c: char) -> Option<u8> {
    // `find` returns a byte offset, which equals the elevation since all digits are ASCII.
    ELEVATION_DIGITS.find(c).map(|i| i as u8)
}

fn digit_of_elevation(elevation: u8) -> char {
    char::from(ELEVATION_DIGITS.as_bytes()[usize::from(elevation)])
}

impl Map {
    /// Parses an ASCII map (format in the `map` module docs). The error says what is
    /// wrong and where, e.g. a ragged row or an unknown symbol.
    pub fn from_ascii(ascii: &str) -> Result<Map, String> {
        let mut blocks: Vec<Vec<&str>> = Vec::new();
        let mut block = Vec::new();
        for line in ascii.lines().map(str::trim) {
            if !line.is_empty() {
                block.push(line);
            } else if !block.is_empty() {
                blocks.push(std::mem::take(&mut block));
            }
        }
        if !block.is_empty() {
            blocks.push(block);
        }
        let presets = Map::default_presets();
        let symbols = DEFAULT_PRESET_SYMBOLS.to_vec();
        match blocks.as_slice() {
            [] => Err("the map has no tile rows".into()),
            [tiles] => Map::from_rows(presets, symbols, tiles, None),
            [tiles, elevation] => Map::from_rows(presets, symbols, tiles, Some(elevation)),
            _ => Err(format!(
                "expected a tile block and an optional elevation block, found {} blocks separated by blank lines",
                blocks.len()
            )),
        }
    }

    /// The map in the ASCII format: the tile block, a blank line, then the elevation block.
    /// Custom presets appear as their symbols, which [`Map::from_ascii`] doesn't know.
    pub fn to_ascii(&self) -> String {
        let mut out = String::new();
        for row in self.tile_rows() {
            out.push_str(&row);
            out.push('\n');
        }
        out.push('\n');
        for row in self.elevation_rows() {
            out.push_str(&row);
            out.push('\n');
        }
        out
    }

    /// One string of preset symbols per row, north to south.
    pub(super) fn tile_rows(&self) -> Vec<String> {
        self.rows(|t| self.symbols[usize::from(t.preset.0)])
    }

    /// One string of elevation digits per row, north to south.
    pub(super) fn elevation_rows(&self) -> Vec<String> {
        self.rows(|t| digit_of_elevation(t.elevation))
    }

    fn rows(&self, cell: impl Fn(&Tile) -> char) -> Vec<String> {
        self.tiles
            .chunks(self.width as usize)
            .map(|row| row.iter().map(&cell).collect())
            .collect()
    }

    /// Builds a map from rows of preset symbols and, optionally, rows of elevation digits
    /// (all 0 when `None`). `symbols[i]` is the symbol of `presets[i]`.
    pub(super) fn from_rows(
        presets: Vec<TilePreset>,
        symbols: Vec<char>,
        tile_rows: &[&str],
        elevation_rows: Option<&[&str]>,
    ) -> Result<Map, String> {
        let Some(first) = tile_rows.first() else {
            return Err("the map has no tile rows".into());
        };
        let width = first.chars().count();
        if width == 0 {
            return Err("tile row 0 is empty".into());
        }
        let height = tile_rows.len();
        let mut tiles = Vec::with_capacity(width * height);
        for (y, row) in tile_rows.iter().enumerate() {
            let len = row.chars().count();
            if len != width {
                return Err(format!("ragged tile rows: row {y} has {len} tiles, row 0 has {width}"));
            }
            for (x, c) in row.chars().enumerate() {
                let Some(i) = symbols.iter().position(|&s| s == c) else {
                    return Err(format!("unknown tile symbol {c:?} at tile ({x}, {y})"));
                };
                tiles.push(Tile::new(TilePresetId(i as u16), 0));
            }
        }
        if let Some(rows) = elevation_rows {
            if rows.len() != height {
                return Err(format!(
                    "the elevation block has {} rows, the tile block has {height}",
                    rows.len()
                ));
            }
            for (y, row) in rows.iter().enumerate() {
                let len = row.chars().count();
                if len != width {
                    return Err(format!(
                        "ragged elevation rows: row {y} has {len} digits, the tile rows have {width}"
                    ));
                }
                for (x, c) in row.chars().enumerate() {
                    let Some(elevation) = elevation_of_digit(c) else {
                        return Err(format!(
                            "invalid elevation digit {c:?} at tile ({x}, {y}): use 0-9 and a-g for 0 to {MAX_ELEVATION}"
                        ));
                    };
                    tiles[y * width + x].elevation = elevation;
                }
            }
        }
        let too_big = || format!("the map is too big: {width} × {height} tiles");
        Ok(Map {
            width: u32::try_from(width).map_err(|_| too_big())?,
            height: u32::try_from(height).map_err(|_| too_big())?,
            presets,
            symbols,
            tiles,
        })
    }
}
