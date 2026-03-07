use bevy::prelude::*;
use crate::types::AppState;
use crate::simulation::types::DiagCategory;

pub mod types;
pub mod utils;
mod systems;
pub mod pathfinding;

/// Plugin for unit-related systems
pub struct UnitsPlugin;

impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<types::OccupancyMap>();
        app.add_systems(Update, (
                systems::core::rebuild_occupancy_map,
                systems::core::grid_position_sync_system,
                systems::core::unit_selection_display,
                systems::core::right_click_move_command,
                systems::core::unit_movement_system,
                systems::core::unit_rotation_system,
                systems::core::turn_rate_movement_system,
                systems::core::collision_repath_system,
                // Behavior systems — write to action channels before legacy systems read them
                systems::behaviors::moving_to_location_system,
                systems::behaviors::moving_to_object_system,
                systems::behaviors::reversing_to_location_system,
                systems::behaviors::stopping_behavior_system,
                systems::behaviors::entering_tunnel_behavior_system,
                systems::behaviors::building_behavior_system,
                systems::core::command_indicator_sync_system,
                systems::core::air_unit_separation_system,
            ).in_set(DiagCategory::Movement)
             .run_if(in_state(AppState::InGame)));
    }
}

/// Plugin for command system
pub struct CommandsPlugin;

impl Plugin for CommandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
                systems::commands::command_input_system,
                systems::commands::hold_position_system,
                systems::commands::stop_command_system,
                systems::commands::patrol_command_system,
            ).in_set(DiagCategory::Commands)
             .run_if(in_state(AppState::InGame)));
    }
}
