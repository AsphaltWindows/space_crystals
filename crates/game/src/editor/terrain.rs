//! The terrain edits behind the Paint, Height and Fill tools, as plain functions on the sim's map, testable without
//! Bevy: brush footprints, symmetry, strokes, painting presets, elevation and flood fill.

use std::collections::BTreeSet;

use sc_sim::{Map, Tile, TilePos, TilePresetId};

use super::tools::{ElevationMode, Symmetry, TOP_ELEVATION};

/// The tiles of a square brush of side `size` (odd) centred on `center`, clipped to the map.
pub fn brush_tiles(map: &Map, center: TilePos, size: u32) -> Vec<TilePos> {
    let r = (size / 2) as i32;
    (center.y - r..=center.y + r)
        .flat_map(|y| (center.x - r..=center.x + r).map(move |x| TilePos::new(x, y)))
        .filter(|p| map.in_bounds(*p))
        .collect()
}

/// `p` followed by its mirror image under `symmetry`, unless the image is `p` itself.
pub fn mirrored_tiles(map: &Map, p: TilePos, symmetry: Symmetry) -> Vec<TilePos> {
    let (w, h) = (map.width() as i32, map.height() as i32);
    let image = match symmetry {
        Symmetry::None => p,
        Symmetry::MirrorX => TilePos::new(w - 1 - p.x, p.y),
        Symmetry::MirrorY => TilePos::new(p.x, h - 1 - p.y),
        Symmetry::Point => TilePos::new(w - 1 - p.x, h - 1 - p.y),
    };
    if image == p { vec![p] } else { vec![p, image] }
}

/// Every tile one brush dab at `center` covers, with its mirror images.
pub fn dab_tiles(map: &Map, center: TilePos, brush: u32, symmetry: Symmetry) -> BTreeSet<TilePos> {
    mirrored_tiles(map, center, symmetry)
        .into_iter()
        .flat_map(|c| brush_tiles(map, c, brush))
        .collect()
}

/// The tiles on the straight line from `a` to `b`, both included, so that a fast stroke leaves no gaps.
pub fn line_tiles(a: TilePos, b: TilePos) -> Vec<TilePos> {
    let (dx, dy) = ((b.x - a.x).abs(), -(b.y - a.y).abs());
    let (sx, sy) = ((b.x - a.x).signum(), (b.y - a.y).signum());
    let (mut p, mut err) = (a, dx + dy);
    let mut line = vec![a];
    while p != b {
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            p.x += sx;
        }
        if e2 <= dx {
            err += dx;
            p.y += sy;
        }
        line.push(p);
    }
    line
}

/// Paints `preset` on `tiles`, keeping their elevation. Returns whether a tile changed.
pub fn paint(map: &mut Map, tiles: impl IntoIterator<Item = TilePos>, preset: TilePresetId) -> bool {
    if preset.0 as usize >= map.presets().len() {
        return false;
    }
    let mut changed = false;
    for p in tiles {
        if let Some(tile) = map.tile(p)
            && tile.preset != preset
        {
            map.set_tile(p, Tile::new(preset, tile.elevation));
            changed = true;
        }
    }
    changed
}

/// The elevation brush (`core/map.md > TilePlacement`, `0..=16`): sets `tiles` to `level`, or raises or lowers
/// them one level. Returns whether a tile changed.
pub fn elevate(map: &mut Map, tiles: impl IntoIterator<Item = TilePos>, mode: ElevationMode, level: u8) -> bool {
    let mut changed = false;
    for p in tiles {
        let Some(tile) = map.tile(p) else {
            continue;
        };
        let elevation = match mode {
            ElevationMode::Set => level.min(TOP_ELEVATION),
            ElevationMode::Raise => tile.elevation.saturating_add(1).min(TOP_ELEVATION),
            ElevationMode::Lower => tile.elevation.saturating_sub(1),
        };
        if elevation != tile.elevation {
            map.set_tile(p, Tile::new(tile.preset, elevation));
            changed = true;
        }
    }
    changed
}

/// Flood-fills the region of equal tiles around `start`, and around its mirror image, with `preset`
/// ([`Map::connected_region`]). Elevations stay. Returns whether a tile changed.
pub fn fill(map: &mut Map, start: TilePos, preset: TilePresetId, symmetry: Symmetry) -> bool {
    let mut changed = false;
    for p in mirrored_tiles(map, start, symmetry) {
        let region = map.connected_region(p);
        changed |= paint(map, region, preset);
    }
    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(x: i32, y: i32) -> TilePos {
        TilePos::new(x, y)
    }

    #[test]
    fn brushes_are_squares_clipped_to_the_map() {
        let map = Map::new(10, 8);
        assert_eq!(brush_tiles(&map, t(4, 4), 1), vec![t(4, 4)]);
        assert_eq!(brush_tiles(&map, t(4, 4), 3).len(), 9);
        assert_eq!(brush_tiles(&map, t(4, 4), 9).len(), 9 * 8, "the 9th row is off the map");
        assert_eq!(brush_tiles(&map, t(0, 0), 5).len(), 9, "a corner keeps a 3×3 quarter");
    }

    #[test]
    fn symmetry_mirrors_tiles_across_the_map() {
        let map = Map::new(10, 8);
        assert_eq!(mirrored_tiles(&map, t(1, 2), Symmetry::None), vec![t(1, 2)]);
        assert_eq!(mirrored_tiles(&map, t(1, 2), Symmetry::MirrorX), vec![t(1, 2), t(8, 2)]);
        assert_eq!(mirrored_tiles(&map, t(1, 2), Symmetry::MirrorY), vec![t(1, 2), t(1, 5)]);
        assert_eq!(mirrored_tiles(&map, t(1, 2), Symmetry::Point), vec![t(1, 2), t(8, 5)]);
        let odd = Map::new(9, 9);
        assert_eq!(
            mirrored_tiles(&odd, t(4, 4), Symmetry::Point),
            vec![t(4, 4)],
            "the center is its own image"
        );
    }

    #[test]
    fn lines_have_no_gaps() {
        assert_eq!(line_tiles(t(0, 0), t(3, 0)), vec![t(0, 0), t(1, 0), t(2, 0), t(3, 0)]);
        assert_eq!(line_tiles(t(2, 2), t(2, 2)), vec![t(2, 2)]);
        let diagonal = line_tiles(t(5, 1), t(1, 4));
        assert_eq!((diagonal[0], *diagonal.last().unwrap()), (t(5, 1), t(1, 4)));
        for pair in diagonal.windows(2) {
            assert!((pair[0].x - pair[1].x).abs() <= 1 && (pair[0].y - pair[1].y).abs() <= 1);
        }
    }

    #[test]
    fn painting_and_elevation_keep_the_other_half_of_the_tile() {
        let mut map = Map::new(4, 4);
        assert!(elevate(&mut map, [t(1, 1)], ElevationMode::Set, 5));
        assert!(paint(&mut map, [t(1, 1), t(2, 1)], TilePresetId::WATER));
        assert_eq!(map.tile(t(1, 1)), Some(Tile::new(TilePresetId::WATER, 5)));
        assert!(!paint(&mut map, [t(1, 1)], TilePresetId::WATER), "no change");
        assert!(
            !paint(&mut map, [t(0, 0)], TilePresetId(99)),
            "unknown presets are ignored"
        );
        assert!(elevate(&mut map, [t(1, 1), t(0, 0)], ElevationMode::Raise, 0));
        assert_eq!(map.tile(t(1, 1)).unwrap().elevation, 6);
        assert_eq!(map.tile(t(0, 0)).unwrap().elevation, 1);
        assert!(elevate(&mut map, [t(0, 0)], ElevationMode::Lower, 0));
        assert!(
            !elevate(&mut map, [t(0, 0)], ElevationMode::Lower, 0),
            "0 is the bottom"
        );
        assert!(elevate(&mut map, [t(3, 3)], ElevationMode::Set, 200));
        assert_eq!(map.tile(t(3, 3)).unwrap().elevation, 16, "16 is the top");
        assert!(!elevate(&mut map, [t(3, 3)], ElevationMode::Raise, 0));
    }

    #[test]
    fn fill_replaces_the_connected_region_and_its_mirror() {
        let mut map = Map::from_ascii("..~..\n..~..\n~~~..").unwrap();
        assert!(fill(&mut map, t(0, 0), TilePresetId::RUGGED, Symmetry::None));
        assert_eq!(map.to_ascii().lines().next(), Some("rr~.."));
        let mut map = Map::from_ascii("..~..\n..~..").unwrap();
        assert!(fill(&mut map, t(0, 0), TilePresetId::MOUNTAIN, Symmetry::MirrorX));
        assert_eq!(map.to_ascii().lines().next(), Some("MM~MM"));
    }
}
