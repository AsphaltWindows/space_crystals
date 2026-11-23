//! Map editing helpers for tools such as the map editor: resizing and flood-fill regions.

use std::collections::VecDeque;

use super::{Map, Tile, TilePresetId};
use crate::units::TilePos;

impl Map {
    /// Resizes the map to `width × height`, anchored at the top-left corner: tiles that still fit keep their
    /// preset and elevation, new tiles are Plane at elevation 0, and the preset table stays as it is.
    ///
    /// Panics if a side is 0, like [`Map::new`].
    pub fn resize(&mut self, width: u32, height: u32) {
        assert!(width > 0 && height > 0, "map must not be empty");
        let (new_w, new_h) = (width as usize, height as usize);
        let (old_w, old_h) = (self.width as usize, self.height as usize);
        let mut tiles = vec![Tile::new(TilePresetId::PLANE, 0); new_w * new_h];
        for y in 0..new_h.min(old_h) {
            let keep = new_w.min(old_w);
            tiles[y * new_w..y * new_w + keep].copy_from_slice(&self.tiles[y * old_w..y * old_w + keep]);
        }
        self.width = width;
        self.height = height;
        self.tiles = tiles;
    }

    /// The region a flood fill started at `start` covers: every tile 4-connected to `start` through tiles equal
    /// to its tile (same preset and elevation). `start` comes first, then the rest in breadth-first order, with
    /// neighbours visited east, west, south, north. Empty if `start` is off the map.
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ascii(map: &Map) -> String {
        map.to_ascii()
    }

    #[test]
    fn resize_keeps_the_top_left_and_fills_with_plane() {
        let mut map = Map::from_ascii("~r\nMc\n\n12\n34").unwrap();
        map.resize(3, 3);
        assert_eq!(ascii(&map), "~r.\nMc.\n...\n\n120\n340\n000\n");
        map.resize(1, 2);
        assert_eq!(ascii(&map), "~\nM\n\n1\n3\n");
        assert_eq!(map.presets().len(), 5, "the preset table is unchanged");
    }

    #[test]
    fn resize_keeps_custom_presets() {
        let mut map = Map::new(2, 1);
        let ash = map.add_preset(Map::default_presets()[0].clone());
        map.set_tile(TilePos::new(1, 0), Tile::new(ash, 3));
        map.resize(4, 2);
        assert_eq!(map.tile(TilePos::new(1, 0)), Some(Tile::new(ash, 3)));
        assert_eq!(map.tile(TilePos::new(3, 1)), Some(Tile::new(TilePresetId::PLANE, 0)));
    }

    #[test]
    #[should_panic(expected = "map must not be empty")]
    fn resize_to_nothing_panics() {
        Map::new(2, 2).resize(0, 2);
    }

    #[test]
    fn connected_region_follows_equal_tiles_only() {
        // The two water areas touch only diagonally; the raised water tile is a different tile.
        let map = Map::from_ascii(
            "
            ~~..
            ~.~~
            ..~~

            0000
            0001
            0000
            ",
        )
        .unwrap();
        let region = map.connected_region(TilePos::new(0, 0));
        assert_eq!(region, vec![TilePos::new(0, 0), TilePos::new(1, 0), TilePos::new(0, 1)]);
        let right = map.connected_region(TilePos::new(2, 1));
        assert_eq!(right, vec![TilePos::new(2, 1), TilePos::new(2, 2), TilePos::new(3, 2)]);
        assert_eq!(map.connected_region(TilePos::new(3, 1)), vec![TilePos::new(3, 1)]);
        assert!(map.connected_region(TilePos::new(4, 0)).is_empty());
        assert!(map.connected_region(TilePos::new(-1, 0)).is_empty());
    }

    #[test]
    fn connected_region_of_a_flat_map_is_every_tile() {
        let map = Map::new(5, 4);
        let mut region = map.connected_region(TilePos::new(2, 2));
        assert_eq!(region.len(), 20);
        region.sort();
        region.dedup();
        assert_eq!(region.len(), 20, "each tile once");
    }
}
