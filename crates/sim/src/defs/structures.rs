//! The attributes of structure types (`core/entities.md > Structure Type`). The faction modules (`defs/gdo.rs`, …)
//! hold the structure types themselves.

use serde::{Deserialize, Serialize};

use crate::defs::{Armor, AttackAttrs, ObjectKind, Resource, UnitBase};
use crate::sides::SideLetter;
use crate::units::Tick;

/// A structure type's own attributes; the attributes every object type has, armour included, are on its
/// `ObjectDef`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StructureDef {
    /// `core/entities.md > Structure Type > SymmetryType`.
    pub symmetry: SymmetryType,
    /// `BuildRadiusExtension`, grid units: how far the type grows its owner's build area around its footprint
    /// (`factions/gdo/structures/README.md > GDOBuildArea`). 0 for none.
    pub build_radius_extension: u32,
    /// `Power`: what the type adds to its owner's power, positive for a generator, negative for a consumer
    /// (`factions/gdo/economy.md`).
    pub power: i32,
    /// Whether the type seeds its owner's build area instead of having to reach it (`factions/gdo/structures/README.md
    /// > GDOBuildArea`: "Seeded by each Deployment Center's and each Outpost's BuildRadiusExtension";
    /// `placement::seeds_build_area`): the Deployment Center.
    pub seeds_build_area: bool,
    /// What the type produces from its queue, and through which side (`crate::production`): the Barracks' units,
    /// `None` for a structure that produces nothing.
    pub production: Option<ProductionDef>,
    /// How the type mines the Space Crystals patch it stands on (`crate::economy::mining`): the Extraction Plate's
    /// rates, `None` for every other structure.
    pub mining: Option<MiningDef>,
    /// The type of object a structure of the type is placed onto, exactly over it, sharing its tiles
    /// (`placement::onto`, `ObjectKind::may_overlap`): the Space Crystals Patch for the Extraction Plate
    /// (`factions/gdo/economy.md > ExtractionPlate`: "Placed onto a Space Crystal Patch"); `None` for the others.
    pub onto: Option<ObjectKind>,
    /// `Constructs`: the buildings the type constructs, one at a time (`crate::construction`), with their costs and
    /// times (`factions/gdo/structures/deployment_center.md > Constructs`, `factions/gdo/economy.md >
    /// ExtractionFacility > Constructs`). A type with any is a constructor (`Object::constructor`); empty for the
    /// others.
    #[serde(deserialize_with = "super::leak::slice")]
    pub constructs: &'static [Recipe],
    /// The Beacon's `Constructs List` (`factions/colonists/economy.md > Warp In > Constructs List`): the buildings a
    /// Beacon transforms into in place, with their costs and WarpInTimes; empty for every other type. Not
    /// [`StructureDef::constructs`], which makes its type a GDO-style constructor (plan decision T14): a Warp In
    /// turns the Beacon itself into the building (`crate::transform`, plan decision T15).
    #[serde(deserialize_with = "super::leak::slice")]
    pub warp_in: &'static [Recipe],
    /// Whether workers deliver their loads to the type, and where at it: `None` for a type that takes no loads
    /// (`factions/colonists/economy.md > Egress Node > Drop Off`: "the only way resources enter the player's resource
    /// pool"; the Tunnel's `SymmetryType - ABCD` in `factions/syndicate/tunnels.md`; `crate::economy::gathering`,
    /// `crate::economy::drop_off`).
    pub drop_off: Option<DropOff>,
    /// Whether the type is an underground structure (plan decision T32; `factions/syndicate/structures/README.md >
    /// Tunnel Expansions`: "Expansions are invisible to enemies without detection and can be walked over by surface
    /// units"): its footprint blocks no movement (`nav::BlockerKind::UndergroundExpansion`), and while it has an owner
    /// it is hidden from the other players (`vision::hidden_from`). The Syndicate's expansions: the Hideout so far.
    pub underground: bool,
    /// The Tunnel's tier table and its network's rules (plan decision T39, `factions/syndicate/tunnels.md > Tunnel >
    /// Tiers`, `factions/syndicate/tunnels.md > Tunnel Network`): an object of a type with one carries its tier
    /// (`Object::tunnel`, `crate::tunnel`). `None` for every other type.
    pub tunnel: Option<TunnelDef>,
    /// How a worker builds the type on its site (plan decisions T31 and T51, slice M4-E; `core/entities.md >
    /// Placement Validation > Worker-Built Structures`, `core/entities.md > ConstructionHP Rule`;
    /// `crate::construction::site`): what an order to build one costs, how long its construction takes, the HP it
    /// starts at, and where its builder goes as it completes. The units that build it list it in their worker's def
    /// (`WorkerDef::builds`). `None` for a type no worker builds; the Drone's Beacon is a deployment instead
    /// (`UnitDef::deploy`, T51). The Tunnel's and the Laser Turret's (slice M4-S1).
    pub site: Option<SiteDef>,
    /// How a Tunnel builds the type in its Tunnel Area, as its one operation (slice M4-F;
    /// `factions/syndicate/tunnels.md > Tunnel ObjectInterfaceState > ExpandMenu`, `factions/syndicate/tunnels.md >
    /// Construction and Upgrade Rules`; `crate::construction::site`): its Tier Requirement, its cost and its build
    /// time, and the HP its site starts at. `None` for a type no Tunnel builds. The Hideout, the Optics Lab and the
    /// Chemical Plant so far; the ExpandMenu lists the types in its order (`defs::EXPAND_MENU`).
    pub expansion: Option<ExpansionDef>,
    /// Its `TechPrerequisite` (`core/tech_and_research.md > Tech Prerequisites`; plan decision T37, slice M4-K): the
    /// structure types its player must own, each at least once, before one can be started, by whichever way it is
    /// built (a constructor's Construct, a Warp In, an expansion, a worker's Build; `crate::tech`). Empty for none;
    /// the Laser Turret's Optics Lab is the first (slice M4-S1).
    #[serde(deserialize_with = "super::leak::slice")]
    pub requires: &'static [ObjectKind],
    /// The type's attack, for a structure that attacks (plan decision T49, slice M4-S1; `systems/combat.md >
    /// AttackAttributes`): the Laser Turret's (`factions/syndicate/structures/laser_turret.md`). It fires through the
    /// phases and the damage every attack shares (`crate::combat`), with no turn rate (it fires in any direction), no
    /// orders, and a target pick of its own (`combat::structures`). `None` for every other structure. Optional in
    /// files.
    #[serde(default)]
    pub attack: Option<AttackAttrs>,
    /// The type's pop-up form (plan decision T50, slice M4-S2; `factions/syndicate/structures/laser_turret.md >
    /// Upright vs Pop-up`): how long it takes to rise and to retract, how long it stays up with no target, and its
    /// armour while underground. An object of a type with one is a pop-up while it stands inside an own Tunnel Area
    /// (`crate::popup`). `None` for every other structure. Optional in files.
    #[serde(default)]
    pub popup: Option<PopUpDef>,
    /// The units the type holds inside (plan decision T60, slice M5-F1; `systems/controls/commands.md > Enter`: "an own
    /// structure that can hold units"; `crate::holding`): the side they walk in at, the side they come out of, how
    /// many its pool holds, and the kinds it takes. `None` for a type that holds none. Optional in files.
    #[serde(default)]
    pub holds: Option<HoldsDef>,
}

/// A structure type that holds units inside (`StructureDef::holds`, plan decision T60; `crate::holding`): one
/// capability for every such building. The Armory's (`factions/cults/structures/armory.md`); later the Altar, the
/// Temple, the Cathedral, the Shrine, the Garrison, the Chop Shop and the Airstrip (`factions/cults/economy.md > Cults
/// Building Mechanics > Evacuation on Destruction`; design change 0319 gives their numbers), and the GDO's Bunker.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HoldsDef {
    /// `EntranceSide`, as its SymmetryType letters it: the side the units walk in at
    /// (`factions/cults/structures/armory.md > EntranceSide - one short side (B)`: "Recruits enter through the
    /// entrance side"), by the entry path in its middle (`crate::holding::entry_path`). Of a type whose every side has
    /// one letter (AAAA), the first side so lettered clockwise from the top.
    pub entrance: SideLetter,
    /// How the units come out (`ExitSide`): the exit rule of a producing structure, the one vocabulary for how a
    /// structure releases units. Only an exit side is built (M5-F1, `defs_are_consistent`): the units come out "one
    /// at a time through the exit doorway" in its middle (`factions/cults/structures/armory.md > ExitSide - opposite
    /// short side (C)`; `core/entities.md > Structure Type > Exit Doorway`; `production::Doorway`). A type that also
    /// produces releases by the same rule (`ProductionDef::exits`). A Garrison's "Units leave through any side, at
    /// the first clear spot next to the footprint (like Evacuation on Destruction)"
    /// (`factions/cults/structures/garrison.md`) will be a rule of its own, by the same search as an evacuation
    /// (`Sim::spot_around`).
    pub exit: ExitRule,
    /// How many units it holds (`factions/cults/structures/armory.md > InternalRecruitCapacity - 10`), those a queued
    /// order reserved and the one in training counted (design change 0347 point 5: "Capacity counts reserved units:
    /// the queue can never exceed the capacity"), so that it bounds the queue of a producer that holds units too.
    pub capacity: u32,
    /// The kinds of unit it takes in (`systems/controls/commands.md > Enter`: "Cults — Armory, Cathedral: Recruits
    /// only").
    #[serde(deserialize_with = "super::leak::slice")]
    pub accepts: &'static [ObjectKind],
}

/// A structure type's pop-up form (`StructureDef::popup`, slice M4-S2; `crate::popup`): the Laser Turret's
/// (`factions/syndicate/structures/laser_turret.md`: "Pop-up: rises in 8 frames (already facing its target); retracts
/// after 32 frames with no target, taking 8 frames").
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PopUpDef {
    /// How long it takes to rise, ticks.
    pub rise_frames: u32,
    /// How long it stays raised with no target before it retracts, ticks.
    pub idle_frames: u32,
    /// How long it takes to retract, ticks.
    pub retract_frames: u32,
    /// Its armour while fully lowered, underground; rising, raised and retracting it has its type's
    /// (`ObjectDef::armor`).
    pub lowered_armor: Armor,
}

/// A Tunnel expansion's construction (`StructureDef::expansion`, slice M4-F): a Tunnel builds it in its Tunnel Area,
/// on a site that is the expansion from its first frame (plan decision T31, `crate::construction::site`), paid when its
/// ghost is placed (`factions/syndicate/tunnels.md > Expansions under construction`, design change 0254).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExpansionDef {
    /// `Tier Requirement`: the lowest tier of a Tunnel that builds it (`factions/syndicate/structures/README.md >
    /// Tunnel Expansions > Expansion Requirements`: "checked against the building Tunnel").
    pub tier: u8,
    /// `Cost`, Space Crystals, paid as its ghost is placed (0254: "Paid when the ghost is placed").
    pub cost: u32,
    /// `Build Time`, ticks: the site completes at the end of its `frames`-th tick, as a worker's (`SiteDef::frames`).
    pub frames: u32,
    /// `core/entities.md > ConstructionHP Rule`: the share of its MaxHP its site starts at (0254: "the ConstructionHP
    /// Rule applies to every expansion (starts at 10%, grows with progress)"), as a worker's (`SiteDef::start_hp`).
    pub start_hp: f32,
}

/// A worker-built structure type's construction on its site (`StructureDef::site`, plan decisions T31 and T51).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SiteDef {
    /// What an order to build one costs.
    pub price: SitePrice,
    /// The construction time, ticks: the site completes at the end of its `frames`-th tick.
    pub frames: u32,
    /// `core/entities.md > ConstructionHP Rule`: the share of its MaxHP a site starts at, "10%". Its MaxHP grows
    /// linearly from there to the whole as the construction progresses ("HP during construction = MaxHP x (10% + 90% x
    /// construction_progress)"), and the damage it takes stays taken (design change 0234).
    pub start_hp: f32,
    /// Where the builder goes as the construction completes (plan decision T51).
    pub builder_goes: BuilderGoes,
}

/// What an order to build a worker-built structure costs (`SiteDef::price`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SitePrice {
    /// Supplies, one for each structure of the type its owner has: those standing, at any tier, those under
    /// construction, and the lingering ghosts of the orders to build one (the Tunnel's: `factions/syndicate/tunnels.md >
    /// Cost Scaling > Construction Cost`, design change 0233). An order that goes without its construction starting,
    /// and a site cancelled, refund the price as it is once they are gone ("the current construction price, not
    /// counting itself", `factions/syndicate/tunnels.md > Upgrade Costs`).
    SuppliesPerOwned,
    /// A fixed price in Space Crystals: the Laser Turret's 100 (`factions/syndicate/structures/laser_turret.md`:
    /// "Cost: 100 Space Crystals (no Supplies)"; slice M4-S1). Paid as the order is given, "with the same refunds" as
    /// an Agent's Tunnel (design change 0297 point 4, `factions/syndicate/structures/laser_turret.md > Construction`):
    /// an order that goes before its site starts gives back the current top price, all of it, the price having no
    /// scaling. A cancelled site gives back "75%" of it, rounded down, "like an expansion" (0297 point 5;
    /// `construction::site::BuildPrice::cancelled`).
    Crystals(u32),
}

/// Where a worker-built structure's builder goes as the construction completes (plan decision T51).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuilderGoes {
    /// Into its owner's Tunnel Network: the Tunnel's Agent (`factions/syndicate/units/agent.md > Building > Tunnel
    /// Construction Flow`: "the Agent is inside the Tunnel Network, available for redeployment from any Tunnel").
    Network,
    /// Out onto the surface, beside the finished structure: the Laser Turret's Agent
    /// (`factions/syndicate/structures/laser_turret.md > Construction`: "the Agent emerges on the surface adjacent to
    /// the finished structure (unlike Tunnels, where the Agent ends up inside the Tunnel Network)"): "at the **first
    /// clear spot next to the turret**, searched in a fixed order starting from the side it came from; if none is
    /// clear, it waits inside until one is" (design change 0298 point 7; `sim/sites.rs`).
    Beside,
    /// Nowhere: its builders build it from outside and stay on the field (plan decision T59, slice M5-E;
    /// `factions/cults/economy.md > Cults Building Mechanics`: "Assigned Recruits are not consumed — they remain on the
    /// field near the site for the duration of construction"; "On completion, all Recruits that participated remain on
    /// the field as ordinary Recruits, free to be issued new orders"). Its site's builder is
    /// `construction::site::Builder::Recruits`.
    Outside,
}

/// Where a structure type takes workers' loads (`StructureDef::drop_off`; plan decision T34: "the drop-off rule comes
/// from the target's def").
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DropOff {
    /// Anywhere at the structure, any number of workers at once (`factions/colonists/units/drone.md > Gathering`:
    /// DropOffReach is measured to the Egress Node's edge).
    Anywhere,
    /// At a hatch on one of its sides, one for each resource (`factions/syndicate/tunnels.md > Tunnel > SymmetryType
    /// - ABCD`, design change 0248: "Each resource must be delivered to its own hatch: crystals only at Side B,
    /// Supplies only at Side D"; "Each hatch is 1 grid square long, near the middle of its side", 0255 point 6;
    /// "Only one Agent may drop off resources at a side at a time"). `crate::economy::drop_off` has the square.
    Hatches {
        /// The side of the Space Crystals' hatch.
        crystals: SideLetter,
        /// The side of the Supplies' hatch (slice M4-I delivers there).
        supplies: SideLetter,
    },
}

impl DropOff {
    /// The side of the hatch that takes `resource`; `None` for a drop-off that takes loads anywhere.
    pub fn hatch(self, resource: Resource) -> Option<SideLetter> {
        match (self, resource) {
            (DropOff::Anywhere, _) => None,
            (DropOff::Hatches { crystals, .. }, Resource::Crystals) => Some(crystals),
            (DropOff::Hatches { supplies, .. }, Resource::Supplies) => Some(supplies),
        }
    }
}

/// `factions/syndicate/tunnels.md > Tunnel > Tiers`: "Tunnels have 3 upgrade tiers. Each tier increases HP, Tunnel
/// Area radius, and unlocks higher-tier buildings and unit transit." One Tunnel type with a tier (plan decision T39),
/// not a type per tier. The tiers' PointArmor and FullArmor are the same (1 and 16): the type's (`ObjectDef::armor`).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TunnelDef {
    /// Tier 1, the base, first: tier `n` is `tiers[n - 1]` ([`TunnelDef::tier`]).
    #[serde(deserialize_with = "super::leak::slice")]
    pub tiers: &'static [TierDef],
    /// `factions/syndicate/tunnels.md > Tunnel Network > Transit delay` (design change 0245): how long after it walks
    /// into the network a unit can first exit, ticks, from any Tunnel; none for a unit produced inside (0246;
    /// `network::Held::may_exit`).
    pub transit_delay: Tick,
    /// `factions/syndicate/tunnels.md > Tunnel Network > Regeneration inside` (design change 0249): the fraction of its
    /// MaxHP that a unit inside the network regenerates a second (`network::Held::regenerate`).
    pub regen_per_second: f32,
}

impl TunnelDef {
    /// The row of tier `tier` (1 for the base), if the table has one.
    pub fn tier(&self, tier: u8) -> Option<&'static TierDef> {
        self.tiers.get(usize::from(tier).checked_sub(1)?)
    }
}

/// One column of `factions/syndicate/tunnels.md > Tunnel > Tiers`.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TierDef {
    /// `HP`: the Tunnel's MaxHP at this tier (`Object::max_hp`).
    pub max_hp: f32,
    /// `Tunnel Area Radius`, grid units: how far the Tunnel Area reaches out from the Tunnel's footprint on every side
    /// (`factions/syndicate/tunnels.md > Tunnel Area`: "a square of (radius + 4 + radius) per side"). Data only until
    /// the Tunnel Area exists (slices M4-B and M4-F).
    pub area_radius: u32,
    /// `Transit`: the classes of unit that may enter and leave the Tunnel Network through a Tunnel of this tier
    /// (`factions/syndicate/tunnels.md > Tunnel Network > Transit Tier Requirements`). Data only until units enter the
    /// network (slice M4-C).
    #[serde(deserialize_with = "super::leak::slice")]
    pub transit: &'static [Transit],
    /// How a Tunnel upgrades to this tier from the one below (`factions/syndicate/tunnels.md > Cost Scaling > Upgrade
    /// Costs`, slice M4-G): `None` for the base tier, which no upgrade reaches.
    #[serde(default)]
    pub upgrade: Option<UpgradeDef>,
}

/// `factions/syndicate/tunnels.md > Cost Scaling > Upgrade Costs`: a Tunnel's upgrade to a tier ([`TierDef::upgrade`],
/// slice M4-G). Data only: the sim prices it (`Sim::upgrade_price`) and counts its frames (`tunnel::Operation`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpgradeDef {
    /// "**Upgrade duration**": how long it takes at full speed, frames (ticks).
    pub frames: u32,
    /// The price's base, in Supplies: the first 2 of "**Upgrade to Tier 2**: 2 + 2 x (number of T2+ Tunnels owned)".
    pub base: u32,
    /// What each of the owner's other Tunnels at this tier or above adds to the price, one upgrading to it included
    /// ("**Upgrades in progress count**"): the second 2 of the same.
    pub per_tunnel: u32,
}

impl TierDef {
    /// Whether a unit of `base` may pass through a Tunnel of this tier ([`Transit::of`]).
    pub fn admits(&self, base: UnitBase) -> bool {
        self.transit.contains(&Transit::of(base))
    }
}

/// `factions/syndicate/tunnels.md > Tunnel Network > Transit Tier Requirements`: the classes of unit base that a
/// Tunnel's tier lets into and out of the network.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Transit {
    /// "Tier 1+: Infantry (Light Infantry, Heavy Infantry)".
    Infantry,
    /// "Tier 2+: Vehicles (Wheeled, Tracked, Drill, Hover Vehicle, Mech)".
    Vehicles,
    /// "Tier 3+: Air units (Hover Craft, Glider)".
    Air,
}

impl Transit {
    /// The class of a unit of `base`, as the requirements list the bases: its base's (`UnitBaseDef::transit`).
    pub fn of(base: UnitBase) -> Transit {
        base.def().transit
    }
}

/// A mining structure's rates (`factions/gdo/economy.md > ExtractionPlate`): it pays its owner a load at the end of
/// each cycle, taken from its patch, and the residual amount once the patch is depleted.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MiningDef {
    /// `MiningRate`: Space Crystals per cycle, while the patch holds that many (the last load takes what is left).
    pub load: u32,
    /// `ResidualMiningRate`: Space Crystals per cycle once the patch is depleted.
    pub residual: u32,
    /// The length of a cycle at full power, ticks.
    pub cycle_ticks: u32,
}

/// A producing structure's recipes and its exits (`factions/gdo/structures/barracks.md > Produces`,
/// `core/entities.md > Structure Type > Exit Doorway`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductionDef {
    /// Where its units come into the game and which way they leave: by the doorway in the middle of an exit side,
    /// or from an energy doorway at the centre through its open sides, the sides as its SymmetryType letters them
    /// (`crate::sides::sides` says where each faces once the building is placed; `crate::production::exit`).
    pub exits: ExitRule,
    /// What it can produce, in the order of the design's list.
    #[serde(deserialize_with = "super::leak::slice")]
    pub recipes: &'static [Recipe],
    /// A Recruitment Hall's automatic production (`factions/cults/economy.md > RecruitmentHall`; plan decision T57,
    /// slice M5-C; `crate::recruitment`): it trains nothing on command, and produces its first recipe's unit by
    /// itself, for free, while the units of its lineage use less than its capacity. `None` for a producer that trains
    /// what it is told. Optional in files.
    #[serde(default)]
    pub recruitment: Option<RecruitmentDef>,
}

impl ProductionDef {
    /// The recipe for a unit of `kind`, if the type produces it.
    pub fn recipe(&self, kind: ObjectKind) -> Option<&'static Recipe> {
        self.recipes.iter().find(|recipe| recipe.kind == kind)
    }
}

/// How a Recruitment Hall recruits (`factions/cults/economy.md > RecruitmentHall`; `crate::recruitment`): the size of
/// the area it claims tiles in and its capacity at full effectiveness. What it recruits, in how many frames at full
/// effectiveness and at what cost, is its `ProductionDef`'s recipe.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecruitmentDef {
    /// The side of its Recruitment Area, in tiles, centred on its footprint (`factions/cults/economy.md >
    /// RecruitmentHall > RecruitmentArea`: "A 10x10 grid unit area centered on the 4x4 building").
    pub area: u32,
    /// Its LocalUnitControlCapacity at 100% effectiveness (`factions/cults/economy.md > RecruitmentHall >
    /// BaseUnitControlCapacity - 20`).
    pub capacity: u32,
}

impl RecruitmentDef {
    /// "The total number of tiles in Recruitment Area" (`factions/cults/economy.md > RecruitmentHall > Effectiveness`):
    /// the area's side squared, those off the map included (**Placeholder (Q0099 point 1)**).
    pub fn tiles(&self) -> u32 {
        self.area.saturating_mul(self.area)
    }
}

/// How a producing structure releases its units, and through which of its sides (plan decision T19;
/// `core/entities.md > Structure Type > Exit Doorway`; `crate::production::exit`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExitRule {
    /// One exit side, the only one its SymmetryType letters so, with the exit doorway in its middle (design change
    /// 0080): the Barracks' B ("Units exit from one of the short sides (B side ...)",
    /// `factions/gdo/structures/barracks.md`).
    Side(SideLetter),
    /// Several exit sides, every one its SymmetryType letters so, each with an exit doorway in its middle
    /// (`core/entities.md > Structure Type > Exit Doorway`: "Buildings with several exit-eligible sides choose the side
    /// by their own rules; a side whose doorway is occupied counts as blocked, and the building pauses only when every
    /// eligible doorway is occupied"): a unit leaves through the one facing its producer's rally point, the bottom one
    /// without a rally point, or the next free one clockwise from it while that one's doorway is occupied
    /// (`crate::production::Doorway::order`; slice M5-C's rule for the Recruitment Hall's four A sides, which design
    /// change 0342 replaced with one exit side, B). No type has it now: **Placeholder (not in the design: the order of
    /// several exit sides, for a building that will have them)**.
    Sides(SideLetter),
    /// An energy doorway at the centre (design change 0145, `factions/colonists/structures/README.md > Portals > Portal
    /// Exits (energy doorway)`: "Portals don't use exit sides"): a unit materializes at the centre and leaves in any
    /// direction that passes out through a side lettered with one of these, the **open** sides. The portals' A sides;
    /// the Heavy and Star Portals' are TBD.
    Centre(#[serde(deserialize_with = "super::leak::slice")] &'static [SideLetter]),
    /// Into its owner's Tunnel Network (`factions/syndicate/structures/README.md > Rally Point Behavior`, plan
    /// decision T35; `crate::network`): the unit never appears at the building, which is underground and has no doorway
    /// (`core/entities.md > Exit Doorway`: "units released underground (e.g. Syndicate Tunnels) are not covered by
    /// this rule"). It stays in the network, or comes out of the parent Tunnel's exit path for a rally point on the
    /// surface. The Hideout's.
    Network,
}

/// One entry of a producer's `Produces` list: "Peacekeeper: 50 Space Crystals, 80 frames (5 seconds)"; of a
/// constructor's `Constructs` list: "Generator: 100 Space Crystals, 240 frames (15 seconds)"; or of the Beacon's Warp
/// In list: "Egress Node | 75 Space Crystals | 320 frames (20 seconds)".
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recipe {
    pub kind: ObjectKind,
    /// Space Crystals, paid when the unit is queued or the construction starts.
    pub cost: u32,
    /// The production or construction time at full power, or the WarpInTime, ticks.
    pub frames: u32,
    /// The types its player must own, each at least once, before the entry can be queued or started (plan decision
    /// T37, slice M4-K; `crate::tech`): a unit's `Unit Prerequisite` (`core/tech_and_research.md > Tech Prerequisites >
    /// Unit Prerequisites`: "A unit type's production option may also declare a TechPrerequisite"), such as the
    /// Hideout's Guard's Optics Lab, or a constructor's "Requires" clause
    /// (`factions/gdo/structures/deployment_center.md > Constructs`). A building's own TechPrerequisite is its def's
    /// (`StructureDef::requires`), which holds wherever it is built. Empty for none.
    #[serde(deserialize_with = "super::leak::slice")]
    pub requires: &'static [ObjectKind],
}

/// `core/entities.md > Structure Type > SymmetryType`: which of a structure's four sides are alike. The letters name
/// the sides clockwise from the top edge, in the default orientation (rotation 0, unflipped): top, right, bottom,
/// left (`> Side lettering convention`, design change 0079). The placed sides are [`crate::sides::sides`]'.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SymmetryType {
    AAAA,
    AAAB,
    AABB,
    ABAB,
    AABC,
    ABAC,
    ABCD,
}

impl SymmetryType {
    /// The letters of the four sides in the default orientation: top, right, bottom, left.
    pub const fn letters(self) -> [SideLetter; 4] {
        use SideLetter::{A, B, C, D};
        match self {
            SymmetryType::AAAA => [A, A, A, A],
            SymmetryType::AAAB => [A, A, A, B],
            SymmetryType::AABB => [A, A, B, B],
            SymmetryType::ABAB => [A, B, A, B],
            SymmetryType::AABC => [A, A, B, C],
            SymmetryType::ABAC => [A, B, A, C],
            SymmetryType::ABCD => [A, B, C, D],
        }
    }

    /// Whether a type of this symmetry may have a footprint that isn't square: "Height and Width can be non-equal
    /// for symmetry types ABAB, ABAC and ABCD" (`core/entities.md > Structure Type`); AAAA "requires for Height and
    /// Width to be equal", and so do the others.
    pub const fn allows_oblong(self) -> bool {
        matches!(self, SymmetryType::ABAB | SymmetryType::ABAC | SymmetryType::ABCD)
    }
}
