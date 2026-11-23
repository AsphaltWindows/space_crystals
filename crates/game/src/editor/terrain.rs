//! The terrain edits behind the Paint, Height and Fill tools, as plain functions on the sim's map, testable without
//! Bevy: brush footprints, symmetry images, strokes, painting presets, corner heights and flood fill.
//!
//! Heights live on tile corners (`core/map.md > Elevation`). The height brush covers N × N tiles and changes their
//! (N + 1)² corners; [`Map::set_corner_heights`] then moves the corners around them as little as needed for the
//! maximum-slope rule, so slopes form on their own (AoE2-style) and stop at non-Traversible tiles. Painting a
//! Traversible preset is refused on tiles steeper than that rule allows.

use std::collections::{BTreeMap, BTreeSet};

use sc_sim::map::MAX_TRAVERSIBLE_SLOPE;
use sc_sim::{CornerPos, Map, MapTransform, Symmetry, Tile, TilePos, TilePresetId};

use super::tools::{ElevationMode, TOP_ELEVATION};

/// The tiles of a square brush of side `size` (odd) centred on `center`, clipped to the map.
pub fn brush_tiles(map: &Map, center: TilePos, size: u32) -> Vec<TilePos> {
    let r = (size / 2) as i32;
    (center.y - r..=center.y + r)
        .flat_map(|y| (center.x - r..=center.x + r).map(move |x| TilePos::new(x, y)))
        .filter(|p| map.in_bounds(*p))
        .collect()
}

/// The transforms of `symmetry` on this map, the identity first; just the identity when the symmetry doesn't fit
/// the map (90° rotation of a map that isn't square).
pub fn transforms(map: &Map, symmetry: Symmetry) -> Vec<MapTransform> {
    let (w, h) = (map.width(), map.height());
    symmetry
        .transforms(w, h)
        .unwrap_or_else(|| Symmetry::None.transforms(w, h).unwrap_or_default())
}

/// `p` followed by its images under `symmetry`, each once.
pub fn image_tiles(map: &Map, p: TilePos, symmetry: Symmetry) -> Vec<TilePos> {
    let mut images = Vec::new();
    for m in transforms(map, symmetry) {
        let image = m.tile(p);
        if !images.contains(&image) {
            images.push(image);
        }
    }
    images
}

/// Every tile one brush dab at `center` covers, with its symmetry images.
pub fn dab_tiles(map: &Map, center: TilePos, brush: u32, symmetry: Symmetry) -> BTreeSet<TilePos> {
    let tiles = brush_tiles(map, center, brush);
    transforms(map, symmetry)
        .iter()
        .flat_map(|m| tiles.iter().map(|t| m.tile(*t)))
        .filter(|t| map.in_bounds(*t))
        .collect()
}

/// Every corner one height-brush dab at `center` changes: the corners of the brush's tiles, with their symmetry
/// images (the corner grid is one larger than the tile grid).
pub fn dab_corners(map: &Map, center: TilePos, brush: u32, symmetry: Symmetry) -> BTreeSet<CornerPos> {
    let corners: BTreeSet<CornerPos> = brush_tiles(map, center, brush)
        .into_iter()
        .flat_map(TilePos::corners)
        .collect();
    transforms(map, symmetry)
        .iter()
        .flat_map(|m| corners.iter().map(|c| m.corner(*c)))
        .filter(|c| map.corner_in_bounds(*c))
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

/// Whether painting `preset` on tile `t` is refused: a Traversible preset needs a tile at most
/// [`MAX_TRAVERSIBLE_SLOPE`] levels steep (`core/map.md > Elevation > Maximum slope`).
pub fn refuses(map: &Map, t: TilePos, preset: TilePresetId) -> bool {
    let traversible = map.presets().get(preset.0 as usize).is_some_and(|p| p.traversible);
    traversible && map.tile_slope(t).is_some_and(|slope| slope > MAX_TRAVERSIBLE_SLOPE)
}

/// What a paint did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Painted {
    pub changed: bool,
    /// Tiles left as they were because they are too steep for the preset ([`refuses`]).
    pub refused: BTreeSet<TilePos>,
}

/// Paints `preset` on `tiles`; the corner heights stay. Unknown presets paint nothing.
pub fn paint(map: &mut Map, tiles: impl IntoIterator<Item = TilePos>, preset: TilePresetId) -> Painted {
    let mut painted = Painted::default();
    if preset.0 as usize >= map.presets().len() {
        return painted;
    }
    for p in tiles {
        let Some(tile) = map.tile(p) else {
            continue;
        };
        if tile.preset == preset {
            continue;
        }
        if refuses(map, p, preset) {
            painted.refused.insert(p);
            continue;
        }
        map.set_tile(p, Tile::new(preset));
        painted.changed = true;
    }
    painted
}

/// The corner heights a height-brush dab asks for: `level` for each corner, or one level up or down from its
/// current height. Corners off the map ask for nothing.
pub fn height_targets(
    map: &Map,
    corners: impl IntoIterator<Item = CornerPos>,
    mode: ElevationMode,
    level: u8,
) -> BTreeMap<CornerPos, u8> {
    corners
        .into_iter()
        .filter_map(|c| {
            let height = map.corner(c)?;
            let target = match mode {
                ElevationMode::Set => level.min(TOP_ELEVATION),
                ElevationMode::Raise => height.saturating_add(1).min(TOP_ELEVATION),
                ElevationMode::Lower => height.saturating_sub(1),
            };
            Some((c, target))
        })
        .collect()
}

/// Sets the corners of `targets` and lets the slopes around them follow ([`Map::set_corner_heights`]). Returns the
/// corners whose height changed.
pub fn shape(map: &mut Map, targets: &BTreeMap<CornerPos, u8>) -> Vec<CornerPos> {
    map.set_corner_heights(targets)
}

/// Flood-fills the region of equal tiles around `start`, and around its symmetry images, with `preset`
/// ([`Map::connected_region`]); tiles too steep for the preset stay ([`refuses`]).
pub fn fill(map: &mut Map, start: TilePos, preset: TilePresetId, symmetry: Symmetry) -> Painted {
    let mut painted = Painted::default();
    for p in image_tiles(map, start, symmetry) {
        let region = map.connected_region(p);
        let done = paint(map, region, preset);
        painted.changed |= done.changed;
        painted.refused.extend(done.refused);
    }
    painted
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(x: i32, y: i32) -> TilePos {
        TilePos::new(x, y)
    }

    fn c(x: i32, y: i32) -> CornerPos {
        CornerPos::new(x, y)
    }

    #[test]
    fn brushes_are_squares_clipped_to_the_map() {
        let map = Map::new(10, 8);
        assert_eq!(brush_tiles(&map, t(4, 4), 1), vec![t(4, 4)]);
        assert_eq!(brush_tiles(&map, t(4, 4), 3).len(), 9);
        assert_eq!(brush_tiles(&map, t(4, 4), 9).len(), 9 * 8, "the 9th row is off the map");
        assert_eq!(brush_tiles(&map, t(0, 0), 5).len(), 9, "a corner keeps a 3×3 quarter");
        // The height brush changes the (N + 1)² corners of its N × N tiles.
        assert_eq!(dab_corners(&map, t(4, 4), 3, Symmetry::None).len(), 16);
        assert_eq!(dab_corners(&map, t(0, 0), 1, Symmetry::None).len(), 4);
    }

    #[test]
    fn symmetry_repeats_tiles_and_corners_at_their_images() {
        let map = Map::new(10, 8);
        assert_eq!(image_tiles(&map, t(1, 2), Symmetry::None), vec![t(1, 2)]);
        assert_eq!(
            image_tiles(&map, t(1, 2), Symmetry::MirrorVerticalAxis),
            vec![t(1, 2), t(8, 2)]
        );
        assert_eq!(
            image_tiles(&map, t(1, 2), Symmetry::MirrorHorizontalAxis),
            vec![t(1, 2), t(1, 5)]
        );
        assert_eq!(image_tiles(&map, t(1, 2), Symmetry::Rotate180), vec![t(1, 2), t(8, 5)]);
        assert_eq!(
            image_tiles(&map, t(1, 2), Symmetry::MirrorBothAxes),
            vec![t(1, 2), t(8, 2), t(1, 5), t(8, 5)]
        );
        // 90° rotation needs a square map; on another map only the tile itself is edited.
        assert_eq!(image_tiles(&map, t(1, 2), Symmetry::Rotate90), vec![t(1, 2)]);
        let square = Map::new(8, 8);
        assert_eq!(
            image_tiles(&square, t(1, 2), Symmetry::Rotate90),
            vec![t(1, 2), t(5, 1), t(6, 5), t(2, 6)]
        );
        let odd = Map::new(9, 9);
        assert_eq!(
            image_tiles(&odd, t(4, 4), Symmetry::Rotate180),
            vec![t(4, 4)],
            "the center is its own image"
        );
        // Corners: the corner grid is one larger than the tile grid.
        let corners = dab_corners(&map, t(0, 0), 1, Symmetry::Rotate180);
        let expected: BTreeSet<CornerPos> = [c(0, 0), c(1, 0), c(0, 1), c(1, 1), c(9, 7), c(10, 7), c(9, 8), c(10, 8)]
            .into_iter()
            .collect();
        assert_eq!(corners, expected);
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
    fn painting_keeps_the_heights_and_refuses_steep_tiles_for_traversible_presets() {
        // Tile (1, 0) rises 0 → 2: too steep for a Traversible preset, fine for a cliff.
        let mut map = Map::from_ascii("cc.\n\n0022\n0022").unwrap();
        let painted = paint(&mut map, [t(0, 0), t(1, 0)], TilePresetId::PLANE);
        assert!(painted.changed);
        assert_eq!(painted.refused, BTreeSet::from([t(1, 0)]));
        assert_eq!(map.to_ascii(), ".c.\n\n0022\n0022\n");
        assert!(
            paint(&mut map, [t(1, 0)], TilePresetId::WATER).changed,
            "water isn't Traversible"
        );
        assert!(!paint(&mut map, [t(0, 0)], TilePresetId::PLANE).changed, "no change");
        assert!(
            !paint(&mut map, [t(0, 0)], TilePresetId(99)).changed,
            "unknown presets are ignored"
        );
        // Fill refuses the same tiles.
        let mut map = Map::from_ascii("ccc\n\n0022\n0022").unwrap();
        let filled = fill(&mut map, t(0, 0), TilePresetId::RUGGED, Symmetry::None);
        assert_eq!(filled.refused, BTreeSet::from([t(1, 0)]));
        assert_eq!(map.to_ascii().lines().next(), Some("rcr"));
    }

    #[test]
    fn height_targets_set_raise_and_lower_each_corner() {
        let map = Map::from_ascii("..\n\n016\n000").unwrap();
        let corners = [c(0, 0), c(1, 0), c(2, 0), c(9, 9)];
        let set = height_targets(&map, corners, ElevationMode::Set, 200);
        assert_eq!(
            set.values().copied().collect::<Vec<_>>(),
            [16, 16, 16],
            "capped, off-map dropped"
        );
        let raise = height_targets(&map, corners, ElevationMode::Raise, 0);
        assert_eq!(raise.values().copied().collect::<Vec<_>>(), [1, 2, 7]);
        let lower = height_targets(&map, corners, ElevationMode::Lower, 0);
        assert_eq!(lower.values().copied().collect::<Vec<_>>(), [0, 0, 5]);
    }

    /// Raising and lowering with every brush and symmetry leaves no Traversible tile steeper than 1 level: the
    /// slopes cascade outwards and stop at cliffs.
    #[test]
    fn height_brushes_never_leave_a_violation() {
        let mut map = Map::from_ascii(
            "
            ............
            ...cc.......
            ...cc...~~..
            ............
            ......r.....
            ............
            ",
        )
        .unwrap();
        let mut level = 0u8;
        for (i, (x, y)) in [(2, 2), (5, 1), (9, 4), (3, 3), (0, 0), (11, 5), (6, 2)]
            .into_iter()
            .enumerate()
        {
            for brush in [1, 3, 5] {
                for symmetry in Symmetry::ALL {
                    let corners = dab_corners(&map, t(x, y), brush, symmetry);
                    let mode = ElevationMode::ALL[(i + brush as usize) % 3];
                    level = (level + 5) % 17;
                    let targets = height_targets(&map, corners, mode, level);
                    shape(&mut map, &targets);
                    assert_eq!(
                        map.too_steep_tiles(),
                        vec![],
                        "{mode:?} at ({x}, {y}), brush {brush}, {symmetry:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn slopes_stop_at_cliffs() {
        // Raising the corners west of the cliff column to 5 slopes the plane to their west down to 0, and leaves
        // the plane east of the cliffs flat.
        let mut map = Map::from_ascii("......c...\n......c...").unwrap();
        let corners = (0..=2).map(|y| c(6, y));
        let targets = height_targets(&map, corners, ElevationMode::Set, 5);
        shape(&mut map, &targets);
        assert_eq!(map.corner(c(1, 1)), Some(0));
        assert_eq!(map.corner(c(5, 1)), Some(4));
        assert_eq!(map.corner(c(7, 1)), Some(0), "the cliff takes the drop");
        assert_eq!(map.too_steep_tiles(), vec![]);
    }
}
