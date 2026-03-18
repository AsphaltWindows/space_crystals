use bevy::prelude::*;
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
                // Phase 1: Rebuild occupancy map from current positions (must run first)
                systems::core::rebuild_occupancy_map,
                // Phase 2: Behavior systems — write to action channels
                (
                    systems::behaviors::moving_to_location_system,
                    systems::behaviors::moving_to_object_system,
                    systems::behaviors::reversing_to_location_system,
                    systems::behaviors::stopping_behavior_system,
                    systems::behaviors::entering_tunnel_behavior_system,
                    systems::behaviors::building_behavior_system,
                    systems::behaviors::gathering_resource_behavior_system,
                    systems::behaviors::dropping_off_resources_behavior_system,
                    systems::behaviors::building_tunnel_behavior_system,
                ).after(systems::core::rebuild_occupancy_map),
                // Phase 3: Movement and collision systems — read occupancy map + action channels
                (
                    systems::core::unit_movement_system,
                    systems::core::turn_rate_movement_system,
                    systems::core::collision_repath_system,
                    systems::core::unit_rotation_system,
                    systems::core::air_unit_separation_system,
                ).after(systems::core::rebuild_occupancy_map),
                // Phase 4: Sync grid positions from transforms (after movement)
                systems::core::grid_position_sync_system
                    .after(systems::core::unit_movement_system)
                    .after(systems::core::turn_rate_movement_system),
                // Phase 5: Visual feedback — runs after commands are issued
                systems::core::unit_selection_display,
                systems::core::right_click_move_command,
                systems::core::set_rally_point_click_system,
                systems::core::schedule_deliveries_click_system,
                systems::core::command_indicator_sync_system
                    .after(systems::core::right_click_move_command),
            ).in_set(DiagCategory::Movement));
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
            ).in_set(DiagCategory::Commands));
    }
}
