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
