use bevy::prelude::*;

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

/// Plugin that configures the simulation fixed timestep.
pub struct SimulationCorePlugin;

impl Plugin for SimulationCorePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Time::<Fixed>::from_hz(FRAMES_PER_SECOND as f64));
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
}
