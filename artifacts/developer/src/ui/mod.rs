use bevy::prelude::*;
use crate::types::AppState;
use crate::simulation::types::DiagCategory;

pub mod types;
pub mod utils;
mod hud;
mod command_panel;
pub mod menu;

use types::{ObjectInterfaceState, CommandPanelTarget, CursorOverUi, CursorTarget, SelectedUnitCapabilities};

/// Plugin for HUD (Heads-Up Display) systems
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ObjectInterfaceState>()
            .init_resource::<CommandPanelTarget>()
            .init_resource::<CursorOverUi>()
            .init_resource::<CursorTarget>()
            .init_resource::<SelectedUnitCapabilities>()
            // Menu systems (Menu state)
            .add_systems(OnEnter(AppState::Menu), menu::setup_menu)
            .add_systems(Update, (
                menu::faction_button_click,
                menu::menu_button_hover,
            ).run_if(in_state(AppState::Menu)))
            // In-game HUD systems
            .add_systems(OnEnter(AppState::InGame), hud::setup_hud.after(crate::game::world::faction::setup_player_resources))
            .add_systems(Update, (
                utils::update_cursor_over_ui,
                command_panel::update_cursor_target.after(utils::update_cursor_over_ui),
                hud::update_minimap_system,
                hud::update_selected_units_grid_system.after(command_panel::command_panel_hotkeys),
                hud::selection_portrait_click_system.after(hud::update_selected_units_grid_system),
                hud::update_resource_bar_system,
                command_panel::update_command_panel_state.after(command_panel::update_cursor_target),
                command_panel::rebuild_command_panel_ui.after(command_panel::update_command_panel_state),
                command_panel::handle_command_button_clicks.after(command_panel::rebuild_command_panel_ui),
                command_panel::command_panel_hotkeys,
                command_panel::update_command_panel_progress,
            ).in_set(DiagCategory::UiHud));
    }
}
