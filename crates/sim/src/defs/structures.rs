//! The attributes of structure types (`core/entities.md > Structure Type`). The faction modules (`defs/gdo.rs`, …)
//! hold the structure types themselves.

use serde::{Deserialize, Serialize};

use crate::defs::{ObjectKind, Resource, UnitBase};
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
    /// (`UnitDef::deploy`, T51). The Tunnel's so far; the Laser Turret's comes with slice S1.
    pub site: Option<SiteDef>,
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
}

/// Where a worker-built structure's builder goes as the construction completes (plan decision T51).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuilderGoes {
    /// Into its owner's Tunnel Network: the Tunnel's Agent (`factions/syndicate/units/agent.md > Building > Tunnel
    /// Construction Flow`: "the Agent is inside the Tunnel Network, available for redeployment from any Tunnel").
    Network,
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
}

impl ProductionDef {
    /// The recipe for a unit of `kind`, if the type produces it.
    pub fn recipe(&self, kind: ObjectKind) -> Option<&'static Recipe> {
        self.recipes.iter().find(|recipe| recipe.kind == kind)
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
