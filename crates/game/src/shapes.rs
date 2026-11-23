//! A builder for flat-coloured placeholder meshes: vertex colours drawn with one shared white `ColorMaterial`.
//!
//! Coordinates are Bevy's (+y up). Callers holding view positions convert them with `coords::view_to_world` first.
//! Layers inside one mesh use small local z offsets: the 2D depth test lets higher z win.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

/// Accumulates vertex-coloured triangles or line segments.
pub struct ColorMesh {
    topology: PrimitiveTopology,
    positions: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

impl ColorMesh {
    pub fn triangles() -> Self {
        Self::new(PrimitiveTopology::TriangleList)
    }

    /// One-pixel lines, whatever the zoom.
    pub fn lines() -> Self {
        Self::new(PrimitiveTopology::LineList)
    }

    fn new(topology: PrimitiveTopology) -> Self {
        Self {
            topology,
            positions: Vec::new(),
            colors: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    fn vertex(&mut self, p: Vec2, z: f32, color: Color) -> u32 {
        let index = self.positions.len() as u32;
        self.positions.push([p.x, p.y, z]);
        self.colors.push(color.to_linear().to_f32_array());
        index
    }

    /// A filled convex polygon, as a fan around its first point.
    pub fn polygon(&mut self, points: &[Vec2], z: f32, color: Color) {
        debug_assert_eq!(self.topology, PrimitiveTopology::TriangleList);
        let first = self.positions.len() as u32;
        for &p in points {
            self.vertex(p, z, color);
        }
        for i in 1..points.len().saturating_sub(1) as u32 {
            self.indices.extend([first, first + i, first + i + 1]);
        }
    }

    /// An axis-aligned rectangle between two opposite corners.
    pub fn rect(&mut self, a: Vec2, b: Vec2, z: f32, color: Color) {
        self.polygon(&[a, Vec2::new(b.x, a.y), b, Vec2::new(a.x, b.y)], z, color);
    }

    /// A triangle whose corners have their own z (the terrain's depth grows southwards).
    pub fn triangle(&mut self, corners: [Vec3; 3], color: Color) {
        debug_assert_eq!(self.topology, PrimitiveTopology::TriangleList);
        for p in corners {
            let i = self.vertex(p.truncate(), p.z, color);
            self.indices.push(i);
        }
    }

    /// A convex quadrilateral whose corners have their own z, given in order around it.
    pub fn quad(&mut self, corners: [Vec3; 4], color: Color) {
        debug_assert_eq!(self.topology, PrimitiveTopology::TriangleList);
        let first = self.positions.len() as u32;
        for p in corners {
            self.vertex(p.truncate(), p.z, color);
        }
        self.indices
            .extend([first, first + 1, first + 2, first, first + 2, first + 3]);
    }

    /// A segment from `a` to `b` drawn `width` thick, as a quad (triangle meshes).
    pub fn segment(&mut self, a: Vec2, b: Vec2, width: f32, z: f32, color: Color) {
        let along = b - a;
        let side = if along.length_squared() > 0.0 {
            along.normalize().perp() * (width / 2.0)
        } else {
            Vec2::new(width / 2.0, 0.0)
        };
        self.polygon(&[a - side, b - side, b + side, a + side], z, color);
    }

    /// One line segment (line meshes only).
    pub fn line(&mut self, a: Vec2, b: Vec2, z: f32, color: Color) {
        self.line_3d(a.extend(z), b.extend(z), color);
    }

    /// One line segment whose ends have their own z (line meshes only).
    pub fn line_3d(&mut self, a: Vec3, b: Vec3, color: Color) {
        debug_assert_eq!(self.topology, PrimitiveTopology::LineList);
        let i = self.vertex(a.truncate(), a.z, color);
        self.vertex(b.truncate(), b.z, color);
        self.indices.extend([i, i + 1]);
    }

    pub fn into_mesh(self) -> Mesh {
        Mesh::new(self.topology, RenderAssetUsages::default())
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colors)
            .with_inserted_indices(Indices::U32(self.indices))
    }
}

/// The white, opaque material that shows vertex colours as they are.
pub fn vertex_color_material() -> ColorMaterial {
    ColorMaterial::from_color(Color::WHITE)
}

/// Mixes `amount` (0–1) of white into `color`.
pub fn lighten(color: Srgba, amount: f32) -> Srgba {
    color.mix(&Srgba::WHITE, amount.clamp(0.0, 1.0))
}

/// Scales `color`'s channels by `factor` (0–1).
pub fn darken(color: Srgba, factor: f32) -> Srgba {
    let f = factor.clamp(0.0, 1.0);
    Srgba::new(color.red * f, color.green * f, color.blue * f, color.alpha)
}

/// Shades `color` by a light factor: below 1 darkens it, above 1 brightens it (each channel capped at 1).
pub fn shade(color: Srgba, factor: f32) -> Srgba {
    let f = if factor.is_finite() { factor.max(0.0) } else { 1.0 };
    let channel = |c: f32| (c * f).min(1.0);
    Srgba::new(
        channel(color.red),
        channel(color.green),
        channel(color.blue),
        color.alpha,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn polygons_fan_into_triangles() {
        let mut mesh = ColorMesh::triangles();
        mesh.rect(Vec2::ZERO, Vec2::ONE, 0.0, Color::WHITE);
        mesh.polygon(
            &[Vec2::ZERO, Vec2::X, Vec2::ONE, Vec2::Y, Vec2::NEG_X],
            0.0,
            Color::BLACK,
        );
        assert_eq!(mesh.vertex_count(), 4 + 5);
        assert_eq!(mesh.indices.len(), 3 * (2 + 3));
        assert!(mesh.indices.iter().all(|&i| (i as usize) < mesh.vertex_count()));
    }

    #[test]
    fn lines_pair_their_vertices() {
        let mut mesh = ColorMesh::lines();
        mesh.line(Vec2::ZERO, Vec2::X, 0.0, Color::WHITE);
        mesh.line_3d(Vec3::Y, Vec3::new(1.0, 1.0, 0.5), Color::WHITE);
        assert_eq!(mesh.indices, vec![0, 1, 2, 3]);
        assert_eq!(mesh.positions[3], [1.0, 1.0, 0.5]);
        assert_eq!(mesh.into_mesh().count_vertices(), 4);
    }

    #[test]
    fn triangles_quads_and_segments_keep_their_corners() {
        let mut mesh = ColorMesh::triangles();
        mesh.triangle(
            [Vec3::ZERO, Vec3::new(1.0, 0.0, 0.1), Vec3::new(0.0, 1.0, 0.2)],
            Color::WHITE,
        );
        mesh.quad([Vec3::ZERO, Vec3::X, Vec3::new(1.0, 1.0, 0.3), Vec3::Y], Color::WHITE);
        assert_eq!(mesh.vertex_count(), 3 + 4);
        assert_eq!(mesh.indices, vec![0, 1, 2, 3, 4, 5, 3, 5, 6]);
        assert_eq!(mesh.positions[5][2], 0.3);
        // A segment 2 wide along x spans y from -1 to 1.
        let mut mesh = ColorMesh::triangles();
        mesh.segment(Vec2::ZERO, Vec2::new(10.0, 0.0), 2.0, 0.0, Color::WHITE);
        let ys: Vec<f32> = mesh.positions.iter().map(|p| p[1]).collect();
        assert_eq!(ys, vec![-1.0, -1.0, 1.0, 1.0]);
        // A zero-length segment still makes a valid (degenerate) quad.
        mesh.segment(Vec2::ONE, Vec2::ONE, 2.0, 0.0, Color::WHITE);
        assert!(mesh.positions.iter().flatten().all(|v| v.is_finite()));
    }

    #[test]
    fn shading_darkens_and_brightens_within_range() {
        let c = Srgba::rgb(0.4, 0.5, 0.6);
        assert_eq!(shade(c, 1.0), c);
        assert!(shade(c, 0.5).red < c.red);
        let bright = shade(c, 2.0);
        assert!(bright.red > c.red && bright.blue == 1.0);
        assert_eq!(shade(c, f32::NAN), c);
    }
}
