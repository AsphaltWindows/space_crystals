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
mod tanks;
mod worker;

pub use attack::{
    AttackAttrs, AttackType, BlastDomain, FullyConnectedSubtype, MELEE_CONTACT_GU, TargetDomain, TargetLockDef,
    TargetType,
};
pub use bases::{ContactStep, Domain, MovementModel, UnitBase, UnitBaseDef};
pub use movement_attrs::{MovementAttrs, ReverseSpeed};
pub use tanks::{PatchKind, SlowDef, TankDef};
pub use worker::{DropOffChoice, LoadDef, Nearest, RepairDef, RepairPayment, RepairTargets, Resource, WorkerDef};

use serde::{Deserialize, Serialize};

use super::{MirageDef, ObjectKind, Recipe, ShroudDef, SpellDef, SupplyChopperDef};
use crate::nav::NavClass;

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
    /// (M6-GA) Its deploy into its pair's other type, for a unit of a vehicle deploy pair (plan decisions T44 and T87,
    /// `crate::vehicle_deploy`): the Artillery Rig's two types (`factions/gdo/units/artillery_rig.md > Modes`), one row
    /// on each. `None` for every other type.
    pub vehicle_deploy: Option<VehicleDeploy>,
    /// (M6-SB) Its tanks, for a unit that lays patches on the ground while a toggle is on (plan decision T93,
    /// `crate::patches`): the Shadebringer's Sludge (`factions/syndicate/units/shadebringer.md > Charges and
    /// recharging`: "Sludge and Cover each have their **own, separate charge**"), and (slice M6-SC) its Cover's; at
    /// most one of each kind ([`TankDef::lays`]). Empty for every other type.
    #[serde(deserialize_with = "super::leak::slice")]
    pub tanks: &'static [TankDef],
    /// Its part in the supply loop, for a Supply Chopper (`crate::supply_run`): it carries Supplies, and lands
    /// on Supply Towers and Supply Delivery Stations. `None` for every other type.
    pub supply_chopper: Option<SupplyChopperDef>,
    /// (M6-ST1) Its own Domain, in place of its base's (plan decision T94; read through [`UnitDef::domain()`]): the
    /// Subterranean Transport's above-ground type, a DrillUnit ("Domain - Underground", `systems/units.md > DrillUnit`)
    /// that "**drives out of the Tunnel in its above-ground mode**, behaving as a tracked vehicle"
    /// (`factions/syndicate/units/subterranean_transport.md > Above-ground mode`): Ground. `None` for every other type,
    /// the Transport's burrowed one among them, which has its base's.
    pub domain: Option<Domain>,
    /// (M6-ST1) Whether its moves crush, in place of its base's `UnitBaseDef::crushes` (read through
    /// [`UnitDef::crushes()`]): the Transport's above-ground type, which "**crushes** enemy Light Infantry like any
    /// tracked vehicle" (`factions/syndicate/units/subterranean_transport.md > Above-ground mode`, design change 0419;
    /// "underground it crushes nothing", its burrowed type its base's). `None` for every other type.
    pub crushes: Option<bool>,
    /// (M6-ST1) Whether it yields, in place of its base's `UnitBaseDef::yields` (read through
    /// [`UnitDef::yields()`]): the Transport's above-ground type stands as a TrackedVehicle does (design change 0440,
    /// answering Q0172 point 1: "WheeledVehicle, TrackedVehicle and a DrillUnit in its above-ground mode stand (they
    /// don't yield and don't take the idle push-apart nudge)"). `None` for every other type, the burrowed Transport's
    /// among them.
    pub yields: Option<bool>,
    /// (M6-ST1) Its hold, for a carrier (plan decision T94, `crate::carry`): the Subterranean Transport's, both of its
    /// types ("they travel along inside it and stay aboard",
    /// `factions/syndicate/units/subterranean_transport.md > Entering the Tunnel Network loaded`). `None` for every
    /// other type.
    pub carries: Option<CarrierDef>,
    /// Whether it is a satellite: unmanned, so no Null Chamber buff (`network::services::manned`;
    /// `factions/syndicate/units/prism.md > Manned - no (no Null Chamber buff)`), and seeing through smoke
    /// (`factions/syndicate/units/prism.md > Sight`). `false` for every type but the Prism.
    pub satellite: bool,
    /// Whether it never goes back into the Tunnel Network once out of it (`network::lets_in`;
    /// `factions/syndicate/units/prism.md > One way only`). `false` for every type but the Prism.
    pub one_way: bool,
    /// How long it unfolds after its rise out of the air exit door, ticks (`Mover::unfold`;
    /// `factions/syndicate/units/prism.md > Launch`), its orders held until it has unfolded. 0 for every type but the
    /// Prism.
    pub unfold_ticks: u32,
    /// Its thrusts (`crate::thrust`, `OrderKind::Thrust`; `factions/syndicate/units/prism.md > Movement`). `None` for
    /// every type but the Prism.
    pub thrust: Option<ThrustDef>,
}

/// A unit's thrusts (`crate::thrust`; `factions/syndicate/units/prism.md > Placeholder stats`).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThrustDef {
    /// The charges it starts with, one spent as each thrust starts; none ever comes back.
    pub charges: u32,
    /// How far each thrust moves it toward the point aimed at, on top of its drift, grid units, whatever that point's
    /// distance.
    pub distance: f32,
    /// How long each thrust lasts, ticks, its distance spread evenly over them.
    pub frames: u32,
}

/// (M6-ST1) A carrier's hold (plan decision T94, `crate::carry`): how many places it has, and the bases of the units
/// it takes with the places one takes. The Subterranean Transport "carries **infantry only**", "**Capacity 4**
/// infantry (user, 2026-11-27: **4 heavy infantry, e.g. 4 Guards**; Syndicate infantry are Heavy Infantry)"
/// (`factions/syndicate/units/subterranean_transport.md > Placeholder stats`): 4 places, an infantry unit of either
/// base taking one.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CarrierDef {
    /// How many places it has: as many units as fit in them are aboard at most.
    pub capacity: u32,
    /// The bases whose units board it, each with the places a unit of it takes. A unit of a base not listed doesn't.
    #[serde(deserialize_with = "super::leak::slice")]
    pub places: &'static [Places],
}

/// (M6-ST1) The places a unit of `base` takes aboard a carrier ([`CarrierDef::places`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Places {
    pub base: UnitBase,
    pub places: u32,
}

impl CarrierDef {
    /// The places a unit of base `base` takes aboard, if it boards at all.
    pub fn places_of(&self, base: UnitBase) -> Option<u32> {
        (self.places.iter()).find_map(|places| (places.base == base).then_some(places.places))
    }
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

/// (M6-GA) A vehicle deploy (plan decisions T44 and T87, `crate::vehicle_deploy`): the unit becomes a unit of type
/// `into` in place, an order that takes `frames` frames and can't be cancelled (`factions/gdo/units/artillery_rig.md >
/// Modes`: "**Deploy is an order, not a mode switch** [...] **the deploying process is what actually changes the
/// unit**: it stays the mobile type until the process completes"). Each type of a pair has its row, which names the
/// other: the Artillery Rig's Deploy into the Artillery Rig (Deployed), whose Undeploy leads back.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VehicleDeploy {
    /// The type the unit becomes once its frames are done: its pair's other type.
    pub into: ObjectKind,
    /// How long the deploy takes, frames ("**Deploy 48 frames (3 s), Undeploy 48 frames.**").
    pub frames: u32,
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

    /// (M6-ST1) Its Domain (`systems/units.md > UnitBase > Domain`): its own (its field `domain`, plan decision T94),
    /// else its base's. Every reader of a type's domain asks this: targeting (`combat::kind_domain`), the collision
    /// layers, the grids and the gate's spots; a landed aircraft is a Ground target by its object
    /// (`combat::object_domain`), and stays on its type's air layer.
    pub fn domain(&self) -> Domain {
        self.domain.unwrap_or(self.base_def().domain)
    }

    /// (M6-ST1) Whether its moves crush (`systems/units.md > UnitCollision > Crushing`, `movement::crush`): its own
    /// (its field `crushes`), else its base's (`UnitBaseDef::crushes`).
    pub fn crushes(&self) -> bool {
        self.crushes.unwrap_or(self.base_def().crushes)
    }

    /// (M6-ST1) Whether it yields (`systems/units.md > UnitCollision > Ground Collision > Friendly yielding`,
    /// `movement::yielding`): its own (its field `yields`), else its base's (`UnitBaseDef::yields`).
    pub fn yields(&self) -> bool {
        self.yields.unwrap_or(self.base_def().yields)
    }

    /// (M6-ST1) The class of the grid it walks on (`UnitBaseDef::nav_class`), by its own Domain
    /// ([`UnitDef::domain()`]): the Transport's above-ground type a ground vehicle's, its burrowed one its base's
    /// drillable ground.
    pub fn nav_class(&self) -> Option<NavClass> {
        bases::nav_class(self.domain(), self.base_def().rugged_terrain)
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

    /// (M6-CM) Its Mirage's numbers, if it casts Mirage ([`UnitDef::spells`]; `crate::mirage`).
    pub fn mirage(&self) -> Option<&'static MirageDef> {
        self.spells.iter().find_map(SpellDef::mirage)
    }

    /// (M6-SB) Its tank of `kind`'s patches, if it has one ([`UnitDef::tanks`]; `crate::patches`).
    pub fn tank(&self, kind: PatchKind) -> Option<&'static TankDef> {
        self.tanks.iter().find(|tank| tank.lays == kind)
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
