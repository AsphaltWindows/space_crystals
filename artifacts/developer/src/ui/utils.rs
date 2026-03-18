use bevy::prelude::*;
use crate::game::world::types::TilePresetEnum;
use super::types::CursorOverUi;

/// Get color for health bar based on health percentage
pub fn get_health_color(health_percent: f32) -> Color {
    if health_percent > 0.6 {
        Color::srgb(0.2, 0.8, 0.2) // Green
    } else if health_percent > 0.3 {
        Color::srgb(0.8, 0.8, 0.2) // Yellow
    } else {
        Color::srgb(0.8, 0.2, 0.2) // Red
    }
}

/// System that checks if the cursor is hovering over any UI node.
/// Sets `CursorOverUi(true)` when any `Interaction` component reports
/// `Hovered` or `Pressed`, otherwise sets it to `false`.
/// Must run before world-click systems (selection, drag-box, right-click move).
pub fn update_cursor_over_ui(
    interactions: Query<&Interaction, With<Node>>,
    mut cursor_over_ui: ResMut<CursorOverUi>,
) {
    cursor_over_ui.0 = interactions.iter().any(|interaction| {
        matches!(interaction, Interaction::Hovered | Interaction::Pressed)
    });
}

/// Ray-AABB intersection test (slab method).
/// Returns Some(t) with the distance along the ray to the intersection point,
/// or None if the ray misses the AABB.
pub fn ray_aabb_intersect(ray_origin: Vec3, ray_dir: Vec3, aabb_min: Vec3, aabb_max: Vec3) -> Option<f32> {
    let inv_dir = Vec3::new(
        if ray_dir.x.abs() > f32::EPSILON { 1.0 / ray_dir.x } else { f32::MAX * ray_dir.x.signum() },
        if ray_dir.y.abs() > f32::EPSILON { 1.0 / ray_dir.y } else { f32::MAX * ray_dir.y.signum() },
        if ray_dir.z.abs() > f32::EPSILON { 1.0 / ray_dir.z } else { f32::MAX * ray_dir.z.signum() },
    );

    let t1 = (aabb_min - ray_origin) * inv_dir;
    let t2 = (aabb_max - ray_origin) * inv_dir;

    let t_min = t1.min(t2);
    let t_max = t1.max(t2);

    let t_enter = t_min.x.max(t_min.y).max(t_min.z);
    let t_exit = t_max.x.min(t_max.y).min(t_max.z);

    if t_enter <= t_exit && t_exit >= 0.0 {
        Some(t_enter.max(0.0))
    } else {
        None
    }
}

/// Get color for tile type on minimap
pub fn get_tile_color(tile_type: &TilePresetEnum) -> Color {
    match tile_type {
        TilePresetEnum::Plane => Color::srgb(0.3, 0.6, 0.3),
        TilePresetEnum::RuggedTerrain => Color::srgb(0.5, 0.4, 0.3),
        TilePresetEnum::Cliff => Color::srgb(0.4, 0.4, 0.4),
        TilePresetEnum::Mountain => Color::srgb(0.5, 0.5, 0.5),
        TilePresetEnum::Water => Color::srgb(0.2, 0.3, 0.6),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ray_aabb_direct_hit() {
        // Ray pointing straight down at a box centered at origin
        let origin = Vec3::new(0.0, 10.0, 0.0);
        let dir = Vec3::new(0.0, -1.0, 0.0);
        let aabb_min = Vec3::new(-1.0, -1.0, -1.0);
        let aabb_max = Vec3::new(1.0, 1.0, 1.0);
        let result = ray_aabb_intersect(origin, dir, aabb_min, aabb_max);
        assert!(result.is_some());
        let t = result.unwrap();
        assert!((t - 9.0).abs() < 0.001); // hits at y=1 → t=9
    }

    #[test]
    fn ray_aabb_miss() {
        // Ray pointing away from the box
        let origin = Vec3::new(0.0, 10.0, 0.0);
        let dir = Vec3::new(0.0, 1.0, 0.0); // pointing UP
        let aabb_min = Vec3::new(-1.0, -1.0, -1.0);
        let aabb_max = Vec3::new(1.0, 1.0, 1.0);
        let result = ray_aabb_intersect(origin, dir, aabb_min, aabb_max);
        assert!(result.is_none());
    }

    #[test]
    fn ray_aabb_miss_lateral() {
        // Ray going sideways past the box
        let origin = Vec3::new(5.0, 5.0, 0.0);
        let dir = Vec3::new(0.0, -1.0, 0.0);
        let aabb_min = Vec3::new(-1.0, -1.0, -1.0);
        let aabb_max = Vec3::new(1.0, 1.0, 1.0);
        let result = ray_aabb_intersect(origin, dir, aabb_min, aabb_max);
        assert!(result.is_none());
    }

    #[test]
    fn ray_aabb_angled_hit() {
        // Angled ray hitting a box (similar to RTS camera angle)
        let origin = Vec3::new(0.0, 40.0, 25.0);
        let target = Vec3::new(0.0, 0.5, 0.0); // center of a unit at ground level
        let dir = (target - origin).normalize();
        let aabb_min = Vec3::new(-0.8, 0.0, -0.8); // padded unit bounds
        let aabb_max = Vec3::new(0.8, 1.0, 0.8);
        let result = ray_aabb_intersect(origin, dir, aabb_min, aabb_max);
        assert!(result.is_some());
    }

    #[test]
    fn ray_aabb_origin_inside_box() {
        // Ray origin is inside the box
        let origin = Vec3::new(0.0, 0.0, 0.0);
        let dir = Vec3::new(1.0, 0.0, 0.0);
        let aabb_min = Vec3::new(-1.0, -1.0, -1.0);
        let aabb_max = Vec3::new(1.0, 1.0, 1.0);
        let result = ray_aabb_intersect(origin, dir, aabb_min, aabb_max);
        assert!(result.is_some());
        assert!(result.unwrap().abs() < 0.001); // t=0 since origin is inside
    }

    #[test]
    fn ray_aabb_axis_aligned() {
        // Ray along X axis hitting a box
        let origin = Vec3::new(-10.0, 0.0, 0.0);
        let dir = Vec3::new(1.0, 0.0, 0.0);
        let aabb_min = Vec3::new(-1.0, -1.0, -1.0);
        let aabb_max = Vec3::new(1.0, 1.0, 1.0);
        let result = ray_aabb_intersect(origin, dir, aabb_min, aabb_max);
        assert!(result.is_some());
        let t = result.unwrap();
        assert!((t - 9.0).abs() < 0.001); // hits at x=-1 → t=9
    }

    #[test]
    fn ray_aabb_near_miss_edge() {
        // Ray just barely missing the edge of the box
        let origin = Vec3::new(1.01, 10.0, 0.0);
        let dir = Vec3::new(0.0, -1.0, 0.0);
        let aabb_min = Vec3::new(-1.0, -1.0, -1.0);
        let aabb_max = Vec3::new(1.0, 1.0, 1.0);
        let result = ray_aabb_intersect(origin, dir, aabb_min, aabb_max);
        assert!(result.is_none());
    }
}
