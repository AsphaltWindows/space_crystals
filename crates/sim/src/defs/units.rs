//! Unit type definitions — `systems/units.md`, `systems/combat.md > AttackAttributes`.
//!
//! [`UnitBase`] has one static [`UnitBaseDef`] per base (`systems/units.md > UnitBase`, [`bases`]). A unit type's own
//! numbers are a [`UnitDef`] on its `ObjectDef` (`ObjectDef::unit`): its base, the attributes of the base's movement
//! model, its turret and attack; its armour is on the `ObjectDef` itself (`ObjectDef::armor`), as structures have
//! armour too. The faction modules (`defs/gdo.rs`, …) hold the unit types themselves. A unit's attack is in [`attack`],
//! what a worker gathers and a repairer repairs in [`worker`] (split off in slice M6-GK); this module re-exports both.
//!
//! Units of measure follow `core/scale.md`: speeds in space units per tick, accelerations in space units per tick²,
//! turn rates in radians per tick, ranges in grid units, durations in ticks (the design's "frames").

mod attack;
mod bases;
mod movement_attrs;
mod worker;

pub use attack::{
    AttackAttrs, AttackType, BlastDomain, FullyConnectedSubtype, MELEE_CONTACT_GU, TargetDomain, TargetLockDef,
    TargetType,
};
pub use bases::{ContactStep, Domain, MovementModel, UnitBase, UnitBaseDef};
pub use movement_attrs::{MovementAttrs, ReverseSpeed};
pub use worker::{DropOffChoice, LoadDef, Nearest, RepairDef, RepairPayment, RepairTargets, Resource, WorkerDef};

use serde::{Deserialize, Serialize};

use super::{ObjectKind, Recipe, ShroudDef, SpellDef};

/// `systems/units.md > TurretAttributes`: only on units whose base can carry a turret (`UnitBaseDef::can_have_turret`),
/// and a unit of such a base may still have none (design change 0154).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TurretAttrs {
    /// `systems/units.md > TurnAngle`: the full arc, radians, at most a full turn.
    pub turn_angle: f32,
    /// `systems/units.md > TurnRate`, radians per tick.
    pub turn_rate: f32,
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
    /// (M5-H1) `systems/units.md > Air Collision > SeparationRadius`: "per unit type, circular, the distance at which
    /// air-to-air repulsion activates. Must be larger than the unit's Silhouette", space units (`movement::air`). Only
    /// air units have one; `None` for every other type. Optional in files: none.
    #[serde(default)]
    pub separation_radius: Option<f32>,
    /// (M5-K) Its Target Lock (`factions/gdo/units/rocketman.md > Target Lock`, `combat::target_lock`): the
    /// Rocketman's. `None` for every other type. Optional in files: none.
    #[serde(default)]
    pub target_lock: Option<TargetLockDef>,
    /// (M6-GD) Its stock (plan decision T89, `crate::stock`): the Drone Operator's drones. `None` for every other type.
    pub stock: Option<StockDef>,
    /// (M6-GD) Its leash and battery, for a unit its launcher keeps on a leash (plan decision T91, `crate::tether`):
    /// the operator's drone. `None` for every other type.
    pub tether: Option<TetherDef>,
    /// (M6-SD) How long the unit takes to rise out of a Tier 3 Tunnel's air exit door, and to descend into its air
    /// entry door, ticks (`Mover::rise`, `orders::state::Enter::descent`): the Phantom's 16
    /// (`factions/syndicate/units/phantom.md > Placeholder stats`, design change 0431: "it **descends** into the entry
    /// door and **rises** out of the exit door over **16 frames (1 s) each**"). 0 for every other type: an air unit
    /// with none comes out at once, and goes in once it is over the door (the plan's D46).
    pub air_door_ticks: u32,
    /// (M6-GH1) Its ammo, for an aircraft that rearms at its Airfield (plan decision T88, `crate::aircraft`;
    /// `factions/gdo/structures/airfield.md > Aircraft Support`: "GDO aircraft carry limited ammo and **rearm at an
    /// Airfield**"): the Hornet's missiles. `None` for every other type, whose weapon never runs out.
    pub ammo: Option<AmmoDef>,
}

/// (M6-GH1) An aircraft's ammo (plan decision T88, `crate::aircraft`): "Ammo per unit is on each aircraft"
/// (`factions/gdo/structures/airfield.md > Aircraft Support`), and "**Rearm time is per unit of ammo**, and the value
/// differs per aircraft type" (`factions/gdo/structures/airfield.md > Returning and rearming`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AmmoDef {
    /// The rounds it carries, full, as it comes into the game: each shot spends one as it is fired or launched, and at
    /// none it can't attack until it rearms. The Hornet's "Ammo: **5 missiles**, then it must reload at its assigned
    /// Airfield" (`factions/gdo/units/hornet.md > AttackAttributes`).
    pub max: u32,
    /// RearmTimePerAmmo, frames: "Rearming an aircraft takes RearmTimePerAmmo × missing ammo"
    /// (`factions/gdo/structures/airfield.md > Returning and rearming`). The Hornet's
    /// "**RearmTimePerAmmo 32 frames** (5 missiles in 10 s)" (`factions/gdo/units/hornet.md > Placeholder stats`).
    /// Slice M6-GH2 rearms; nothing reads it before.
    pub rearm_ticks_per_ammo: u32,
}

/// (M6-GD) A unit's stock (plan decision T89, `crate::stock`): items it carries, buys one at a time and spends one at a
/// time. The Drone Operator's drones (`factions/gdo/units/drone_operator.md > Drone Stock`); (slice M6-GM) the
/// Twister's mines.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StockDef {
    /// What it holds: the type of the unit it launches, one at a time (`OrderKind::Launch`, `crate::tether`): the
    /// operator's drone ("The Drone Operator launches a drone from its stock").
    pub item: ObjectKind,
    /// The most it holds ("holds a **maximum of 1**").
    pub max: u32,
    /// What it holds as it comes into the game ("Starts with **1** drone when trained").
    pub start: u32,
    /// What one more costs, Space Crystals, paid as its purchase starts (`Ability::Buy`; "**Build Drone** (hotkey
    /// **F**): 50 Space Crystals").
    pub price: u32,
    /// How long a purchase takes, frames ("160 frames (10 s)").
    pub frames: u32,
}

/// (M6-GD) A unit on its launcher's leash (plan decision T91, `crate::tether`): the operator's drone
/// (`factions/gdo/units/drone_operator.md > Drone limits — how a drone ends`). Its attack is its explosion, which it
/// uses once: "a **melee-range attack that it can use only once** — the attack is its explosion [...] using its attack
/// destroys it" (`crate::combat`).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TetherDef {
    /// Its leash range, grid units, edge to edge from its operator ("**leash range 8 grid units**, measured **edge to
    /// edge** like ranges"): past it, it "falls and explodes".
    pub leash: f32,
    /// Its battery, frames ("**battery 20 s** (320 frames)"): as it runs out, it falls without exploding.
    pub battery: u32,
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

    /// (M5-V2) Whether it takes the Reverse order (`systems/controls/commands.md > Reverse`: "Only available to bases
    /// with CanReverse = true"): its base can reverse, and its movement model knows how ([`MovementAttrs::reverse_speed`]:
    /// FixedTurnRadiusMovement and, M5-V3, SpeedTurnRadiusMovement). The gate and the client's command card ask this.
    pub fn can_reverse(&self) -> bool {
        self.base_def().can_reverse && self.movement.reverse_speed().is_some()
    }

    /// (M5-H1) Whether it moves by GliderMovement (`systems/units.md > GliderMovement`): it never stops
    /// (`movement::glider`). Its kind's movement model, which no Augmentation changes.
    pub fn glides(&self) -> bool {
        matches!(self.movement, MovementAttrs::Glider { .. })
    }
}

#[cfg(test)]
mod tests;
