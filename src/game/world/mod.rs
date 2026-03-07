use bevy::prelude::*;
use crate::types::AppState;
use crate::simulation::types::DiagCategory;

pub mod types;
pub mod utils;
pub mod map;
pub(crate) mod faction;
mod resources;

/// Plugin for map-related systems
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        let grid = types::GridMap::default();
        let fog_map = types::FogOfWarMap::new(grid.width, grid.height);
        app.insert_resource(grid)
            .insert_resource(types::GdoBuildArea::default())
            .insert_resource(types::ElevationMap::default())
            .insert_resource(fog_map)
            .insert_resource(types::LastKnownStructures::default())
            .add_systems(OnEnter(AppState::InGame), map::spawn_grid)
            .add_systems(Update, (
                map::tile_hover_system,
                map::draw_grid_lines,
                map::apply_fog_rendering,
                map::apply_structure_fog_rendering,
                crate::game::utils::billboard_label_system,
            ).in_set(DiagCategory::Map)
             .run_if(in_state(AppState::InGame)))
            .add_systems(FixedUpdate, map::update_fog_of_war
                .in_set(DiagCategory::FogOfWar)
                .run_if(in_state(AppState::InGame)));
    }
}

/// Plugin for resource-related systems
pub struct ResourcesPlugin;

impl Plugin for ResourcesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(types::SelectionState::default())
            .add_systems(OnEnter(AppState::InGame), (
                resources::spawn_space_crystal_patches.after(map::spawn_grid),
                resources::spawn_supply_delivery_stations.after(map::spawn_grid),
            ))
            .add_systems(Update, (
                resources::selection_system,
                resources::drag_box_system,
                resources::draw_drag_box_ui,
                resources::manage_selection_indicators,
                resources::log_selection_changes,
                resources::sds_delivery_timer,
            ).in_set(DiagCategory::Selection)
             .run_if(in_state(AppState::InGame)));
    }
}

/// Plugin for faction systems
pub struct FactionPlugin;

impl Plugin for FactionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::ui::types::PlacementState>();
        app.init_resource::<crate::types::ControlGroups>();
        app.init_resource::<crate::types::Selection>();
        app.init_resource::<types::LastRecallState>();

        app.add_systems(OnEnter(AppState::InGame), (
                faction::setup_player_resources,
                faction::setup_gdo_game_start.after(map::spawn_grid),
                faction::setup_syndicate_game_start.after(map::spawn_grid),
                faction::setup_enemy_test_units.after(map::spawn_grid),
            ))
            .add_systems(Update, (
                faction::compute_power_grid,
                faction::display_resources_system,
                resources::control_group_system,
                resources::selection_group_sync_system,
                resources::active_group_cycle_system,
                resources::selection_validation_system,
                faction::manage_placement_ghost,
                faction::update_placement_ghost,
                faction::placement_click_system,
                faction::manage_build_area_overlay,
                faction::barracks_rally_point_system,
            ).in_set(DiagCategory::Faction)
             .run_if(in_state(AppState::InGame)))
            .add_systems(FixedUpdate, (
                faction::dc_construction_tick_system,
                faction::barracks_production_tick_system,
                faction::extraction_plate_mining_system,
                faction::ef_construction_tick_system,
                faction::construction_hp_tick_system,
                faction::rally_target_cleanup_system,
                faction::tunnel_construction_tick_system,
            ).in_set(DiagCategory::Construction)
             .run_if(in_state(AppState::InGame)))
            .add_systems(FixedUpdate,
                faction::supply_tower_production_tick_system
                    .in_set(DiagCategory::SupplyDelivery)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}
