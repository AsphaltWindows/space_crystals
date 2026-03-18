use bevy::prelude::*;
use crate::types::AppState;

pub mod types;
pub mod utils;
#[cfg(feature = "diagnostics")]
pub mod diagnostics;

#[allow(unused_imports)]
pub use types::*;

// === Simulation Constants ===

/// Simulation runs at 16 frames per second
pub const FRAMES_PER_SECOND: u32 = 16;

/// Each grid unit contains 64 space units
pub const SPACE_UNITS_PER_GRID_UNIT: u32 = 64;

/// Plugin that configures the simulation fixed timestep and system set ordering.
pub struct SimulationCorePlugin;

impl Plugin for SimulationCorePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Time::<Fixed>::from_hz(FRAMES_PER_SECOND as f64));

        // Configure DiagCategory system set ordering and state gating.
        // All Update sets are gated to AppState::InGame at the set level,
        // so individual system tuples no longer need .run_if(in_state(...)).
        app.configure_sets(Update, (
            DiagCategory::Map,
            DiagCategory::Selection,
            DiagCategory::Faction.after(DiagCategory::Selection),
            DiagCategory::Movement.after(DiagCategory::UiHud),
            DiagCategory::Commands,
            DiagCategory::Combat.after(DiagCategory::Movement),
            DiagCategory::Turrets.after(DiagCategory::Combat),
            DiagCategory::Projectiles.after(DiagCategory::Combat),
            DiagCategory::UiHud.after(DiagCategory::Faction),
            DiagCategory::Camera,
        ).run_if(in_state(AppState::InGame)));

        // FixedUpdate sets
        app.configure_sets(FixedUpdate, (
            DiagCategory::FogOfWar,
            DiagCategory::Construction,
            DiagCategory::SupplyDelivery,
        ).run_if(in_state(AppState::InGame)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn space_units_per_grid_unit_is_64() {
        assert_eq!(SPACE_UNITS_PER_GRID_UNIT, 64);
    }

    #[test]
    fn frames_per_second_is_16() {
        assert_eq!(FRAMES_PER_SECOND, 16);
    }

    #[test]
    fn grid_unit_to_space_unit_conversion() {
        let grid = GridUnit(1.0);
        let space: SpaceUnit = grid.into();
        assert_eq!(space.0, 64.0);
    }

    #[test]
    fn space_unit_to_grid_unit_conversion() {
        let space = SpaceUnit(128.0);
        let grid: GridUnit = space.into();
        assert_eq!(grid.0, 2.0);
    }

    #[test]
    fn grid_unit_method_conversion() {
        let grid = GridUnit(2.5);
        let space = grid.to_space_units();
        assert_eq!(space.0, 160.0);
    }

    #[test]
    fn space_unit_method_conversion() {
        let space = SpaceUnit(64.0);
        let grid = space.to_grid_units();
        assert_eq!(grid.0, 1.0);
    }

    #[test]
    fn roundtrip_conversion() {
        let original = GridUnit(3.0);
        let space: SpaceUnit = original.into();
        let back: GridUnit = space.into();
        assert_eq!(original.0, back.0);
    }

    #[test]
    fn simulation_core_plugin_configures_sets() {
        // Verify the plugin builds without panicking and registers set configurations
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<AppState>();
        app.add_plugins(SimulationCorePlugin);
        // If configure_sets had invalid ordering, this would panic
        app.update();
    }

    #[test]
    fn diag_category_has_all_variants() {
        // Ensure all DiagCategory variants exist for set configuration
        let variants = [
            DiagCategory::Map,
            DiagCategory::Selection,
            DiagCategory::Faction,
            DiagCategory::Movement,
            DiagCategory::Commands,
            DiagCategory::Combat,
            DiagCategory::Turrets,
            DiagCategory::Projectiles,
            DiagCategory::UiHud,
            DiagCategory::Camera,
            DiagCategory::FogOfWar,
            DiagCategory::Construction,
            DiagCategory::SupplyDelivery,
        ];
        assert_eq!(variants.len(), 13);
    }

    #[test]
    fn diag_category_is_system_set() {
        // DiagCategory must derive SystemSet — verify it implements the required traits
        fn assert_system_set<T: SystemSet>() {}
        assert_system_set::<DiagCategory>();
    }
}
