#![allow(dead_code)]

use bevy::prelude::*;

/// System set labels for diagnostics instrumentation.
/// Always compiled (zero-cost metadata). The diagnostics plugin
/// optionally adds timing systems ordered around these sets.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum DiagCategory {
    FogOfWar,
    Construction,
    SupplyDelivery,
    Combat,
    Turrets,
    Projectiles,
    Movement,
    Commands,
    Selection,
    Map,
    Faction,
    UiHud,
    Camera,
}

/// Newtype wrapper for strategic-scale spatial measurement.
/// Used for structure placement (grid snapping), range, sight range,
/// min range, and other strategic distances.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct GridUnit(pub f32);

/// Newtype wrapper for fine-grained spatial measurement.
/// Used for unit silhouettes, movement speeds (space units per frame),
/// acceleration, and physical positioning.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct SpaceUnit(pub f32);

impl From<GridUnit> for SpaceUnit {
    fn from(grid: GridUnit) -> Self {
        SpaceUnit(grid.0 * super::SPACE_UNITS_PER_GRID_UNIT as f32)
    }
}

impl From<SpaceUnit> for GridUnit {
    fn from(space: SpaceUnit) -> Self {
        GridUnit(space.0 / super::SPACE_UNITS_PER_GRID_UNIT as f32)
    }
}

impl GridUnit {
    /// Convert to SpaceUnit
    pub fn to_space_units(self) -> SpaceUnit {
        self.into()
    }
}

impl SpaceUnit {
    /// Convert to GridUnit
    pub fn to_grid_units(self) -> GridUnit {
        self.into()
    }
}
