//! The overlay's shapes: lines, outlines, fills and dots in view space, laid on the drawn terrain.

use std::collections::BTreeSet;

use bevy::prelude::*;
use sc_sim::{Map, SU_PER_GU, TilePos};

use crate::camera::map_extent_su;
use crate::coords::{drawn_at_height, drawn_position, view_to_world};
use crate::map_render::tile_surface;
use crate::shapes::ColorMesh;

/// Builds the overlay from shapes in view space (where the terrain is drawn), into Bevy world coordinates.
pub(super) struct Overlay<'a> {
    pub map: &'a Map,
    pub mesh: ColorMesh,
    /// Line thickness in space units.
    pub line: f32,
}

impl Overlay<'_> {
    pub fn polygon(&mut self, points: &[Vec2], z: f32, color: Color) {
        let world: Vec<Vec2> = points.iter().map(|p| view_to_world(*p)).collect();
        self.mesh.polygon(&world, z, color);
    }

    pub fn segment(&mut self, a: Vec2, b: Vec2, z: f32, color: Color) {
        self.mesh
            .segment(view_to_world(a), view_to_world(b), self.line, z, color);
    }

    /// Where tile `t` is drawn: its NW, NE, SE and SW corners and its center ([`tile_surface`]).
    fn drawn_tile(&self, t: TilePos) -> [Vec2; 5] {
        tile_surface(self.map, t).map(|(ground, height)| drawn_at_height(ground, height))
    }

    /// Fills tile `t` where the terrain draws it.
    pub fn tile_fill(&mut self, t: TilePos, color: Color) {
        let [nw, ne, se, sw, center] = self.drawn_tile(t);
        for (a, b) in [(nw, ne), (ne, se), (se, sw), (sw, nw)] {
            self.polygon(&[a, b, center], 0.0, color);
        }
    }

    /// Outlines the area `tiles` cover where the terrain draws it: every tile edge without a neighbour in the set.
    pub fn tiles_outline(&mut self, tiles: &BTreeSet<TilePos>, color: Color) {
        for &t in tiles {
            let [nw, ne, se, sw, _] = self.drawn_tile(t);
            for ((dx, dy), a, b) in [((0, -1), nw, ne), ((1, 0), ne, se), ((0, 1), se, sw), ((-1, 0), sw, nw)] {
                let neighbour = t.x.checked_add(dx).zip(t.y.checked_add(dy));
                if !neighbour.is_some_and(|(x, y)| tiles.contains(&TilePos::new(x, y))) {
                    self.segment(a, b, 0.001, color);
                }
            }
        }
    }

    /// The ground segment from `a` to `b`, where the terrain draws it.
    pub fn surface_line(&mut self, a: Vec2, b: Vec2, z: f32, color: Color) {
        for pair in surface_path(self.map, a, b).windows(2) {
            self.segment(pair[0], pair[1], z, color);
        }
    }

    /// Outlines the ground rectangle `rect` where the terrain draws it.
    pub fn surface_outline(&mut self, rect: Rect, color: Color) {
        let corners = rect_corners(rect);
        for i in 0..4 {
            self.surface_line(corners[i], corners[(i + 1) % 4], 0.001, color);
        }
    }

    /// Outlines a rectangle of view space.
    pub fn view_outline(&mut self, rect: Rect, color: Color) {
        let corners = rect_corners(rect);
        for i in 0..4 {
            self.segment(corners[i], corners[(i + 1) % 4], 0.001, color);
        }
    }

    /// A cross at the view point `at`.
    pub fn cross(&mut self, at: Vec2, half: f32, color: Color) {
        self.segment(at - Vec2::new(half, 0.0), at + Vec2::new(half, 0.0), 0.002, color);
        self.segment(at - Vec2::new(0.0, half), at + Vec2::new(0.0, half), 0.002, color);
    }

    /// A square dot `size` across at the view point `at`.
    pub fn dot(&mut self, at: Vec2, size: f32, color: Color) {
        self.polygon(
            &rect_corners(Rect::from_center_size(at, Vec2::splat(size))),
            0.003,
            color,
        );
    }
}

/// A rectangle's corners, around it.
pub(super) fn rect_corners(rect: Rect) -> [Vec2; 4] {
    let (a, b) = (rect.min, rect.max);
    [a, Vec2::new(b.x, a.y), b, Vec2::new(a.x, b.y)]
}

/// The view points along the ground segment from `a` to `b` where its drawn path bends: its ends and its crossings
/// of the tile grid on the map, between which the height along an axis-aligned segment is linear.
pub(super) fn surface_path(map: &Map, a: Vec2, b: Vec2) -> Vec<Vec2> {
    if !a.is_finite() || !b.is_finite() {
        return Vec::new();
    }
    let extent = map_extent_su(map);
    let mut ts = vec![0.0, 1.0];
    for (from, to, size) in [(a.x, b.x, extent.x), (a.y, b.y, extent.y)] {
        if from == to {
            continue;
        }
        // Only grid lines on the map: off it the ground is flat along the segment.
        let (lo, hi) = (from.min(to).max(0.0), from.max(to).min(size));
        for k in (lo / SU_PER_GU).ceil() as i64..=(hi / SU_PER_GU).floor() as i64 {
            let t = (k as f32 * SU_PER_GU - from) / (to - from);
            if t > 0.0 && t < 1.0 {
                ts.push(t);
            }
        }
    }
    ts.sort_by(f32::total_cmp);
    ts.dedup();
    ts.into_iter().map(|t| drawn_position(map, a.lerp(b, t))).collect()
}

/// Whether the ground rectangle `rect` is near enough to the map to draw: a damaged file's spawn may lie anywhere.
pub(super) fn near_map(map: &Map, rect: Rect) -> bool {
    let area = Rect::from_corners(Vec2::ZERO, map_extent_su(map)).inflate(SU_PER_GU * 4.0);
    rect.min.is_finite() && rect.max.is_finite() && !area.intersect(rect).is_empty()
}

pub(super) fn tile_rect(t: TilePos) -> Rect {
    Rect::from_corners(t.corner(), t.corner() + Vec2::splat(SU_PER_GU))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surface_paths_bend_at_the_grid_lines_on_the_map() {
        // A ramp rising 0 → 2 → 4 eastwards over two tiles.
        let map = Map::from_ascii("..\n\n024\n024").unwrap();
        let path = surface_path(&map, Vec2::new(-64.0, 32.0), Vec2::new(256.0, 32.0));
        let xs: Vec<f32> = path.iter().map(|p| p.x).collect();
        assert_eq!(
            xs,
            [-64.0, 0.0, 64.0, 128.0, 256.0],
            "the ends, then the grid lines on the map only"
        );
        assert_eq!(path[1], drawn_position(&map, Vec2::new(0.0, 32.0)));
        assert_eq!(path[3].y, 32.0 - crate::coords::lift(4.0));
        // A damaged file's spawn far away costs no more than the map's size.
        let far = surface_path(&map, Vec2::new(-1e30, 0.0), Vec2::new(1e30, 0.0));
        assert!(far.len() <= 5);
        assert!(surface_path(&map, Vec2::NAN, Vec2::ZERO).is_empty());
        assert!(!near_map(&map, Rect::new(1e12, 1e12, 1e12 + 64.0, 1e12 + 64.0)));
        assert!(near_map(&map, Rect::new(-64.0, -64.0, 0.0, 0.0)));
    }
}
