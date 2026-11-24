//! Unit type definitions — `systems/units.md`, `systems/combat.md > AttackAttributes`.
//!
//! [`UnitBase`] has one static [`UnitBaseDef`] per base (`systems/units.md > UnitBase`). A unit type's own numbers
//! are a [`UnitDef`] on its `ObjectDef` (`ObjectDef::unit`): its base, the attributes of the base's movement model,
//! its turret and attack; its armour is on the `ObjectDef` itself (`ObjectDef::armor`), as structures have armour
//! too. The faction modules (`defs/gdo.rs`, …) hold the unit types themselves.
//!
//! Units of measure follow `core/scale.md`: speeds in space units per tick, accelerations in space units per tick²,
//! turn rates in radians per tick, ranges in grid units, durations in ticks (the design's "frames").

use crate::nav::NavClass;
use crate::units::SU_PER_GU;

/// `systems/units.md > UnitBase > Domain`: where a unit exists. It is also the unit's collision layer
/// (`docs/movement.md > 6.1`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
#[derive(Clone, Debug, PartialEq)]
pub struct UnitBaseDef {
    pub base: UnitBase,
    /// `> Domain`.
    pub domain: Domain,
    /// `> HasTurret`.
    pub has_turret: bool,
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
    has_turret: false,
    directional_armor: false,
    rugged_terrain: true,
    crushable: true,
    can_turn_in_place: true,
    can_reverse: false,
    movement_model: MovementModel::TurnRate,
};

/// `systems/units.md > HeavyInfantry`.
static HEAVY_INFANTRY: UnitBaseDef = UnitBaseDef {
    base: UnitBase::HeavyInfantry,
    domain: Domain::Ground,
    has_turret: false,
    directional_armor: false,
    rugged_terrain: true,
    crushable: false,
    can_turn_in_place: true,
    can_reverse: false,
    movement_model: MovementModel::TurnRate,
};

/// `systems/units.md > WheeledVehicle`.
static WHEELED_VEHICLE: UnitBaseDef = UnitBaseDef {
    base: UnitBase::WheeledVehicle,
    domain: Domain::Ground,
    has_turret: true,
    directional_armor: true,
    rugged_terrain: false,
    crushable: false,
    can_turn_in_place: false,
    can_reverse: true,
    movement_model: MovementModel::FixedTurnRadius,
};

/// `systems/units.md > TrackedVehicle`.
static TRACKED_VEHICLE: UnitBaseDef = UnitBaseDef {
    base: UnitBase::TrackedVehicle,
    domain: Domain::Ground,
    has_turret: true,
    directional_armor: true,
    rugged_terrain: false,
    crushable: false,
    can_turn_in_place: true,
    can_reverse: true,
    movement_model: MovementModel::SpeedTurnRadius,
};

/// `systems/units.md > DrillUnit`.
static DRILL_UNIT: UnitBaseDef = UnitBaseDef {
    base: UnitBase::DrillUnit,
    domain: Domain::Underground,
    has_turret: true,
    directional_armor: true,
    rugged_terrain: false,
    crushable: false,
    can_turn_in_place: true,
    can_reverse: true,
    movement_model: MovementModel::SpeedTurnRadius,
};

/// `systems/units.md > HoverVehicle`.
static HOVER_VEHICLE: UnitBaseDef = UnitBaseDef {
    base: UnitBase::HoverVehicle,
    domain: Domain::Ground,
    has_turret: true,
    directional_armor: true,
    // `> RuggedTerrain - true, with double drag`: crossing Rugged tiles waits for DragMovement (slice M3-G). Until
    // then the Drone moves on the stand-in, to which Rugged tiles stay impassable (plan decision T25).
    rugged_terrain: false,
    crushable: false,
    can_turn_in_place: true,
    can_reverse: false,
    movement_model: MovementModel::Drag,
};

/// `systems/units.md > Mech`.
static MECH: UnitBaseDef = UnitBaseDef {
    base: UnitBase::Mech,
    domain: Domain::Ground,
    has_turret: true,
    directional_armor: true,
    rugged_terrain: true,
    crushable: false,
    can_turn_in_place: true,
    can_reverse: false,
    movement_model: MovementModel::TurnRate,
};

/// `systems/units.md > HoverCraft`.
static HOVER_CRAFT: UnitBaseDef = UnitBaseDef {
    base: UnitBase::HoverCraft,
    domain: Domain::Air,
    has_turret: true,
    directional_armor: false,
    rugged_terrain: false,
    crushable: false,
    can_turn_in_place: true,
    can_reverse: false,
    movement_model: MovementModel::Drag,
};

/// `systems/units.md > Glider`.
static GLIDER: UnitBaseDef = UnitBaseDef {
    base: UnitBase::Glider,
    domain: Domain::Air,
    has_turret: true,
    directional_armor: false,
    rugged_terrain: false,
    crushable: false,
    can_turn_in_place: false,
    can_reverse: false,
    movement_model: MovementModel::Glider,
};

/// `systems/units.md > Unit > UnitBaseAttributes[UnitBase]`: a unit type's movement numbers, one variant per
/// movement model. An acceleration or deceleration of `None` is infinite: the unit reaches its speed, or stops,
/// within the tick.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MovementAttrs {
    /// `systems/units.md > TurnRateMovement`.
    TurnRate {
        /// Radians per tick.
        turn_rate: f32,
        acceleration: Option<f32>,
        deceleration: Option<f32>,
        /// Space units per tick.
        max_speed: f32,
    },
    /// `systems/units.md > FixedTurnRadiusMovement`.
    FixedTurnRadius {
        /// Space units.
        minimum_turn_radius: f32,
        forward_acceleration: Option<f32>,
        forward_max_speed: f32,
        reverse_acceleration: Option<f32>,
        reverse_max_speed: f32,
        deceleration: Option<f32>,
    },
    /// `systems/units.md > SpeedTurnRadiusMovement`.
    SpeedTurnRadius {
        speed_to_turn_radius_ratio: f32,
        acceleration: Option<f32>,
        deceleration: Option<f32>,
        max_speed: f32,
    },
    /// `systems/units.md > DragMovement`: top speed is where thrust and drag balance.
    Drag {
        forward_acceleration: f32,
        omni_directional_acceleration: f32,
        drag_ratio: f32,
        /// Radians per tick.
        turn_rate: f32,
    },
    /// `systems/units.md > GliderMovement`.
    Glider {
        idle_speed: f32,
        max_speed: f32,
        acceleration: Option<f32>,
        deceleration: Option<f32>,
        max_centripetal_acceleration: f32,
    },
}

impl MovementAttrs {
    pub fn model(&self) -> MovementModel {
        match self {
            MovementAttrs::TurnRate { .. } => MovementModel::TurnRate,
            MovementAttrs::FixedTurnRadius { .. } => MovementModel::FixedTurnRadius,
            MovementAttrs::SpeedTurnRadius { .. } => MovementModel::SpeedTurnRadius,
            MovementAttrs::Drag { .. } => MovementModel::Drag,
            MovementAttrs::Glider { .. } => MovementModel::Glider,
        }
    }

    /// The top forward speed, space units per tick. `DragMovement`: `(OmniDirectionalAcceleration +
    /// ForwardAcceleration) / DragRatio` (`systems/units.md > DragMovement`).
    pub fn max_speed(&self) -> f32 {
        match *self {
            MovementAttrs::TurnRate { max_speed, .. }
            | MovementAttrs::SpeedTurnRadius { max_speed, .. }
            | MovementAttrs::Glider { max_speed, .. } => max_speed,
            MovementAttrs::FixedTurnRadius { forward_max_speed, .. } => forward_max_speed,
            MovementAttrs::Drag {
                forward_acceleration,
                omni_directional_acceleration,
                drag_ratio,
                ..
            } => (omni_directional_acceleration + forward_acceleration) / drag_ratio,
        }
    }

    /// The most the unit turns per tick, radians, for the models with a fixed turn rate
    /// (`systems/combat.md > LocomotionOrientationConstraints`); `None` for the models whose turning depends on
    /// their speed.
    pub fn turn_rate(&self) -> Option<f32> {
        match *self {
            MovementAttrs::TurnRate { turn_rate, .. } | MovementAttrs::Drag { turn_rate, .. } => Some(turn_rate),
            MovementAttrs::FixedTurnRadius { .. }
            | MovementAttrs::SpeedTurnRadius { .. }
            | MovementAttrs::Glider { .. } => None,
        }
    }
}

/// `systems/units.md > TurretAttributes`: only units whose base has a turret.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TurretAttrs {
    /// `> TurnAngle`: the full arc, radians, at most a full turn.
    pub turn_angle: f32,
    /// `> TurnRate`, radians per tick.
    pub turn_rate: f32,
}

/// `systems/combat.md > AttackType`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AttackType {
    /// `> FullyConnected`, with its subtype.
    FullyConnected(FullyConnectedSubtype),
    /// `> HeadDisjointed`.
    HeadDisjointed,
    /// `> TailDisjointed`.
    TailDisjointed,
    /// `> DoublyDisjointed`.
    DoublyDisjointed,
}

/// `systems/combat.md > FullyConnected > Subtype`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FullyConnectedSubtype {
    Ranged,
    /// `> Melee`: "Fixed short range (adjacent contact)": its Range is always [`MELEE_CONTACT_GU`], and the
    /// ElevationModifier never adds to it (`combat::max_range`).
    Melee,
}

/// The Range of every melee attack, grid units: `systems/combat.md > FullyConnected > Melee` ("Fixed short range
/// (adjacent contact)") and `> Range Measurement` ("Melee (adjacent contact) is a gap of about 0"), as the Sabre
/// Legionnaire's "Range: melee (adjacent contact)". **Placeholder** (question Q0042 point 5): a gap of 8 space units,
/// the one that counts as Proximity (`orders::PROXIMITY_SU`, design change 0032). Exactly 0 would be fragile: two
/// boxes the movement step brings together may end a rounding error apart. A test keeps every melee attack at this
/// Range, with no MinRange (`defs_are_consistent`).
pub const MELEE_CONTACT_GU: f32 = 8.0 / SU_PER_GU;

/// `systems/combat.md > AttackAttributes > TargetDomain`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TargetDomain {
    Ground,
    Air,
    Universal,
}

/// `systems/combat.md > AttackAttributes > TargetType`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TargetType {
    SingleTarget,
    /// With its `AoERadius`, grid units.
    AoE {
        radius: f32,
    },
}

/// `systems/combat.md > AttackAttributes`, read by the attack sequence (`combat`).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AttackAttrs {
    pub attack_type: AttackType,
    pub target_domain: TargetDomain,
    pub target_type: TargetType,
    pub damage: f32,
    /// Grid units.
    pub range: f32,
    /// Grid units.
    pub min_range: f32,
    /// Space units per tick; only for `HeadDisjointed` and `DoublyDisjointed`.
    pub projectile_speed: Option<f32>,
    /// `systems/combat.md > AttackPhase`: the phases' durations, ticks.
    pub aim_ticks: u32,
    pub firing_ticks: u32,
    pub cooldown_ticks: u32,
    pub reload_ticks: u32,
}

/// `factions/gdo/units.md > Peacekeeper > Riot Shield (toggled ability)`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RiotShieldDef {
    /// MaxSpeed is multiplied by this while the shield is up (the `Mover`'s speed factor).
    pub speed_factor: f32,
    /// Incoming damage is multiplied by this while the shield is up, before armour
    /// (`systems/combat.md > DamageModifierOrder`, step 1).
    pub damage_factor: f32,
    /// How long raising or lowering the shield takes, ticks.
    pub toggle_ticks: u32,
}

/// A unit type's own attributes (`systems/units.md > Unit`); the attributes every object type has are on its
/// `ObjectDef`.
#[derive(Clone, Debug, PartialEq)]
pub struct UnitDef {
    /// `> UnitBase`.
    pub base: UnitBase,
    /// `> UnitBaseAttributes[UnitBase]`: the attributes of the base's movement model.
    pub movement: MovementAttrs,
    /// `systems/units.md > LightInfantry > RuggedTerrainDefenseBonus`: the fraction of damage taken off on Rugged
    /// tiles. Light infantry only.
    pub rugged_terrain_defense_bonus: Option<f32>,
    /// `> TurretAttributes[UnitBase]`: `None` without a turret.
    pub turret: Option<TurretAttrs>,
    /// `> AttackAttributes`: `None` for units that can't attack.
    pub attack: Option<AttackAttrs>,
    /// `factions/gdo/units.md > UnitControlCost`: GDO units only.
    pub unit_control_cost: Option<u32>,
    /// `factions/syndicate/tunnels.md`, a unit's `TunnelSpaceCost`: Syndicate units only.
    pub tunnel_space_cost: Option<u32>,
    /// `factions/colonists/units.md`, a unit's `BeaconCapacityCost` (`factions/colonists/overview.md > Beacon
    /// Capacity`): Colonists units only (plan decision T14). Data only until slice M3-C counts it.
    pub beacon_capacity_cost: Option<u32>,
    /// `factions/gdo/units.md > Peacekeeper > Riot Shield`.
    pub riot_shield: Option<RiotShieldDef>,
}

impl UnitDef {
    pub fn base_def(&self) -> &'static UnitBaseDef {
        self.base.def()
    }
}

#[cfg(test)]
mod tests;
