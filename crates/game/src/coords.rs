//! Sim ↔ Bevy coordinates and draw order (`ARCHITECTURE.md > Client architecture > Visuals`).
//!
//! 1 Bevy world unit = 1 space unit. The sim's +y points south, Bevy's +y points up, so `bevy_y = -sim_y`.
//! Convert only through these helpers, never ad hoc.

use bevy::math::{Quat, Vec2, Vec3};

/// Sim position (space units, +y south) → Bevy world position (+y up).
pub fn sim_to_world(p: Vec2) -> Vec2 {
    Vec2::new(p.x, -p.y)
}

/// Bevy world position (+y up) → sim position (space units, +y south).
pub fn world_to_sim(p: Vec2) -> Vec2 {
    Vec2::new(p.x, -p.y)
}

/// A sim facing θ (0 = east, positive = clockwise on screen) is a Bevy rotation of −θ about z.
pub fn facing_to_rotation(facing: f32) -> Quat {
    Quat::from_rotation_z(-facing)
}

/// Interpolates a facing along the shortest arc.
pub fn lerp_facing(from: f32, to: f32, alpha: f32) -> f32 {
    from + sc_sim::math::wrap_angle(to - from) * alpha
}

/// Draw layers in StarCraft 1 order, back to front. Each has its own z band; within a band, z grows with sim y
/// so that southern objects draw in front.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DrawLayer {
    Terrain,
    GroundDecals,
    GroundObjects,
    Air,
    Overlays,
}

/// Distance between the bases of adjacent z bands.
const Z_BAND: f32 = 100.0;
/// z added per space unit of sim y inside a band. A 1024-tile-tall map spans 65.5 z, which stays inside a band.
const Z_PER_SU: f32 = 0.001;

impl DrawLayer {
    /// The z of this layer's northern edge (sim y = 0). The 2D camera sees z in (-1000, 1000).
    pub fn base_z(self) -> f32 {
        let index = match self {
            DrawLayer::Terrain => 0.0,
            DrawLayer::GroundDecals => 1.0,
            DrawLayer::GroundObjects => 2.0,
            DrawLayer::Air => 3.0,
            DrawLayer::Overlays => 4.0,
        };
        -500.0 + index * Z_BAND
    }

    /// The y-sorted z of something anchored at sim y `sim_y` (its ground position).
    pub fn z_at(self, sim_y: f32) -> f32 {
        self.base_z() + sim_y * Z_PER_SU
    }
}

/// Translation of a visual anchored at sim position `p` in `layer`.
pub fn anchored_translation(layer: DrawLayer, p: Vec2) -> Vec3 {
    sim_to_world(p).extend(layer.z_at(p.y))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn y_is_flipped_both_ways() {
        let p = Vec2::new(100.0, 250.0);
        assert_eq!(sim_to_world(p), Vec2::new(100.0, -250.0));
        assert_eq!(world_to_sim(sim_to_world(p)), p);
    }

    #[test]
    fn southern_objects_draw_in_front_and_layers_stack() {
        let north = DrawLayer::GroundObjects.z_at(64.0);
        let south = DrawLayer::GroundObjects.z_at(640.0);
        assert!(south > north);
        // The deepest point of a huge map stays below the next layer.
        assert!(DrawLayer::GroundObjects.z_at(1024.0 * 64.0) < DrawLayer::Air.base_z());
        assert!(DrawLayer::Terrain.base_z() > -1000.0 && DrawLayer::Overlays.z_at(65_536.0) < 1000.0);
    }

    #[test]
    fn facing_turns_clockwise_on_screen() {
        // Sim facing +90° points south (+y sim) = down on screen = -y in Bevy.
        let v = facing_to_rotation(std::f32::consts::FRAC_PI_2) * Vec3::X;
        assert!((v - Vec3::NEG_Y).length() < 1e-6);
    }

    #[test]
    fn facing_interpolates_along_the_shortest_arc() {
        let pi = std::f32::consts::PI;
        let mid = lerp_facing(pi - 0.1, -pi + 0.1, 0.5);
        assert!((sc_sim::math::wrap_angle(mid) - pi).abs() < 1e-5);
    }
}
