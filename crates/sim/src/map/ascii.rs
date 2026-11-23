//! The ASCII map format and the row encoding it shares with the serde form
//! (see the `map` module docs).

use super::{DEFAULT_PRESET_SYMBOLS, MAX_ELEVATION, MAX_MAP_SIDE, Map, Tile, TilePreset, TilePresetId, corner_count};

/// Height digits, base 17: index = height (`core/map.md > Elevation > CornerHeight`).
const HEIGHT_DIGITS: &str = "0123456789abcdefg";

fn height_of_digit(c: char) -> Option<u8> {
    // `find` returns a byte offset, which equals the height since all digits are ASCII.
    HEIGHT_DIGITS.find(c).map(|i| i as u8)
}

fn digit_of_height(height: u8) -> char {
    char::from(HEIGHT_DIGITS.as_bytes()[usize::from(height)])
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
            [tiles, heights] => Map::from_rows(presets, symbols, tiles, Some(heights)),
            _ => Err(format!(
                "expected a tile block and an optional height block, found {} blocks separated by blank lines",
                blocks.len()
            )),
        }
    }

    /// The map in the ASCII format: the tile block, a blank line, then the height block.
    /// Custom presets appear as their symbols, which [`Map::from_ascii`] doesn't know.
    pub fn to_ascii(&self) -> String {
        let mut out = String::new();
        for row in self.tile_rows() {
            out.push_str(&row);
            out.push('\n');
        }
        out.push('\n');
        for row in self.height_rows() {
            out.push_str(&row);
            out.push('\n');
        }
        out
    }

    /// One string of preset symbols per tile row, north to south.
    pub(super) fn tile_rows(&self) -> Vec<String> {
        self.tiles
            .chunks(self.width as usize)
            .map(|row| row.iter().map(|t| self.symbols[usize::from(t.preset.0)]).collect())
            .collect()
    }

    /// One string of height digits per corner row, north to south.
    pub(super) fn height_rows(&self) -> Vec<String> {
        self.heights
            .chunks(self.width as usize + 1)
            .map(|row| row.iter().map(|&h| digit_of_height(h)).collect())
            .collect()
    }

    /// Builds a map from rows of preset symbols and, optionally, rows of corner height digits
    /// (all 0 when `None`). `symbols[i]` is the symbol of `presets[i]`.
    pub(super) fn from_rows(
        presets: Vec<TilePreset>,
        symbols: Vec<char>,
        tile_rows: &[&str],
        height_rows: Option<&[&str]>,
    ) -> Result<Map, String> {
        let Some(first) = tile_rows.first() else {
            return Err("the map has no tile rows".into());
        };
        let width = first.chars().count();
        if width == 0 {
            return Err("tile row 0 is empty".into());
        }
        let height = tile_rows.len();
        // Before anything is allocated for the map.
        let max = MAX_MAP_SIDE as usize;
        if width > max || height > max {
            return Err(format!(
                "the map is too big: {width} x {height} tiles (at most {max} x {max})"
            ));
        }
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
                tiles.push(Tile::new(TilePresetId(i as u16)));
            }
        }
        // Both sides are at most `MAX_MAP_SIDE`.
        let (w, h) = (width as u32, height as u32);
        let mut heights = vec![0; corner_count(w, h)];
        if let Some(rows) = height_rows {
            if rows.len() != height + 1 {
                return Err(format!(
                    "the height block has {} rows, expected {} (one per corner row: one more than the {height} tile rows)",
                    rows.len(),
                    height + 1
                ));
            }
            for (y, row) in rows.iter().enumerate() {
                let len = row.chars().count();
                if len != width + 1 {
                    return Err(format!(
                        "height row {y} has {len} digits, expected {} (one per corner: one more than the {width} tiles of a row)",
                        width + 1
                    ));
                }
                for (x, c) in row.chars().enumerate() {
                    let Some(value) = height_of_digit(c) else {
                        return Err(format!(
                            "invalid height digit {c:?} at corner ({x}, {y}): use 0-9 and a-g for 0 to {MAX_ELEVATION}"
                        ));
                    };
                    heights[y * (width + 1) + x] = value;
                }
            }
        }
        Ok(Map {
            width: w,
            height: h,
            presets,
            symbols,
            tiles,
            heights,
        })
    }
}
