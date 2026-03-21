use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::types::AppState;
use crate::game::world::types::TilePresetEnum;
use super::types::{CursorOverUi, PointerDisplayType, PointerIndicator, UiCameraEntity, ObjectInterfaceState};

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

// --- Pointer indicator ---

/// Offset in pixels from the cursor position (bottom-right) so the indicator
/// doesn't intercept mouse clicks on entities below.
const POINTER_OFFSET_X: f32 = 12.0;
const POINTER_OFFSET_Y: f32 = 12.0;

/// Size of the pointer indicator square in logical pixels.
const POINTER_SIZE: f32 = 16.0;

impl PointerDisplayType {
    /// Returns the indicator color for each pointer display variant.
    pub fn indicator_color(&self) -> Color {
        match self {
            Self::Inactive => Color::srgba(0.5, 0.5, 0.5, 0.3),
            Self::Move => Color::srgba(0.2, 0.9, 0.2, 0.7),
            Self::Attack => Color::srgba(0.9, 0.15, 0.15, 0.7),
            Self::AttackGround => Color::srgba(0.9, 0.3, 0.1, 0.7),
            Self::Patrol => Color::srgba(0.9, 0.7, 0.1, 0.7),
            Self::GatherResources => Color::srgba(0.9, 0.8, 0.1, 0.7),
            Self::ReturnResources => Color::srgba(0.8, 0.7, 0.2, 0.7),
            Self::Enter => Color::srgba(0.1, 0.7, 0.9, 0.7),
        }
    }
}

/// Spawns the pointer indicator UI entity during InGame state setup.
/// Must run after `setup_hud` which creates the `UiCameraEntity` resource.
pub fn spawn_pointer_indicator(mut commands: Commands, ui_cam: Res<UiCameraEntity>) {
    commands.spawn((
        Node {
            width: Val::Px(POINTER_SIZE),
            height: Val::Px(POINTER_SIZE),
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.5, 0.5, 0.5, 0.3)),
        UiTargetCamera(ui_cam.0),
        PointerIndicator,
        Visibility::Hidden,
        DespawnOnExit(AppState::InGame),
    ));
}

/// Updates the pointer indicator appearance and position each frame.
/// Reads the resolved `PointerDisplayType` and positions the indicator
/// near the cursor. Hides the indicator during placement mode or when
/// the cursor is over UI elements.
pub fn update_pointer_display(
    pointer_type: Res<PointerDisplayType>,
    interface_state: Res<ObjectInterfaceState>,
    cursor_over_ui: Res<CursorOverUi>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut indicator: Query<(&mut Node, &mut BackgroundColor, &mut Visibility), With<PointerIndicator>>,
) {
    let Ok((mut node, mut bg_color, mut visibility)) = indicator.single_mut() else {
        return;
    };

    // Get cursor position from the primary window
    let cursor_pos = windows.iter().next().and_then(|w| w.cursor_position());

    // Hide during placement mode, when cursor is over UI, or when cursor is off-window
    if interface_state.is_placement_mode() || cursor_over_ui.0 || cursor_pos.is_none() {
        *visibility = Visibility::Hidden;
        return;
    }

    let pos = cursor_pos.unwrap();

    // Show indicator and position it near cursor
    *visibility = Visibility::Inherited;
    node.left = Val::Px(pos.x + POINTER_OFFSET_X);
    node.top = Val::Px(pos.y + POINTER_OFFSET_Y);

    // Update color based on pointer display type
    *bg_color = BackgroundColor(pointer_type.indicator_color());
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::StructureMenuState;

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

    // --- Pointer indicator tests ---

    #[test]
    fn indicator_color_returns_distinct_colors_for_each_variant() {
        let variants = [
            PointerDisplayType::Inactive,
            PointerDisplayType::Move,
            PointerDisplayType::Attack,
            PointerDisplayType::AttackGround,
            PointerDisplayType::Patrol,
            PointerDisplayType::GatherResources,
            PointerDisplayType::ReturnResources,
            PointerDisplayType::Enter,
        ];

        let colors: Vec<Color> = variants.iter().map(|v| v.indicator_color()).collect();
        for i in 0..colors.len() {
            for j in (i + 1)..colors.len() {
                assert_ne!(
                    colors[i], colors[j],
                    "Variants {:?} and {:?} should have distinct colors",
                    variants[i], variants[j]
                );
            }
        }
    }

    #[test]
    fn inactive_color_is_mostly_transparent() {
        let color = PointerDisplayType::Inactive.indicator_color();
        match color {
            Color::Srgba(c) => {
                assert!(c.alpha < 0.5, "Inactive should be mostly transparent, got alpha={}", c.alpha);
            }
            _ => panic!("Expected Srgba color"),
        }
    }

    #[test]
    fn attack_color_is_red() {
        let color = PointerDisplayType::Attack.indicator_color();
        match color {
            Color::Srgba(c) => {
                assert!(c.red > 0.5, "Attack red channel should be > 0.5");
                assert!(c.green < 0.5, "Attack green channel should be < 0.5");
            }
            _ => panic!("Expected Srgba color"),
        }
    }

    #[test]
    fn move_color_is_green() {
        let color = PointerDisplayType::Move.indicator_color();
        match color {
            Color::Srgba(c) => {
                assert!(c.green > 0.5, "Move green channel should be > 0.5");
                assert!(c.red < 0.5, "Move red channel should be < 0.5");
            }
            _ => panic!("Expected Srgba color"),
        }
    }

    #[test]
    fn enter_color_is_cyan() {
        let color = PointerDisplayType::Enter.indicator_color();
        match color {
            Color::Srgba(c) => {
                assert!(c.blue > 0.5, "Enter blue channel should be > 0.5");
                assert!(c.green > 0.5, "Enter green channel should be > 0.5");
                assert!(c.red < 0.5, "Enter red channel should be < 0.5");
            }
            _ => panic!("Expected Srgba color"),
        }
    }

    #[test]
    fn placement_mode_hides_indicator() {
        let state = ObjectInterfaceState::StructureMenu(
            StructureMenuState::DcAwaitingPlacement,
        );
        assert!(state.is_placement_mode());
    }

    #[test]
    fn default_state_is_not_placement_mode() {
        let state = ObjectInterfaceState::Default;
        assert!(!state.is_placement_mode());
    }
}
