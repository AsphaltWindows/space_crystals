//! Unit type definitions — `systems/units.md`, `systems/combat.md > AttackAttributes`.
//!
//! [`UnitBase`] has one static [`UnitBaseDef`] per base (`systems/units.md > UnitBase`, [`bases`]). A unit type's own
//! numbers are a [`UnitDef`] on its `ObjectDef` (`ObjectDef::unit`): its base, the attributes of the base's movement
//! model, its turret and attack; its armour is on the `ObjectDef` itself (`ObjectDef::armor`), as structures have
//! armour too. The faction modules (`defs/gdo.rs`, …) hold the unit types themselves.
//!
//! Units of measure follow `core/scale.md`: speeds in space units per tick, accelerations in space units per tick²,
//! turn rates in radians per tick, ranges in grid units, durations in ticks (the design's "frames").

mod bases;

pub use bases::{ContactStep, Domain, MovementModel, UnitBase, UnitBaseDef};

use serde::{Deserialize, Serialize};

use super::{ObjectKind, Recipe, ShroudDef, SpellDef};
use crate::units::SU_PER_GU;

/// `systems/units.md > Unit > UnitBaseAttributes[UnitBase]`: a unit type's movement numbers, one variant per
/// movement model. An acceleration or deceleration of `None` is infinite: the unit reaches its speed, or stops,
/// within the tick.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
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

/// `systems/units.md > TurretAttributes`: only on units whose base can carry a turret (`UnitBaseDef::can_have_turret`),
/// and a unit of such a base may still have none (design change 0154).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TurretAttrs {
    /// `systems/units.md > TurnAngle`: the full arc, radians, at most a full turn.
    pub turn_angle: f32,
    /// `systems/units.md > TurnRate`, radians per tick.
    pub turn_rate: f32,
}

/// `systems/combat.md > AttackType`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttackType {
    /// `systems/combat.md > FullyConnected`, with its subtype.
    FullyConnected(FullyConnectedSubtype),
    /// `systems/combat.md > HeadDisjointed`.
    HeadDisjointed,
    /// `systems/combat.md > TailDisjointed`.
    TailDisjointed,
    /// `systems/combat.md > DoublyDisjointed`.
    DoublyDisjointed,
}

/// `systems/combat.md > FullyConnected > Subtype`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FullyConnectedSubtype {
    Ranged,
    /// `systems/combat.md > Melee`: "Fixed short range (adjacent contact)": its Range is always [`MELEE_CONTACT_GU`],
    /// and the ElevationModifier never adds to it (`combat::max_range`).
    Melee,
}

/// The Range of every melee attack, grid units: `systems/combat.md > FullyConnected > Melee` ("Short reach: 8 space
/// units for all melee attacks for now") and `systems/combat.md > Range Measurement`, as the Sabre Legionnaire's
/// "Range: melee (adjacent contact)". **Placeholder** (design change 0149: "may become a per-attack value later"): a
/// gap of 8 space units, the one that counts as Proximity (`tuning::orders::PROXIMITY_SU`, design change 0032). Exactly
/// 0 would be fragile ("an exact 0 would fail on tiny collision gaps"): two boxes the movement step brings together may
/// end a rounding error apart. A test keeps every melee attack at this Range, with no MinRange (`defs_are_consistent`).
pub const MELEE_CONTACT_GU: f32 = 8.0 / SU_PER_GU;

/// `systems/combat.md > AttackAttributes > TargetDomain`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetDomain {
    Ground,
    Air,
    Universal,
}

/// `systems/combat.md > AttackAttributes > TargetType`.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TargetType {
    SingleTarget,
    /// With its `AoERadius`, grid units.
    AoE {
        radius: f32,
    },
}

/// `systems/combat.md > AttackAttributes`, read by the attack sequence (`combat`).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
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

/// `factions/gdo/units/peacekeeper.md > Riot Shield (toggled ability)`: a Riot Shield, the same on both of the
/// unit types it switches between (design change 0173: "shield down and shield up are two unit types [...] They share
/// everything except what the shield changes"; `crate::riot_shield`).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RiotShieldDef {
    /// The type of a unit whose shield is down or going down: the Peacekeeper.
    pub lowered: ObjectKind,
    /// The type of a unit whose shield is up or going up: Peacekeeper (Shielded), whose MaxSpeed is the lowered
    /// type's halved ("MaxSpeed is halved (4 → 2 space units/frame)"), which a unit on its way down moves at too
    /// (design change 0171: "While raising **or** lowering, the Peacekeeper moves at half speed (as with the shield
    /// up)").
    pub raised: ObjectKind,
    /// Incoming damage is multiplied by this once the shield is all the way up, before armour
    /// (`systems/combat.md > DamageModifierOrder`, step 1; design change 0171: "only once the shield is fully up").
    pub damage_factor: f32,
    /// How long raising or lowering the shield takes, ticks (`factions/gdo/units/peacekeeper.md > Toggle time`).
    pub toggle_ticks: u32,
}

/// `factions/colonists/units/weapon_conversion.md`: a unit with two forms, each of which converts into the other, the
/// same on both of its types ("two forms of the same underlying unit"; `crate::conversion`).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConversionDef {
    /// The unit's two forms, a type each: the Sabre and the Rifle Legionnaire.
    pub forms: [ObjectKind; 2],
    /// How long a conversion takes, either way, ticks (`factions/colonists/units/weapon_conversion.md > Convert
    /// Command`: "The unit converts in place over 4 frames").
    pub ticks: u32,
}

/// What a worker carries and delivers (`factions/syndicate/units/agent.md > Gathering`: "Agents gather resources and
/// deliver them to Tunnels"): each has its own [`LoadDef`] in a [`WorkerDef`] and its own hatch at a Tunnel
/// (`DropOff::Hatches`).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Resource {
    /// Space Crystals, mined at a Space Crystals patch (`core/map.md > SpaceCrystalsPatch`).
    #[default]
    Crystals,
    /// Supplies, picked up at a Supply Delivery Station (`core/map.md > SupplyDeliveryStation`; slice M4-I).
    Supplies,
}

/// What a worker does with one resource (`factions/colonists/units/drone.md > Gathering`,
/// `factions/syndicate/units/agent.md > Gathering > Space Crystals` and `factions/syndicate/units/agent.md >
/// Supplies`): one load of it, from picking it up to unloading it.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LoadDef {
    /// CarryCapacity: the most it carries of the resource in one load.
    pub carry_capacity: u32,
    /// MiningDuration (PickUpDuration for Supplies), ticks.
    pub pick_up_ticks: u32,
    /// DropOffDuration, ticks.
    pub drop_off_ticks: u32,
    /// MiningReach, space units: it can pick up once its Silhouette is within this of the pick-up point's edge.
    pub pick_up_reach: f32,
    /// DropOffReach, space units: within this of the drop-off point's edge (`StructureDef::drop_off`: the whole
    /// structure, or the resource's hatch).
    pub drop_off_reach: f32,
}

/// Which drop-off a worker's return trip heads for, and when it is chosen (`WorkerDef::drop_off`;
/// `orders::gather`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DropOffChoice {
    /// `factions/colonists/units/drone.md > Assigned Drop-off`: the type has an AssignedDropOff that lasts from
    /// Gather to Gather, and a Set Drop-off Location command. The Drone.
    Assigned,
    /// The own drop-off that takes loads nearest to the worker where it finished mining, chosen anew as each return
    /// starts, the earlier id among equals; no Set Drop-off command. [`Nearest`] is how "nearest" is measured.
    /// - The Recruit, edge to edge (design changes 0345 and 0375, answering Q0102 and Q0136;
    ///   `factions/cults/units/recruit.md > Gathering`: "at **every return** the Recruit takes its load to the own
    ///   completed Storage **nearest to the Recruit itself** (edge to edge) **when it finishes mining**").
    /// - The Agent, in a straight line (design changes 0375 and 0384; `factions/syndicate/units/agent.md > Gathering
    ///   rules`, Which Tunnel: "the own operational Tunnel whose crystal hatch is **nearest to the Agent itself**
    ///   (straight line) **when it finishes mining**", "chosen **at every return**, like the Recruit's Storage").
    EachReturn(Nearest),
}

/// How a worker that chooses its drop-off at every return measures "nearest" ([`DropOffChoice::EachReturn`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Nearest {
    /// From the worker's Silhouette to where the drop-off takes Space Crystals, edge to edge: the Recruit's Storage.
    EdgeToEdge,
    /// From the worker's centre to the centre of the drop-off's crystal point, its crystal hatch, in a straight line,
    /// whatever the load (design change 0292 point 2): the Agent's Tunnel (`economy::drop_off::nearest`).
    StraightLine,
}

/// `factions/colonists/units/drone.md > Gathering`, `factions/syndicate/units/agent.md > Gathering`: what makes a
/// unit type a worker (`crate::economy::gathering`). One Gather job runs every worker (plan decision T34); these are
/// the differences between the types.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorkerDef {
    /// Its loads of Space Crystals.
    pub crystals: LoadDef,
    /// Its loads of Supplies (`factions/syndicate/units/agent.md > Gathering > Supplies`), for a type that gathers
    /// them: the Agent (slice M4-I). Such a worker takes a Gather on a Supply Delivery Station (`orders::gather`).
    pub supplies: Option<LoadDef>,
    /// `factions/syndicate/units/agent.md > Gathering > Space Crystals > Patch sharing`: "any number of Agents may
    /// mine the same Space Crystals patch at the same time (unlike Drones)"; `factions/syndicate/units/agent.md >
    /// Gathering rules`: "Agents share patches (no occupancy rule)". Workers of such a type neither occupy a patch nor
    /// take one for occupied (`orders::gather`).
    pub shares_patches: bool,
    /// Which drop-off a return trip heads for ([`DropOffChoice`]).
    pub drop_off: DropOffChoice,
    /// `factions/colonists/units/drone.md > Patch Occupancy (one Drone per patch) > Search radius` (design change 0123;
    /// the design hasn't named the stat yet), grid units: a worker looks for another patch only within this distance of
    /// itself, centre to centre (`orders::gather`).
    pub search_radius: f32,
    /// `factions/colonists/units/drone.md > Gathering Drones pass through each other` (design change 0106):
    /// workers of this type that are on a Gather job don't collide with each other, whoever owns them
    /// (`orders::gather::on_gather_job`, `movement::space::blocks`). "Drones only": the workers of the other
    /// factions keep normal collision while they mine.
    pub pass_through: bool,
    /// The structures it builds on their sites with a Build order (`core/entities.md > Placement Validation >
    /// Worker-Built Structures`, plan decision T51; `OrderKind::Build`, `crate::construction::site`), each of a type
    /// with a `StructureDef::site`: the Agent's Tunnel (`factions/syndicate/units/agent.md > Building`, slice M4-E).
    /// Empty for a worker that builds nothing so: the Drone deploys instead (`UnitDef::deploy`).
    #[serde(deserialize_with = "super::leak::slice")]
    pub builds: &'static [ObjectKind],
    /// Plan decision T59 (slice M5-E): workers of this type **share a build**. A Build order given to several of them
    /// is one order, paid once, and they build its one site together from outside it, each from its own spot beside
    /// the footprint (`factions/cults/economy.md > Build Command`: "All selected Recruits walk to the location and
    /// begin construction from outside the site"; `factions/cults/units/recruit.md > Building`; `crate::spot_work`):
    /// the Recruit. A worker of another type builds alone, one to a Build, from inside its site (the Agent:
    /// `Reject::SeveralConstructors`). Optional in files: false.
    #[serde(default)]
    pub shares_builds: bool,
}

/// (M5-N8) What a unit type repairs (`UnitDef::repairs`, plan decision T83; `crate::repair`): "**Repair rules
/// shared by every repairer**" (`factions/cults/units/recruit.md > Repair`, `factions/syndicate/units/agent.md >
/// Repair`, `factions/gdo/units/field_engineer.md > Abilities`). Only its owner's objects, and only those whose repair
/// has terms (`repair::terms`).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RepairDef {
    pub targets: RepairTargets,
}

/// (M5-N8) The objects a repairer repairs ([`RepairDef`]), and whether it pays for each.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RepairTargets {
    /// Its owner's buildings and vehicles, for Space Crystals (`factions/cults/units/recruit.md > Repair`: "Recruits
    /// can **repair** own **buildings and vehicles**. Repair **costs Space Crystals**"): the Recruit's. A vehicle is a
    /// unit of neither infantry base that isn't a glider (`repair::is_vehicle`).
    OwnBuildingsAndVehicles,
    /// Its owner's objects of these kinds, each free or paid (`factions/syndicate/units/agent.md > Repair`: "Agents can
    /// **repair** own **Tunnels** and **Laser Turrets** ... **Laser Turrets cost Space Crystals to repair; Tunnels are
    /// repaired for free**"): the Agent's. That a Tunnel's repair is free is the Agent's rule, so it is the Agent's
    /// list that says so.
    Kinds(#[serde(deserialize_with = "super::leak::slice")] &'static [(ObjectKind, RepairPayment)]),
}

/// (M5-N8) Whether a repair costs its owner anything ([`RepairTargets`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepairPayment {
    /// "Repaired for free".
    Free,
    /// Half the target's price over its whole MaxHP, "paid gradually as HP is restored" (`tuning::repair`).
    Paid,
}

impl WorkerDef {
    /// Its loads of `resource`, if it gathers that resource.
    pub fn load(&self, resource: Resource) -> Option<LoadDef> {
        match resource {
            Resource::Crystals => Some(self.crystals),
            Resource::Supplies => self.supplies,
        }
    }
}

/// A unit type's own attributes (`systems/units.md > Unit`); the attributes every object type has are on its
/// `ObjectDef`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UnitDef {
    /// `systems/units.md > UnitBase`.
    pub base: UnitBase,
    /// `systems/units.md > UnitBaseAttributes[UnitBase]`: the attributes of the base's movement model.
    pub movement: MovementAttrs,
    /// `systems/units.md > LightInfantry > RuggedTerrainDefenseBonus`: the fraction of damage taken off on Rugged
    /// tiles. Light infantry only.
    pub rugged_terrain_defense_bonus: Option<f32>,
    /// `systems/units.md > TurretAttributes[UnitBase]`: `None` without a turret.
    pub turret: Option<TurretAttrs>,
    /// `systems/units.md > AttackAttributes`: `None` for units that can't attack.
    pub attack: Option<AttackAttrs>,
    /// `factions/gdo/overview.md > Unit Control`, a unit's `UnitControlCost`: GDO units, and the Cults' (plan decision
    /// T55), whose Unit Control cost "equals the number of Recruits consumed to produce it"
    /// (`factions/cults/structures/recruitment_hall.md > Unit Control Tracking`): 1 for each infantry unit.
    pub unit_control_cost: Option<u32>,
    /// `factions/syndicate/tunnels.md`, a unit's `TunnelSpaceCost`: Syndicate units only.
    pub tunnel_space_cost: Option<u32>,
    /// `factions/colonists/units/README.md`, a unit's `BeaconCapacityCost` (`factions/colonists/overview.md > Beacon
    /// Capacity`): Colonists units only (plan decision T14). Data only until slice M3-C counts it.
    pub beacon_capacity_cost: Option<u32>,
    /// `factions/gdo/units/peacekeeper.md > Riot Shield`.
    pub riot_shield: Option<RiotShieldDef>,
    /// `factions/colonists/units/weapon_conversion.md`: the two forms it converts between, for a unit that has
    /// another form (`crate::conversion`).
    pub conversion: Option<ConversionDef>,
    /// `factions/colonists/units/drone.md > Deploying`: what it deploys into, in place, for a unit that deploys
    /// (`crate::deploy`): the structure, the frames it takes, and its cost.
    pub deploy: Option<Recipe>,
    /// `factions/colonists/units/drone.md > Gathering`: `None` for units that gather nothing.
    pub worker: Option<WorkerDef>,
    /// (M5-N1a) `factions/cults/augmentation.md > Eligibility`: "All four Cults infantry types: Recruit, Militant,
    /// Gunner, and Disciple. Only Cults infantry can be augmented". `None` for every other type: a unit whose kind
    /// changes to one loses its Augmentations (plan decision T73, `Sim::change_kind`). What they do:
    /// `crate::augmentation`.
    #[serde(default)]
    pub augmentable: Option<Augmentable>,
    /// (M5-N8) What it repairs (plan decision T83, `crate::repair`): the Recruit and the Field Engineer their owner's
    /// buildings and vehicles, the Agent its owner's Tunnels and Laser Turrets. `None` for a type that repairs
    /// nothing. A unit's, not a worker's (slice M5-N9): the Field Engineer repairs but gathers nothing
    /// (`factions/gdo/units/field_engineer.md > Abilities`). Optional in files: none.
    #[serde(default)]
    pub repairs: Option<RepairDef>,
    /// (M5-N4b) The spells it casts on its energy pool (`ObjectDef::energy`, `crate::energy`), in its panel's order:
    /// the Disciple's Shroud (`factions/cults/units/disciple.md > Spells > Shroud`, `crate::shroud`). Empty for a type
    /// that casts none. Optional in files: none.
    #[serde(default, deserialize_with = "super::leak::slice")]
    pub spells: &'static [SpellDef],
}

/// (M5-N1a) How an augmentable unit type takes its Augmentations' increments (`UnitDef::augmentable`,
/// `factions/cults/augmentation.md > Increments`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Augmentable {
    /// "**Unarmed units** are infantry whose attack is an unarmed melee attack — currently only the Recruit": the
    /// whole of each increment. The others are armed: they "receive only the MaxHP part of Constitution increments and
    /// only the movement-speed part of Metabolism increments" (the Militant and the Gunner; the Disciple, which has no
    /// weapon, "for Augmentation it is **armed**", `factions/cults/units/disciple.md > AttackAttributes`).
    pub unarmed: bool,
}

impl UnitDef {
    pub fn base_def(&self) -> &'static UnitBaseDef {
        self.base.def()
    }

    /// Whether the unit has a turret: its TurretAttributes are set (design change 0154, `systems/units.md >
    /// CanHaveTurret`: "whether a particular unit actually has one is set by its own TurretAttributes"). Its attack
    /// source is then the turret, otherwise its base (`systems/combat.md > AttackAttributes`).
    pub fn has_turret(&self) -> bool {
        self.turret.is_some()
    }

    /// (M5-N4b) Its Shroud's numbers, if it casts Shroud ([`UnitDef::spells`]).
    pub fn shroud(&self) -> Option<&'static ShroudDef> {
        self.spells.iter().find_map(SpellDef::shroud)
    }
}

#[cfg(test)]
mod tests;
