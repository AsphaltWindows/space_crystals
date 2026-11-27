//! What makes a unit type a worker ([`WorkerDef`], `factions/colonists/units/drone.md > Gathering`,
//! `factions/syndicate/units/agent.md > Gathering`) and what it repairs ([`RepairDef`], plan decision T83). Split off
//! `defs/units.rs` in slice M6-GK; `super` re-exports every item.

use serde::{Deserialize, Serialize};

use crate::defs::ObjectKind;

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
    #[serde(deserialize_with = "crate::defs::leak::slice")]
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
    Kinds(#[serde(deserialize_with = "crate::defs::leak::slice")] &'static [(ObjectKind, RepairPayment)]),
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
