//! Terrain height queries (`core/map.md > Elevation`): the height at any point, elevation levels, flat and sloped
//! tiles, and the maximum-slope rule.

use glam::Vec2;

use super::{MAX_TRAVERSIBLE_SLOPE, Map};
use crate::units::{SU_PER_GU, TilePos};

impl Map {
    /// The heights of a tile's corners, north-west, north-east, south-west, south-east.
    pub fn tile_corner_heights(&self, t: TilePos) -> Option<[u8; 4]> {
        if !self.in_bounds(t) {
            return None;
        }
        // In bounds, so all four corners exist.
        Some(t.corners().map(|c| self.corner(c).unwrap_or(0)))
    }

    /// `core/map.md > Elevation > Flat and sloped tiles`: the tile's level if all four corners have the same
    /// height, `None` for a sloped tile or one off the map.
    pub fn tile_level(&self, t: TilePos) -> Option<u8> {
        let [a, b, c, d] = self.tile_corner_heights(t)?;
        (a == b && b == c && c == d).then_some(a)
    }

    /// Whether the tile exists and is flat (`core/map.md > Elevation > Flat and sloped tiles`).
    pub fn is_flat(&self, t: TilePos) -> bool {
        self.tile_level(t).is_some()
    }

    /// The tile's slope: its highest corner minus its lowest (0 for a flat tile).
    pub fn tile_slope(&self, t: TilePos) -> Option<u8> {
        let heights = self.tile_corner_heights(t)?;
        let (min, max) = heights.iter().fold((u8::MAX, 0), |(lo, hi), &h| (lo.min(h), hi.max(h)));
        Some(max - min)
    }

    /// `core/map.md > Elevation > Maximum slope`: whether the tile breaks the rule, i.e. it is Traversible and its
    /// corners differ by more than [`MAX_TRAVERSIBLE_SLOPE`].
    pub fn is_too_steep(&self, t: TilePos) -> bool {
        self.tile_preset(t).is_some_and(|p| p.traversible) && self.tile_slope(t).unwrap_or(0) > MAX_TRAVERSIBLE_SLOPE
    }

    /// Every tile that breaks the maximum-slope rule ([`Map::is_too_steep`]) with its slope, row by row.
    pub fn too_steep_tiles(&self) -> Vec<(TilePos, u8)> {
        (0..self.height as i32)
            .flat_map(|y| (0..self.width as i32).map(move |x| TilePos::new(x, y)))
            .filter(|&t| self.is_too_steep(t))
            .map(|t| (t, self.tile_slope(t).unwrap_or(0)))
            .collect()
    }

    /// `core/map.md > Elevation > Terrain height at a point`: the height at `p` (space units), interpolated
    /// bilinearly from the corners of the tile containing it, so it is continuous across tile edges. Flat tiles
    /// give their level exactly. A point off the map takes the height of the nearest point on it, and a
    /// non-finite coordinate counts as 0.
    pub fn height_at(&self, p: Vec2) -> f32 {
        let (w, h) = (self.width as f32, self.height as f32);
        // `max` before `min` turns NaN into 0.
        let gx = (p.x / SU_PER_GU).max(0.0).min(w);
        let gy = (p.y / SU_PER_GU).max(0.0).min(h);
        // The tile containing the point; the far edges belong to the last row and column.
        let tx = (gx.floor() as i32).min(self.width as i32 - 1);
        let ty = (gy.floor() as i32).min(self.height as i32 - 1);
        let (u, v) = (gx - tx as f32, gy - ty as f32);
        let [nw, ne, sw, se] = self
            .tile_corner_heights(TilePos::new(tx, ty))
            .unwrap_or([0; 4])
            .map(f32::from);
        // Written from the north-west corner so that a flat tile gives its level exactly, whatever u and v are.
        nw + (ne - nw) * u + (sw - nw) * v + (nw - ne - sw + se) * u * v
    }

    /// `core/map.md > Elevation > Elevation level`: the terrain height at `p`, rounded down.
    pub fn elevation_level(&self, p: Vec2) -> u8 {
        self.height_at(p).floor() as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::{Tile, TilePresetId};
    use crate::units::CornerPos;

    /// A 3×2 map: a ramp from 0 (west) to 2 (east) on the north row, flat at 2 in the south-east.
    fn ramp() -> Map {
        Map::from_ascii(
            "
            ...
            c..

            0122
            0122
            0222
            ",
        )
        .unwrap()
    }

    #[test]
    fn interpolation_hits_the_corners_and_is_continuous_across_tile_edges() {
        let map = ramp();
        for y in 0..=2 {
            for x in 0..=3 {
                let c = CornerPos::new(x, y);
                assert_eq!(map.height_at(c.pos()), f32::from(map.corner(c).unwrap()), "{c:?}");
            }
        }
        // Along every vertical and horizontal tile edge, the heights just either side agree.
        let e = 1e-3;
        for i in 0..=96 {
            let along = i as f32 * 2.0;
            for edge in [64.0, 128.0] {
                let p = Vec2::new(edge, along);
                let (west, east) = (map.height_at(p - Vec2::X * e), map.height_at(p + Vec2::X * e));
                assert!((west - east).abs() < 1e-3, "x edge at {p}: {west} vs {east}");
            }
            let p = Vec2::new(along * 1.5, 64.0);
            let (north, south) = (map.height_at(p - Vec2::Y * e), map.height_at(p + Vec2::Y * e));
            assert!((north - south).abs() < 1e-3, "y edge at {p}: {north} vs {south}");
        }
        // Halfway up the ramp.
        assert_eq!(map.height_at(Vec2::new(32.0, 10.0)), 0.5);
        assert_eq!(map.height_at(Vec2::new(96.0, 40.0)), 1.5);
    }

    #[test]
    fn the_level_is_the_height_rounded_down() {
        let map = ramp();
        assert_eq!(map.elevation_level(Vec2::new(63.0, 10.0)), 0, "just below 1");
        assert_eq!(map.elevation_level(Vec2::new(64.0, 10.0)), 1, "exactly 1");
        assert_eq!(map.elevation_level(Vec2::new(127.9, 10.0)), 1);
        assert_eq!(map.elevation_level(Vec2::new(150.0, 100.0)), 2, "the flat part");
        // Flat tiles give their level exactly, everywhere on them.
        let mut flat = Map::new(2, 2);
        for c in TilePos::new(1, 1).corners() {
            flat.set_corner(c, 3);
        }
        for i in 0..64 {
            let p = TilePos::new(1, 1).corner() + Vec2::new(i as f32 * 0.997, 63.99 - i as f32 * 0.31);
            assert_eq!(flat.height_at(p), 3.0, "{p}");
        }
    }

    #[test]
    fn points_off_the_map_take_the_nearest_height() {
        let map = ramp();
        assert_eq!(map.height_at(Vec2::new(500.0, 10.0)), 2.0);
        assert_eq!(map.height_at(Vec2::new(-50.0, 10.0)), 0.0);
        assert_eq!(map.height_at(Vec2::new(192.0, 128.0)), 2.0, "the far corner");
        assert_eq!(map.height_at(Vec2::new(f32::NAN, 0.0)), 0.0);
        assert_eq!(map.height_at(Vec2::splat(f32::INFINITY)), 2.0);
        assert_eq!(map.elevation_level(Vec2::splat(f32::NEG_INFINITY)), 0);
    }

    #[test]
    fn flatness_slope_and_steepness() {
        let map = ramp();
        let t = TilePos::new;
        assert_eq!(map.tile_corner_heights(t(1, 0)), Some([1, 2, 1, 2]));
        assert_eq!(map.tile_level(t(2, 0)), Some(2));
        assert!(map.is_flat(t(2, 1)));
        assert!(!map.is_flat(t(1, 0)));
        assert_eq!(map.tile_level(t(0, 0)), None);
        assert_eq!(map.tile_slope(t(0, 0)), Some(1));
        assert_eq!(map.tile_slope(t(0, 1)), Some(2), "the cliff");
        assert_eq!(map.tile_slope(t(3, 0)), None, "off the map");
        assert!(!map.is_flat(t(-1, 0)));
        // The cliff may be steep; a Traversible tile may not.
        assert!(map.too_steep_tiles().is_empty());
        let mut steep = map.clone();
        steep.set_tile(t(0, 1), Tile::new(TilePresetId::RUGGED));
        assert!(steep.is_too_steep(t(0, 1)));
        assert_eq!(steep.too_steep_tiles(), vec![(t(0, 1), 2)]);
    }
}
