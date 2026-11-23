//! Map editing helpers for tools such as the map editor: resizing, flood-fill regions, and setting corner heights
//! so that slopes form on their own.

use std::collections::{BTreeMap, VecDeque};

use super::{MAX_ELEVATION, MAX_TRAVERSIBLE_SLOPE, Map, Tile, TilePresetId, corner_count};
use crate::units::{CornerPos, TilePos};

impl Map {
    /// Resizes the map to `width × height`, anchored at the top-left corner. Tiles and corners that still fit
    /// keep their preset and height; new tiles are Plane, and each new corner takes the height of the nearest old
    /// corner, so the new strip continues the old edge flat outwards. The preset table stays as it is.
    ///
    /// Panics if a side is 0, like [`Map::new`].
    pub fn resize(&mut self, width: u32, height: u32) {
        assert!(width > 0 && height > 0, "map must not be empty");
        let (new_w, new_h) = (width as usize, height as usize);
        let (old_w, old_h) = (self.width as usize, self.height as usize);
        let mut tiles = vec![Tile::new(TilePresetId::PLANE); new_w * new_h];
        for y in 0..new_h.min(old_h) {
            let keep = new_w.min(old_w);
            tiles[y * new_w..y * new_w + keep].copy_from_slice(&self.tiles[y * old_w..y * old_w + keep]);
        }
        let mut heights = vec![0; corner_count(width, height)];
        for y in 0..=new_h {
            for x in 0..=new_w {
                heights[y * (new_w + 1) + x] = self.heights[y.min(old_h) * (old_w + 1) + x.min(old_w)];
            }
        }
        self.width = width;
        self.height = height;
        self.tiles = tiles;
        self.heights = heights;
    }

    /// The region a flood fill started at `start` covers: every tile 4-connected to `start` through tiles of the
    /// same preset. `start` comes first, then the rest in breadth-first order, with neighbours visited east, west,
    /// south, north. Empty if `start` is off the map.
    pub fn connected_region(&self, start: TilePos) -> Vec<TilePos> {
        let Some(target) = self.tile(start) else {
            return Vec::new();
        };
        let mut seen = vec![false; self.tiles.len()];
        let mut region = Vec::new();
        let mut queue = VecDeque::from([start]);
        while let Some(p) = queue.pop_front() {
            let Some(i) = self.index(p) else {
                continue;
            };
            if seen[i] || self.tiles[i] != target {
                continue;
            }
            seen[i] = true;
            region.push(p);
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                queue.push_back(TilePos::new(p.x + dx, p.y + dy));
            }
        }
        region
    }

    /// Sets each corner of `targets` to its height (capped at [`MAX_ELEVATION`]; corners off the map are ignored)
    /// and moves every other corner as little as needed to keep the maximum-slope rule
    /// (`core/map.md > Elevation > Maximum slope`): slopes form outwards on their own, AoE2-style. Two corners
    /// constrain each other when they share a Traversible tile, so the slopes stop at non-Traversible tiles
    /// (cliffs). On a map without violations, targets that keep the rule among themselves (all at one level, or
    /// each one level from where it was) leave none, and an existing violation never gets worse. Contradictory
    /// targets are still set as given. Returns the corners whose height changed, in row order.
    ///
    /// Each corner ends at its old height clamped between the bounds the targets impose: no lower than a target's
    /// height minus the corner's distance from it, and no higher than a target's height plus that distance,
    /// distances counted in steps between corners that share a Traversible tile.
    pub fn set_corner_heights(&mut self, targets: &BTreeMap<CornerPos, u8>) -> Vec<CornerPos> {
        let count = self.heights.len();
        let top = usize::from(MAX_ELEVATION);
        let step = MAX_TRAVERSIBLE_SLOPE;
        // `lo[i]` and `hi[i]` bound corner i; buckets hold the corners whose bound reached that level.
        let mut lo = vec![0u8; count];
        let mut hi = vec![MAX_ELEVATION; count];
        let mut pinned = vec![false; count];
        let mut lo_buckets: Vec<Vec<usize>> = vec![Vec::new(); top + 1];
        let mut hi_buckets: Vec<Vec<usize>> = vec![Vec::new(); top + 1];
        for (&c, &h) in targets {
            let Some(i) = self.corner_index(c) else {
                continue;
            };
            let h = h.min(MAX_ELEVATION);
            (lo[i], hi[i], pinned[i]) = (h, h, true);
            lo_buckets[usize::from(h)].push(i);
            hi_buckets[usize::from(h)].push(i);
        }
        let links = SlopeLinks::new(self);
        // Lower bounds spread downhill from the highest target, upper bounds uphill from the lowest, one level per
        // step (Dial's algorithm: every step costs the same).
        for level in (1..=top).rev() {
            for i in std::mem::take(&mut lo_buckets[level]) {
                if usize::from(lo[i]) != level {
                    continue;
                }
                let bound = lo[i].saturating_sub(step);
                for j in links.of(i) {
                    if !pinned[j] && lo[j] < bound {
                        lo[j] = bound;
                        lo_buckets[usize::from(bound)].push(j);
                    }
                }
            }
        }
        for level in 0..top {
            for i in std::mem::take(&mut hi_buckets[level]) {
                if usize::from(hi[i]) != level {
                    continue;
                }
                let bound = hi[i].saturating_add(step).min(MAX_ELEVATION);
                for j in links.of(i) {
                    if !pinned[j] && hi[j] > bound {
                        hi[j] = bound;
                        hi_buckets[usize::from(bound)].push(j);
                    }
                }
            }
        }
        let mut changed = Vec::new();
        let corner_width = self.width as usize + 1;
        for i in 0..count {
            // A target has lo = hi = its height. `max` after `min` never panics, even between contradictory
            // targets.
            let new = self.heights[i].min(hi[i]).max(lo[i]);
            if new != self.heights[i] {
                self.heights[i] = new;
                changed.push(CornerPos::new((i % corner_width) as i32, (i / corner_width) as i32));
            }
        }
        changed
    }
}

/// Which corners constrain each other's height: those that share a Traversible tile (`core/map.md > Elevation >
/// Maximum slope`). Each corner links to up to 8 neighbours: along the edges of its tiles and across their
/// diagonals.
struct SlopeLinks {
    width: usize,
    height: usize,
    /// Whether each tile is Traversible, row-major.
    traversible: Vec<bool>,
}

impl SlopeLinks {
    fn new(map: &Map) -> Self {
        Self {
            width: map.width as usize,
            height: map.height as usize,
            traversible: map
                .tiles
                .iter()
                .map(|t| map.presets[usize::from(t.preset.0)].traversible)
                .collect(),
        }
    }

    fn traversible(&self, x: isize, y: isize) -> bool {
        x >= 0
            && y >= 0
            && (x as usize) < self.width
            && (y as usize) < self.height
            && self.traversible[y as usize * self.width + x as usize]
    }

    /// The corners linked to corner index `i`.
    fn of(&self, i: usize) -> impl Iterator<Item = usize> + '_ {
        let corner_width = self.width + 1;
        let (x, y) = ((i % corner_width) as isize, (i / corner_width) as isize);
        let (max_x, max_y) = (self.width as isize, self.height as isize);
        [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)]
            .into_iter()
            .filter(move |&(dx, dy)| {
                let (nx, ny) = (x + dx, y + dy);
                if nx < 0 || ny < 0 || nx > max_x || ny > max_y {
                    return false;
                }
                // The tiles that have both corners: one across a diagonal, two along an edge.
                let (tx, ty) = (x.min(nx), y.min(ny));
                match (dx, dy) {
                    (0, _) => self.traversible(x - 1, ty) || self.traversible(x, ty),
                    (_, 0) => self.traversible(tx, y - 1) || self.traversible(tx, y),
                    _ => self.traversible(tx, ty),
                }
            })
            .map(move |(dx, dy)| ((y + dy) as usize) * corner_width + (x + dx) as usize)
    }
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

    fn ascii(map: &Map) -> String {
        map.to_ascii()
    }

    fn targets(list: &[(i32, i32, u8)]) -> BTreeMap<CornerPos, u8> {
        list.iter().map(|&(x, y, h)| (c(x, y), h)).collect()
    }

    #[test]
    fn resize_keeps_the_top_left_and_continues_the_edges() {
        let mut map = Map::from_ascii("~r\nMc\n\n012\n345\n678").unwrap();
        map.resize(3, 3);
        assert_eq!(ascii(&map), "~r.\nMc.\n...\n\n0122\n3455\n6788\n6788\n");
        map.resize(1, 2);
        assert_eq!(ascii(&map), "~\nM\n\n01\n34\n67\n");
        assert_eq!(map.presets().len(), 5, "the preset table is unchanged");
    }

    #[test]
    fn resize_keeps_custom_presets() {
        let mut map = Map::new(2, 1);
        let ash = map.add_preset(Map::default_presets()[0].clone());
        map.set_tile(t(1, 0), Tile::new(ash));
        map.set_corner(c(2, 1), 3);
        map.resize(4, 2);
        assert_eq!(map.tile(t(1, 0)), Some(Tile::new(ash)));
        assert_eq!(map.tile(t(3, 1)), Some(Tile::new(TilePresetId::PLANE)));
        assert_eq!(map.corner(c(2, 1)), Some(3));
        assert_eq!(map.corner(c(4, 2)), Some(3), "the nearest old corner is (2, 1)");
        assert_eq!(map.corner(c(4, 0)), Some(0));
    }

    #[test]
    #[should_panic(expected = "map must not be empty")]
    fn resize_to_nothing_panics() {
        Map::new(2, 2).resize(0, 2);
    }

    #[test]
    fn connected_region_follows_equal_presets_only() {
        // The two water areas touch only diagonally; heights don't matter.
        let map = Map::from_ascii(
            "
            ~~..
            ~.~~
            ..~~

            00000
            00011
            00011
            00000
            ",
        )
        .unwrap();
        let region = map.connected_region(t(0, 0));
        assert_eq!(region, vec![t(0, 0), t(1, 0), t(0, 1)]);
        let right = map.connected_region(t(2, 1));
        assert_eq!(right, vec![t(2, 1), t(3, 1), t(2, 2), t(3, 2)]);
        assert!(map.connected_region(t(4, 0)).is_empty());
        assert!(map.connected_region(t(-1, 0)).is_empty());
    }

    #[test]
    fn connected_region_of_a_flat_map_is_every_tile() {
        let map = Map::new(5, 4);
        let mut region = map.connected_region(t(2, 2));
        assert_eq!(region.len(), 20);
        region.sort();
        region.dedup();
        assert_eq!(region.len(), 20, "each tile once");
    }

    #[test]
    fn raising_a_corner_forms_slopes_around_it() {
        let mut map = Map::new(6, 6);
        let changed = map.set_corner_heights(&targets(&[(3, 3, 3)]));
        assert_eq!(
            ascii(&map).split("\n\n").nth(1).unwrap(),
            "0000000\n0111110\n0122210\n0123210\n0122210\n0111110\n0000000\n"
        );
        assert_eq!(changed.len(), 25);
        assert!(map.too_steep_tiles().is_empty());
        // Lowering it again lowers only what the rule needs.
        let changed = map.set_corner_heights(&targets(&[(3, 3, 0)]));
        assert_eq!(changed.len(), 9);
        assert_eq!(map.corner(c(1, 1)), Some(1));
        assert!(map.too_steep_tiles().is_empty());
    }

    #[test]
    fn slopes_stop_at_non_traversible_tiles() {
        // A cliff column at x = 2: raising the east side leaves the west side alone.
        let mut map = Map::from_ascii("..c..\n..c..\n..c..").unwrap();
        let east = (3..=5).flat_map(|x| (0..=3).map(move |y| (c(x, y), 4)));
        let changed = map.set_corner_heights(&east.collect());
        assert_eq!(changed.len(), 12);
        assert_eq!(
            ascii(&map).split("\n\n").nth(1).unwrap(),
            "000444\n000444\n000444\n000444\n"
        );
        assert!(map.too_steep_tiles().is_empty(), "the cliff takes the drop");
        // Raising the west foot of the cliff spreads west only.
        map.set_corner_heights(&targets(&[(2, 1, 2)]));
        assert_eq!(map.corner(c(1, 1)), Some(1));
        assert_eq!(map.corner(c(3, 1)), Some(4), "across the cliff: untouched");
        assert!(map.too_steep_tiles().is_empty());
    }

    #[test]
    fn heights_are_capped_and_corners_off_the_map_ignored() {
        let mut map = Map::new(2, 2);
        let changed = map.set_corner_heights(&targets(&[(1, 1, 200), (9, 9, 3), (-1, 0, 3)]));
        assert_eq!(map.corner(c(1, 1)), Some(MAX_ELEVATION));
        assert_eq!(map.corner(c(0, 0)), Some(MAX_ELEVATION - 1));
        assert_eq!(changed.len(), 9);
        // Contradictory targets don't panic.
        let mut map = Map::new(1, 1);
        map.set_corner_heights(&targets(&[(0, 0, 0), (1, 1, 9)]));
        assert_eq!((map.corner(c(0, 0)), map.corner(c(1, 1))), (Some(0), Some(9)));
    }
}
