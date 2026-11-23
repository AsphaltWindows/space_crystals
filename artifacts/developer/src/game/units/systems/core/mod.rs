mod utility;
mod command_dispatch;
mod rally;
mod legacy_movement;
mod occupancy;
mod indicators;
mod channel_locomotion;

// Re-export all public items so external `use systems::core::*` continues to work.
pub use utility::{grid_position_sync_system, unit_selection_display};
pub use command_dispatch::right_click_move_command;
pub use rally::{set_rally_point_click_system, schedule_deliveries_click_system};
pub use legacy_movement::{
    unit_movement_system, unit_rotation_system,
    turn_rate_movement_system, fixed_turn_radius_movement_system,
    speed_turn_radius_movement_system, drag_movement_system, glider_movement_system,
};
pub use occupancy::{rebuild_occupancy_map, collision_repath_system};
pub use indicators::{command_indicator_sync_system, air_unit_separation_system};
pub use channel_locomotion::{
    channel_turnrate_locomotion_system, channel_fallback_locomotion_system,
    channel_orientation_system, crushing_system,
};
