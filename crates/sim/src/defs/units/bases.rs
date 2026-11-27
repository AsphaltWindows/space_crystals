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

/// `systems/units.md > UnitCollision > Ground Collision`, "The contact step" (design change 0323): the step a moving
/// unit takes when a standing or oncoming unit blocks it on contact (`docs/movement.md > 6.3`, rule 4;
/// `movement::collide::contact`). Which one a base takes is data, here.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContactStep {
    /// "the step is at **90°** to its heading (straight sideways), and its length is **the unit's own width** (the
    /// size of its Silhouette across its heading), so it clears the lane it was in".
    Sideways,
    /// The step as built before design change 0323: the move turned 45° to the side, else one tick's step along the
    /// blocker's face.
    Diagonal,
    /// (M5-V2) "**WheeledVehicles** instead **turn and graze** around the obstacle" (design change 0323): no step
    /// aside, a unit that can't turn in place steers along the blocker's face and grazes past it, its side committed
    /// as a contact step's is (`movement::collide::contact`).
    Graze,
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
    /// (M5-V4) `systems/units.md > UnitCollision > Crushing`: "Units whose UnitBase can crush (TrackedVehicle, Mech, and
    /// a **DrillUnit in its above-ground mode** [...]; never underground) crush enemy units whose UnitBase has Crushable =
    /// true (Light Infantry)" (plan decision T86, `movement::crush`). False for the DrillUnit: it has no above-ground
    /// mode yet, and underground it never crushes (design change 0419: slice M6-ST1 gives it the mode, and this rule
    /// with it).
    #[serde(default)]
    pub crushes: bool,
    /// `> CanTurnInPlace`.
    pub can_turn_in_place: bool,
    /// `> CanReverse`.
    pub can_reverse: bool,
    /// `> MovementModel`.
    pub movement_model: MovementModel,
    /// `systems/units.md > UnitCollision > Ground Collision`, "The contact step" (design change 0323).
    pub contact_step: ContactStep,
    /// (M5-V2) `systems/units.md > UnitCollision > Ground Collision > Friendly yielding`, "Who yields": whether an idle
    /// unit of this base steps aside for a blocked unit of its owner, and moves off a unit it overlaps (design change
    /// 0126, "the same units that yield"). Design change 0353, answering Q0119 point 1: "an idle vehicle doesn't
    /// yield (it can't take a sidestep; it stands like a unit on Hold Position)". False for the WheeledVehicle, the
    /// base Q0119 asked about, which can't step sideways nor turn in place, and (M5-V3) for the TrackedVehicle: design
    /// change 0440, answering Q0172 point 1, "WheeledVehicle, TrackedVehicle and a DrillUnit in its above-ground mode
    /// stand (they don't yield and don't take the idle push-apart nudge)". A HoverVehicle steps any way (0323 gives it
    /// the sideways contact step) and yields as before. The DrillUnit has no above-ground mode yet (slice M6-ST1 gives
    /// it one, and its standing with it): underground it yields as before.
    pub yields: bool,
    /// (M5-H2) `systems/units.md > Air Collision`: how strongly the soft separation of air units pushes a unit of this
    /// base off another (`movement::air`), as the share it takes of the full push (half of what the pair lacks, at
    /// most `tuning::air::Air::separation_push_su` a tick): a HoverCraft all of it, as in slice M5-H1, a Glider a third
    /// (design change 0412; a quarter from design change 0405). `None` for the bases that don't fly, which collide
    /// hard (`Domain::hard_collision`).
    pub separation_strength: Option<f32>,
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

    /// How many unit bases there are.
    pub const COUNT: usize = UnitBase::ALL.len();

    /// The base's def, in the current rules (`crate::ruleset::rules`).
    #[inline]
    pub fn def(self) -> &'static UnitBaseDef {
        &crate::ruleset::rules().bases[self]
    }

    /// Every base's def in the built-in rules (`crate::ruleset::Ruleset::BUILTIN`), in [`UnitBase::ALL`]'s order.
    pub(crate) const BUILTIN_DEFS: [UnitBaseDef; UnitBase::COUNT] = [
        LIGHT_INFANTRY,
        HEAVY_INFANTRY,
        WHEELED_VEHICLE,
        TRACKED_VEHICLE,
        DRILL_UNIT,
        HOVER_VEHICLE,
        MECH,
        HOVER_CRAFT,
        GLIDER,
    ];

    /// The base's def in the built-in rules (`crate::ruleset::Ruleset::builtin`): its own here.
    pub fn builtin_def(self) -> &'static UnitBaseDef {
        &crate::ruleset::Ruleset::builtin_ref().bases[self]
    }

    /// Whether it is an infantry base (`systems/units.md > UnitBase`: LightInfantry and HeavyInfantry): the units
    /// trained from Recruits keep the Recruit's damage and Augmentations (`sim/producers/pool.rs`), and they aren't
    /// vehicles to a repairer (slice M5-N8, `crate::repair::is_vehicle`).
    pub fn is_infantry(self) -> bool {
        matches!(self, UnitBase::LightInfantry | UnitBase::HeavyInfantry)
    }
}

impl crate::ruleset::Key for UnitBase {
    const ALL: &'static [UnitBase] = &UnitBase::ALL;

    #[inline]
    fn index(self) -> usize {
        self as usize
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
const LIGHT_INFANTRY: UnitBaseDef = UnitBaseDef {
    base: UnitBase::LightInfantry,
    domain: Domain::Ground,
    can_have_turret: false,
    directional_armor: false,
    rugged_terrain: true,
    crushable: true,
    crushes: false,
    can_turn_in_place: true,
    can_reverse: false,
    movement_model: MovementModel::TurnRate,
    contact_step: ContactStep::Sideways, // 0323: "infantry (LightInfantry, HeavyInfantry)"
    yields: true,
    separation_strength: None,
    transit: Transit::Infantry, // "Tier 1+: Infantry (Light Infantry, Heavy Infantry)"
};

/// `systems/units.md > HeavyInfantry`.
const HEAVY_INFANTRY: UnitBaseDef = UnitBaseDef {
    base: UnitBase::HeavyInfantry,
    domain: Domain::Ground,
    can_have_turret: false,
    directional_armor: false,
    rugged_terrain: true,
    crushable: false,
    crushes: false,
    can_turn_in_place: true,
    can_reverse: false,
    movement_model: MovementModel::TurnRate,
    contact_step: ContactStep::Sideways, // 0323: "infantry (LightInfantry, HeavyInfantry)"
    yields: true,
    separation_strength: None,
    transit: Transit::Infantry, // "Tier 1+: Infantry (Light Infantry, Heavy Infantry)"
};

/// `systems/units.md > WheeledVehicle`.
const WHEELED_VEHICLE: UnitBaseDef = UnitBaseDef {
    base: UnitBase::WheeledVehicle,
    domain: Domain::Ground,
    can_have_turret: true,
    directional_armor: true,
    rugged_terrain: false,
    crushable: false,
    crushes: false,
    can_turn_in_place: false,
    can_reverse: true,
    movement_model: MovementModel::FixedTurnRadius,
    contact_step: ContactStep::Graze, // 0323: "WheeledVehicles turn and graze around the obstacle instead"
    yields: false,                    // 0353: "an idle vehicle doesn't yield"
    separation_strength: None,
    transit: Transit::Vehicles, // "Tier 2+: Vehicles (Wheeled, Tracked, Drill, Hover Vehicle, Mech)"
};

/// `systems/units.md > TrackedVehicle`.
const TRACKED_VEHICLE: UnitBaseDef = UnitBaseDef {
    base: UnitBase::TrackedVehicle,
    domain: Domain::Ground,
    can_have_turret: true,
    directional_armor: true,
    rugged_terrain: false,
    crushable: false,
    crushes: true, // "Units whose UnitBase can crush (TrackedVehicle, ...)"
    can_turn_in_place: true,
    can_reverse: true,
    movement_model: MovementModel::SpeedTurnRadius,
    contact_step: ContactStep::Sideways, // 0323: "and TrackedVehicles"
    yields: false,                       // 0440: an idle TrackedVehicle stands
    separation_strength: None,
    transit: Transit::Vehicles, // "Tier 2+: Vehicles (Wheeled, Tracked, Drill, Hover Vehicle, Mech)"
};

/// `systems/units.md > DrillUnit`.
const DRILL_UNIT: UnitBaseDef = UnitBaseDef {
    base: UnitBase::DrillUnit,
    domain: Domain::Underground,
    can_have_turret: true,
    directional_armor: true,
    rugged_terrain: false,
    crushable: false,
    crushes: false, // only above ground (0419), a mode slice M6-ST1 builds
    can_turn_in_place: true,
    can_reverse: true,
    movement_model: MovementModel::SpeedTurnRadius,
    // Design change 0338: "DrillUnits take 0323's 90° body-width step" (as the TrackedVehicle: the Subterranean
    // Transport is tracked when above ground).
    contact_step: ContactStep::Sideways,
    yields: true,
    separation_strength: None,
    transit: Transit::Vehicles, // "Tier 2+: Vehicles (Wheeled, Tracked, Drill, Hover Vehicle, Mech)"
};

/// `systems/units.md > HoverVehicle`.
const HOVER_VEHICLE: UnitBaseDef = UnitBaseDef {
    base: UnitBase::HoverVehicle,
    domain: Domain::Ground,
    can_have_turret: true,
    directional_armor: true,
    // `> RuggedTerrain - true, with double drag` (design change 0039): it crosses Rugged tiles, on the grids of the
    // bases that do (`NavClass::GroundRugged`), and its DragRatio is doubled while the tile under its centre is
    // one (`movement::drag`, slice M3-G).
    rugged_terrain: true,
    crushable: false,
    crushes: false,
    can_turn_in_place: true,
    can_reverse: false,
    movement_model: MovementModel::Drag,
    contact_step: ContactStep::Sideways, // 0323: "hover vehicles (HoverVehicle, HoverCraft)"
    yields: true,
    separation_strength: None,
    transit: Transit::Vehicles, // "Tier 2+: Vehicles (Wheeled, Tracked, Drill, Hover Vehicle, Mech)"
};

/// `systems/units.md > Mech`.
const MECH: UnitBaseDef = UnitBaseDef {
    base: UnitBase::Mech,
    domain: Domain::Ground,
    can_have_turret: true,
    directional_armor: true,
    rugged_terrain: true,
    crushable: false,
    crushes: true, // "(..., Mech, ...)"
    can_turn_in_place: true,
    can_reverse: false,
    movement_model: MovementModel::TurnRate,
    // Design change 0325: "Mechs turn to face the side, then step sideways the same way". No Mech unit exists; the
    // turn first isn't built.
    contact_step: ContactStep::Sideways,
    yields: true,
    separation_strength: None,
    transit: Transit::Vehicles, // "Tier 2+: Vehicles (Wheeled, Tracked, Drill, Hover Vehicle, Mech)"
};

/// `systems/units.md > HoverCraft`.
const HOVER_CRAFT: UnitBaseDef = UnitBaseDef {
    base: UnitBase::HoverCraft,
    domain: Domain::Air,
    can_have_turret: true,
    directional_armor: false,
    rugged_terrain: false,
    crushable: false,
    crushes: false,
    can_turn_in_place: true,
    can_reverse: false,
    movement_model: MovementModel::Drag,
    // 0323 names it with the hover vehicles, but it is an air base: ground collision never reaches it.
    contact_step: ContactStep::Sideways,
    yields: true,
    // The full push, slice M5-H1's (plan decision T81), the strength the Glider's is a share of (design change 0405).
    separation_strength: Some(1.0),
    transit: Transit::Air, // "Tier 3+: Air units (Hover Craft, Glider)"
};

/// `systems/units.md > Glider`.
const GLIDER: UnitBaseDef = UnitBaseDef {
    base: UnitBase::Glider,
    domain: Domain::Air,
    can_have_turret: true,
    directional_armor: false,
    rugged_terrain: false,
    crushable: false,
    crushes: false,
    can_turn_in_place: false,
    can_reverse: false,
    movement_model: MovementModel::Glider,
    contact_step: ContactStep::Diagonal, // An air base: ground collision never reaches it.
    yields: true,
    // "**Gliders push apart slowly** (user, 2026-11-27; Q0148): gliders do separate, but with a **much weaker**
    // repulsion than other air units (**a third** of the HoverCraft strength, user, 2026-11-26, Q0152, to try; was a
    // quarter), so they drift apart gradually, mostly while circling, and their attack runs and loops are barely bent"
    // (design changes 0405 and 0412, answering Q0148 part 3 and Q0152). Placeholder.
    separation_strength: Some(1.0 / 3.0),
    transit: Transit::Air, // "Tier 3+: Air units (Hover Craft, Glider)"
};
