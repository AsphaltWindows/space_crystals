//! The unit bases — `systems/units.md > UnitBase` (moved out of `defs/units.rs`, plan decision T28): one static
//! [`UnitBaseDef`] per [`UnitBase`], the same for every unit type of that base, with its [`Domain`] and its
//! [`MovementModel`].

use serde::{Deserialize, Serialize};

use crate::defs::Transit;
use crate::nav::NavClass;

/// `systems/units.md > UnitBase > Domain`: where a unit exists. It is also the unit's collision layer
/// (`docs/movement.md > 6.1`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Domain {
    Ground,
    Air,
    Underground,
}

impl Domain {
    /// Whether units of this layer collide hard with each other: `systems/units.md > UnitCollision > Ground
    /// Collision`; air units only separate softly (`> Air Collision`). Underground units collide like ground units
    /// until the design says otherwise (question Q0011, gap G14).
    pub fn hard_collision(self) -> bool {
        match self {
            Domain::Ground | Domain::Underground => true,
            Domain::Air => false,
        }
    }
}

/// `systems/units.md > MovementModel`: how a unit moves, turns and accelerates.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MovementModel {
    /// `systems/units.md > TurnRateMovement`.
    TurnRate,
    /// `> FixedTurnRadiusMovement`.
    FixedTurnRadius,
    /// `> SpeedTurnRadiusMovement`.
    SpeedTurnRadius,
    /// `> DragMovement`.
    Drag,
    /// `> GliderMovement`.
    Glider,
}

/// `systems/units.md > UnitBase`: the nine unit bases.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UnitBase {
    LightInfantry,
    HeavyInfantry,
    WheeledVehicle,
    TrackedVehicle,
    DrillUnit,
    HoverVehicle,
    Mech,
    HoverCraft,
    Glider,
}

/// What a unit base is (`systems/units.md > UnitBase`): the same for every unit type of that base.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UnitBaseDef {
    pub base: UnitBase,
    /// `> Domain`.
    pub domain: Domain,
    /// `> CanHaveTurret` (design change 0154, renamed from HasTurret): units of this base **can** carry a turret.
    /// Whether one actually has one is its own TurretAttributes' ([`super::UnitDef::has_turret`]): the HoverVehicle
    /// base can, and the Drone, built on it, has none.
    pub can_have_turret: bool,
    /// `> DirectionalArmor`.
    pub directional_armor: bool,
    /// `> RuggedTerrain`: the base crosses Rugged tiles (`core/map.md > TilePreset`).
    pub rugged_terrain: bool,
    /// `> Crushable` (`systems/units.md > UnitCollision > Crushing`).
    pub crushable: bool,
    /// `> CanTurnInPlace`.
    pub can_turn_in_place: bool,
    /// `> CanReverse`.
    pub can_reverse: bool,
    /// `> MovementModel`.
    pub movement_model: MovementModel,
    /// The class of unit the base belongs to for the Tunnel Network's transit (`factions/syndicate/tunnels.md > Tunnel
    /// Network > Transit Tier Requirements`, as it lists the bases; [`Transit::of`]).
    pub transit: Transit,
}

impl UnitBase {
    /// Every base, in declaration order.
    pub const ALL: [UnitBase; 9] = [
        UnitBase::LightInfantry,
        UnitBase::HeavyInfantry,
        UnitBase::WheeledVehicle,
        UnitBase::TrackedVehicle,
        UnitBase::DrillUnit,
        UnitBase::HoverVehicle,
        UnitBase::Mech,
        UnitBase::HoverCraft,
        UnitBase::Glider,
    ];

    pub fn def(self) -> &'static UnitBaseDef {
        match self {
            UnitBase::LightInfantry => &LIGHT_INFANTRY,
            UnitBase::HeavyInfantry => &HEAVY_INFANTRY,
            UnitBase::WheeledVehicle => &WHEELED_VEHICLE,
            UnitBase::TrackedVehicle => &TRACKED_VEHICLE,
            UnitBase::DrillUnit => &DRILL_UNIT,
            UnitBase::HoverVehicle => &HOVER_VEHICLE,
            UnitBase::Mech => &MECH,
            UnitBase::HoverCraft => &HOVER_CRAFT,
            UnitBase::Glider => &GLIDER,
        }
    }
}

impl UnitBaseDef {
    /// The passability rules a unit of this base paths with (`docs/movement.md > 5.2`); `None` for air units,
    /// which cross every tile of the map (question Q0011, gap G14).
    pub fn nav_class(&self) -> Option<NavClass> {
        match self.domain {
            Domain::Ground if self.rugged_terrain => Some(NavClass::GroundRugged),
            Domain::Ground => Some(NavClass::Ground),
            Domain::Underground => Some(NavClass::Underground),
            Domain::Air => None,
        }
    }
}

/// `systems/units.md > LightInfantry`.
static LIGHT_INFANTRY: UnitBaseDef = UnitBaseDef {
    base: UnitBase::LightInfantry,
    domain: Domain::Ground,
    can_have_turret: false,
    directional_armor: false,
    rugged_terrain: true,
    crushable: true,
    can_turn_in_place: true,
    can_reverse: false,
    movement_model: MovementModel::TurnRate,
    transit: Transit::Infantry, // "Tier 1+: Infantry (Light Infantry, Heavy Infantry)"
};

/// `systems/units.md > HeavyInfantry`.
static HEAVY_INFANTRY: UnitBaseDef = UnitBaseDef {
    base: UnitBase::HeavyInfantry,
    domain: Domain::Ground,
    can_have_turret: false,
    directional_armor: false,
    rugged_terrain: true,
    crushable: false,
    can_turn_in_place: true,
    can_reverse: false,
    movement_model: MovementModel::TurnRate,
    transit: Transit::Infantry, // "Tier 1+: Infantry (Light Infantry, Heavy Infantry)"
};

/// `systems/units.md > WheeledVehicle`.
static WHEELED_VEHICLE: UnitBaseDef = UnitBaseDef {
    base: UnitBase::WheeledVehicle,
    domain: Domain::Ground,
    can_have_turret: true,
    directional_armor: true,
    rugged_terrain: false,
    crushable: false,
    can_turn_in_place: false,
    can_reverse: true,
    movement_model: MovementModel::FixedTurnRadius,
    transit: Transit::Vehicles, // "Tier 2+: Vehicles (Wheeled, Tracked, Drill, Hover Vehicle, Mech)"
};

/// `systems/units.md > TrackedVehicle`.
static TRACKED_VEHICLE: UnitBaseDef = UnitBaseDef {
    base: UnitBase::TrackedVehicle,
    domain: Domain::Ground,
    can_have_turret: true,
    directional_armor: true,
    rugged_terrain: false,
    crushable: false,
    can_turn_in_place: true,
    can_reverse: true,
    movement_model: MovementModel::SpeedTurnRadius,
    transit: Transit::Vehicles, // "Tier 2+: Vehicles (Wheeled, Tracked, Drill, Hover Vehicle, Mech)"
};

/// `systems/units.md > DrillUnit`.
static DRILL_UNIT: UnitBaseDef = UnitBaseDef {
    base: UnitBase::DrillUnit,
    domain: Domain::Underground,
    can_have_turret: true,
    directional_armor: true,
    rugged_terrain: false,
    crushable: false,
    can_turn_in_place: true,
    can_reverse: true,
    movement_model: MovementModel::SpeedTurnRadius,
    transit: Transit::Vehicles, // "Tier 2+: Vehicles (Wheeled, Tracked, Drill, Hover Vehicle, Mech)"
};

/// `systems/units.md > HoverVehicle`.
static HOVER_VEHICLE: UnitBaseDef = UnitBaseDef {
    base: UnitBase::HoverVehicle,
    domain: Domain::Ground,
    can_have_turret: true,
    directional_armor: true,
    // `> RuggedTerrain - true, with double drag` (design change 0039): it crosses Rugged tiles, on the grids of the
    // bases that do (`NavClass::GroundRugged`), and its DragRatio is doubled while the tile under its centre is
    // one (`movement::drag`, slice M3-G).
    rugged_terrain: true,
    crushable: false,
    can_turn_in_place: true,
    can_reverse: false,
    movement_model: MovementModel::Drag,
    transit: Transit::Vehicles, // "Tier 2+: Vehicles (Wheeled, Tracked, Drill, Hover Vehicle, Mech)"
};

/// `systems/units.md > Mech`.
static MECH: UnitBaseDef = UnitBaseDef {
    base: UnitBase::Mech,
    domain: Domain::Ground,
    can_have_turret: true,
    directional_armor: true,
    rugged_terrain: true,
    crushable: false,
    can_turn_in_place: true,
    can_reverse: false,
    movement_model: MovementModel::TurnRate,
    transit: Transit::Vehicles, // "Tier 2+: Vehicles (Wheeled, Tracked, Drill, Hover Vehicle, Mech)"
};

/// `systems/units.md > HoverCraft`.
static HOVER_CRAFT: UnitBaseDef = UnitBaseDef {
    base: UnitBase::HoverCraft,
    domain: Domain::Air,
    can_have_turret: true,
    directional_armor: false,
    rugged_terrain: false,
    crushable: false,
    can_turn_in_place: true,
    can_reverse: false,
    movement_model: MovementModel::Drag,
    transit: Transit::Air, // "Tier 3+: Air units (Hover Craft, Glider)"
};

/// `systems/units.md > Glider`.
static GLIDER: UnitBaseDef = UnitBaseDef {
    base: UnitBase::Glider,
    domain: Domain::Air,
    can_have_turret: true,
    directional_armor: false,
    rugged_terrain: false,
    crushable: false,
    can_turn_in_place: false,
    can_reverse: false,
    movement_model: MovementModel::Glider,
    transit: Transit::Air, // "Tier 3+: Air units (Hover Craft, Glider)"
};
