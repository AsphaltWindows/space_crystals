//! The terrain edits behind the Paint, Height and Fill tools, as plain functions on the sim's map, testable without
//! Bevy: brush footprints, strokes, painting presets, corner heights and flood fill.
//!
//! Heights live on tile corners (`core/map.md > Elevation`). The height brush covers N × N tiles and changes their
//! (N + 1)² corners; [`Map::set_corner_heights`] then moves the corners around them as little as needed for the
//! maximum-slope rule, so slopes form on their own (AoE2-style) and stop at non-Traversible tiles. Painting a
//! Traversible preset is refused on tiles steeper than that rule allows.
//!
//! On a symmetric map the tools change only the editable part ([`MapDef::region`]); the editor then generates the
//! rest from it (`core/map_editor.md > Symmetry > Editing a symmetric map`). Height edits also set the images of
//! their corners, so that the slopes forming around them are symmetric too, across the axes as well.

use std::collections::{BTreeMap, BTreeSet};

use sc_sim::map::MAX_TRAVERSIBLE_SLOPE;
use sc_sim::{CornerPos, Map, MapDef, Tile, TilePos, TilePresetId};

use super::tools::{ElevationMode, TOP_ELEVATION};

/// The tiles of a square brush of side `size` (odd) centred on `center`, clipped to the map.
pub fn brush_tiles(map: &Map, center: TilePos, size: u32) -> Vec<TilePos> {
    let r = (size / 2) as i32;
    (center.y - r..=center.y + r)
        .flat_map(|y| (center.x - r..=center.x + r).map(move |x| TilePos::new(x, y)))
        .filter(|p| map.in_bounds(*p))
        .collect()
}

/// `p` followed by its images under the map's symmetry, each once.
pub fn image_tiles(map: &MapDef, p: TilePos) -> Vec<TilePos> {
    let mut images = Vec::new();
    for m in map.transforms() {
        let image = m.tile(p);
        if !images.contains(&image) {
            images.push(image);
        }
    }
    images
}

/// `tiles` with their images under the map's symmetry: what an edit of `tiles` changes once the locked part is
/// generated.
pub fn tiles_with_images(map: &MapDef, tiles: &BTreeSet<TilePos>) -> BTreeSet<TilePos> {
    let transforms = map.transforms();
    tiles
        .iter()
        .flat_map(|t| transforms.iter().map(move |m| m.tile(*t)))
        .filter(|t| map.terrain.in_bounds(*t))
        .collect()
}

/// `corners` with their images under the map's symmetry.
pub fn corners_with_images(map: &MapDef, corners: &BTreeSet<CornerPos>) -> BTreeSet<CornerPos> {
    let transforms = map.transforms();
    corners
        .iter()
        .flat_map(|c| transforms.iter().map(move |m| m.corner(*c)))
        .filter(|c| map.terrain.corner_in_bounds(*c))
        .collect()
}

/// The tiles one brush dab at `center` edits: the brush's tiles in the editable part.
pub fn dab_tiles(map: &MapDef, center: TilePos, brush: u32) -> BTreeSet<TilePos> {
    let region = map.region();
    brush_tiles(&map.terrain, center, brush)
        .into_iter()
        .filter(|t| region.tile(*t))
        .collect()
}

/// The corners one height-brush dab at `center` sets: the corners of its tiles ([`dab_tiles`]; the corner grid is
/// one larger than the tile grid).
pub fn dab_corners(map: &MapDef, center: TilePos, brush: u32) -> BTreeSet<CornerPos> {
    dab_tiles(map, center, brush)
        .into_iter()
        .flat_map(TilePos::corners)
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

/// One height dab of `corners` (editable ones): their targets, with their images', set on the map. Returns the
/// corners whose height changed.
pub fn raise_or_set(map: &mut MapDef, corners: &BTreeSet<CornerPos>, mode: ElevationMode, level: u8) -> Vec<CornerPos> {
    let all = corners_with_images(map, corners);
    let targets = height_targets(&map.terrain, all, mode, level);
    shape(&mut map.terrain, &targets)
}

/// The tiles a fill from `start` changes: the region of equal tiles around it ([`Map::connected_region`]) and the
/// regions around its images, in the editable part. The rest of them is generated.
pub fn fill_tiles(map: &MapDef, start: TilePos) -> BTreeSet<TilePos> {
    let region = map.region();
    image_tiles(map, start)
        .into_iter()
        .filter(|p| map.terrain.in_bounds(*p))
        .flat_map(|p| map.terrain.connected_region(p))
        .filter(|t| region.tile(*t))
        .collect()
}

/// Flood-fills from `start` with `preset` ([`fill_tiles`]); tiles too steep for the preset stay ([`refuses`]).
pub fn fill(map: &mut MapDef, start: TilePos, preset: TilePresetId) -> Painted {
    let tiles = fill_tiles(map, start);
    paint(&mut map.terrain, tiles, preset)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sc_sim::{Symmetry, scenarios};

    fn t(x: i32, y: i32) -> TilePos {
        TilePos::new(x, y)
    }

    fn c(x: i32, y: i32) -> CornerPos {
        CornerPos::new(x, y)
    }

    /// A map of `terrain` with `symmetry` (whatever its slots).
    fn with(terrain: Map, symmetry: Symmetry) -> MapDef {
        MapDef {
            terrain,
            symmetry,
            ..scenarios::empty_map(4, 4)
        }
    }

    #[test]
    fn brushes_are_squares_clipped_to_the_map() {
        let map = with(Map::new(10, 8), Symmetry::None);
        assert_eq!(brush_tiles(&map.terrain, t(4, 4), 1), vec![t(4, 4)]);
        assert_eq!(brush_tiles(&map.terrain, t(4, 4), 3).len(), 9);
        assert_eq!(
            brush_tiles(&map.terrain, t(4, 4), 9).len(),
            9 * 8,
            "the 9th row is off the map"
        );
        assert_eq!(
            brush_tiles(&map.terrain, t(0, 0), 5).len(),
            9,
            "a corner keeps a 3×3 quarter"
        );
        // The height brush changes the (N + 1)² corners of its N × N tiles.
        assert_eq!(dab_corners(&map, t(4, 4), 3).len(), 16);
        assert_eq!(dab_corners(&map, t(0, 0), 1).len(), 4);
    }

    /// On a symmetric map a dab edits only its tiles in the editable part; their images follow.
    #[test]
    fn dabs_edit_the_editable_part_only() {
        let map = with(Map::new(10, 8), Symmetry::MirrorVerticalAxis);
        // A 3x3 brush on the axis: the west columns 4 only; column 5 is the image of column 4.
        let tiles = dab_tiles(&map, t(5, 3), 3);
        assert_eq!(tiles, BTreeSet::from([t(4, 2), t(4, 3), t(4, 4)]));
        assert_eq!(tiles_with_images(&map, &tiles).len(), 6);
        assert!(dab_tiles(&map, t(8, 3), 1).is_empty(), "the east half is locked");
        let corners = dab_corners(&map, t(4, 3), 1);
        assert_eq!(corners.len(), 4);
        assert_eq!(
            corners_with_images(&map, &corners),
            BTreeSet::from([c(4, 3), c(5, 3), c(6, 3), c(4, 4), c(5, 4), c(6, 4)]),
            "the axis corners are their own images"
        );
    }

    #[test]
    fn images_of_tiles() {
        let map = |s| with(Map::new(10, 8), s);
        assert_eq!(image_tiles(&map(Symmetry::None), t(1, 2)), vec![t(1, 2)]);
        assert_eq!(
            image_tiles(&map(Symmetry::MirrorVerticalAxis), t(1, 2)),
            vec![t(1, 2), t(8, 2)]
        );
        assert_eq!(
            image_tiles(&map(Symmetry::MirrorHorizontalAxis), t(1, 2)),
            vec![t(1, 2), t(1, 5)]
        );
        assert_eq!(image_tiles(&map(Symmetry::Rotate180), t(1, 2)), vec![t(1, 2), t(8, 5)]);
        assert_eq!(
            image_tiles(&map(Symmetry::MirrorBothAxes), t(1, 2)),
            vec![t(1, 2), t(8, 2), t(1, 5), t(8, 5)]
        );
        // 90° rotation needs a square map; on another map only the tile itself is edited.
        assert_eq!(image_tiles(&map(Symmetry::Rotate90), t(1, 2)), vec![t(1, 2)]);
        let square = with(Map::new(8, 8), Symmetry::Rotate90);
        assert_eq!(image_tiles(&square, t(1, 2)), vec![t(1, 2), t(5, 1), t(6, 5), t(2, 6)]);
        let odd = with(Map::new(9, 9), Symmetry::Rotate180);
        assert_eq!(image_tiles(&odd, t(4, 4)), vec![t(4, 4)], "the center is its own image");
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
        let mut map = with(Map::from_ascii("ccc\n\n0022\n0022").unwrap(), Symmetry::None);
        let filled = fill(&mut map, t(0, 0), TilePresetId::RUGGED);
        assert_eq!(filled.refused, BTreeSet::from([t(1, 0)]));
        assert_eq!(map.terrain.to_ascii().lines().next(), Some("rcr"));
    }

    /// A fill on a symmetric map takes the regions around the start's images too, in the editable part: a region
    /// that crosses the axis and its image, which may be another region.
    #[test]
    fn fills_cover_the_regions_of_the_images() {
        let terrain = Map::from_ascii(
            "
            ..~~~~..
            ......~.
            .~......
            ..~~~~..
            ",
        )
        .unwrap();
        let map = with(terrain, Symmetry::Rotate180);
        // The water at the top crosses the vertical center line; its image is the water at the bottom.
        let tiles = fill_tiles(&map, t(2, 0));
        assert_eq!(tiles, BTreeSet::from([t(2, 0), t(3, 0), t(2, 3), t(3, 3)]));
        assert!(tiles.iter().all(|t| map.region().tile(*t)));
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

    /// Raising and lowering with every brush and symmetry leaves no Traversible tile steeper than 1 level, and a
    /// symmetric map symmetric: the slopes cascade outwards, the same way on every side, and stop at cliffs.
    #[test]
    fn height_brushes_never_leave_a_violation_or_an_asymmetry() {
        let terrain = Map::from_ascii(
            "
            ............
            ...cc....cc.
            ...cc....cc.
            ............
            ......r.....
            ..r.........
            ......~~....
            ......~~....
            .........r..
            ............
            .cc.....r...
            .cc....cc...
            ",
        )
        .unwrap();
        for symmetry in Symmetry::ALL {
            let mut map = with(terrain.clone(), symmetry);
            map.regenerate();
            let mut level = 0u8;
            for (i, (x, y)) in [(2, 2), (5, 1), (9, 4), (3, 3), (0, 0), (5, 5), (6, 2)]
                .into_iter()
                .enumerate()
            {
                for brush in [1, 3, 5] {
                    let corners = dab_corners(&map, t(x, y), brush);
                    let mode = ElevationMode::ALL[(i + brush as usize) % 3];
                    level = (level + 5) % 17;
                    raise_or_set(&mut map, &corners, mode, level);
                    let at = format!("{mode:?} at ({x}, {y}), brush {brush}, {symmetry:?}");
                    assert_eq!(map.terrain.too_steep_tiles(), vec![], "{at}");
                    assert_eq!(map.first_asymmetry(), None, "{at}");
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
