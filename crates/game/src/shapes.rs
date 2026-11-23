//! A builder for flat-coloured placeholder meshes: vertex colours drawn with one shared white `ColorMaterial`.
//!
//! Coordinates are Bevy's (+y up). Callers holding sim positions convert them with `coords::sim_to_world` first.
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

    /// One line segment (line meshes only).
    pub fn line(&mut self, a: Vec2, b: Vec2, z: f32, color: Color) {
        debug_assert_eq!(self.topology, PrimitiveTopology::LineList);
        let i = self.vertex(a, z, color);
        self.vertex(b, z, color);
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
        mesh.line(Vec2::Y, Vec2::ONE, 0.0, Color::WHITE);
        assert_eq!(mesh.indices, vec![0, 1, 2, 3]);
        assert_eq!(mesh.into_mesh().count_vertices(), 4);
    }
}
