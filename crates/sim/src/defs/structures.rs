//! The attributes of structure types (`core/entities.md > Structure Type`). The faction modules (`defs/gdo.rs`, …)
//! hold the structure types themselves.

use crate::defs::{ObjectKind, UnitBase};
use crate::sides::SideLetter;

/// A structure type's own attributes; the attributes every object type has, armour included, are on its
/// `ObjectDef`.
#[derive(Clone, Debug, PartialEq)]
pub struct StructureDef {
    /// `core/entities.md > Structure Type > SymmetryType`.
    pub symmetry: SymmetryType,
    /// `BuildRadiusExtension`, grid units: how far the type grows its owner's build area around its footprint
    /// (`factions/gdo/structures.md > GDOBuildArea`). 0 for none.
    pub build_radius_extension: u32,
    /// `Power`: what the type adds to its owner's power, positive for a generator, negative for a consumer
    /// (`factions/gdo/economy.md`).
    pub power: i32,
    /// What the type produces from its queue, and through which side (`crate::production`): the Barracks' units,
    /// `None` for a structure that produces nothing.
    pub production: Option<ProductionDef>,
    /// How the type mines the Space Crystals patch it stands on (`crate::economy::mining`): the Extraction Plate's
    /// rates, `None` for every other structure.
    pub mining: Option<MiningDef>,
    /// `Constructs`: the buildings the type constructs, one at a time (`crate::construction`), with their costs and
    /// times (`factions/gdo/structures.md > DeploymentCenter > Constructs`, `factions/gdo/economy.md >
    /// ExtractionFacility > Constructs`). A type with any is a constructor (`Object::constructor`); empty for the
    /// others.
    pub constructs: &'static [Recipe],
    /// The Beacon's `Constructs List` (`factions/colonists/economy.md > Warp In > Constructs List`): the buildings a
    /// Beacon transforms into in place, with their costs and WarpInTimes; empty for every other type. Not
    /// [`StructureDef::constructs`], which makes its type a GDO-style constructor (plan decision T14): a Warp In
    /// turns the Beacon itself into the building (`crate::transform`, plan decision T15).
    pub warp_in: &'static [Recipe],
    /// Whether workers deliver their loads to the type (`factions/colonists/economy.md > Egress Node > Drop Off`:
    /// "the only way resources enter the player's resource pool"; `crate::economy::gathering`).
    pub drop_off: bool,
    /// Whether the type is an underground structure (plan decision T32; `factions/syndicate/structures.md > Tunnel
    /// Expansions`: "Expansions are invisible to enemies without detection and can be walked over by surface units"):
    /// its footprint blocks no movement (`nav::BlockerKind::UndergroundExpansion`), and while it has an owner it is
    /// hidden from the other players (`vision::hidden_from`). The Syndicate's expansions: the Hideout so far.
    pub underground: bool,
    /// The Tunnel's tier table (plan decision T39, `factions/syndicate/tunnels.md > Tunnel > Tiers`): an object of a
    /// type with one carries its tier (`Object::tunnel`, `crate::tunnel`). `None` for every other type.
    pub tunnel: Option<&'static TunnelDef>,
}

/// `factions/syndicate/tunnels.md > Tunnel > Tiers`: "Tunnels have 3 upgrade tiers. Each tier increases HP, Tunnel
/// Area radius, and unlocks higher-tier buildings and unit transit." One Tunnel type with a tier (plan decision T39),
/// not a type per tier. The tiers' PointArmor and FullArmor are the same (1 and 16): the type's (`ObjectDef::armor`).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TunnelDef {
    /// Tier 1, the base, first: tier `n` is `tiers[n - 1]` ([`TunnelDef::tier`]).
    pub tiers: &'static [TierDef],
}

impl TunnelDef {
    /// The row of tier `tier` (1 for the base), if the table has one.
    pub fn tier(&self, tier: u8) -> Option<&'static TierDef> {
        self.tiers.get(usize::from(tier).checked_sub(1)?)
    }
}

/// One column of `factions/syndicate/tunnels.md > Tunnel > Tiers`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TierDef {
    /// `HP`: the Tunnel's MaxHP at this tier (`Object::max_hp`).
    pub max_hp: f32,
    /// `Tunnel Area Radius`, grid units: how far the Tunnel Area reaches out from the Tunnel's footprint on every side
    /// (`> Tunnel Area`: "a square of (radius + 4 + radius) per side"). Data only until the Tunnel Area exists (slices
    /// M4-B and M4-F).
    pub area_radius: u32,
    /// `Transit`: the classes of unit that may enter and leave the Tunnel Network through a Tunnel of this tier
    /// (`> Tunnel Network > Transit Tier Requirements`). Data only until units enter the network (slice M4-C).
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
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Transit {
    /// "Tier 1+: Infantry (Light Infantry, Heavy Infantry)".
    Infantry,
    /// "Tier 2+: Vehicles (Wheeled, Tracked, Drill, Hover Vehicle, Mech)".
    Vehicles,
    /// "Tier 3+: Air units (Hover Craft, Glider)".
    Air,
}

impl Transit {
    /// The class of a unit of `base`, as the requirements list the bases.
    pub fn of(base: UnitBase) -> Transit {
        match base {
            UnitBase::LightInfantry | UnitBase::HeavyInfantry => Transit::Infantry,
            UnitBase::WheeledVehicle
            | UnitBase::TrackedVehicle
            | UnitBase::DrillUnit
            | UnitBase::HoverVehicle
            | UnitBase::Mech => Transit::Vehicles,
            UnitBase::HoverCraft | UnitBase::Glider => Transit::Air,
        }
    }
}

/// A mining structure's rates (`factions/gdo/economy.md > ExtractionPlate`): it pays its owner a load at the end of
/// each cycle, taken from its patch, and the residual amount once the patch is depleted.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MiningDef {
    /// `MiningRate`: Space Crystals per cycle, while the patch holds that many (the last load takes what is left).
    pub load: u32,
    /// `ResidualMiningRate`: Space Crystals per cycle once the patch is depleted.
    pub residual: u32,
    /// The length of a cycle at full power, ticks.
    pub cycle_ticks: u32,
}

/// A producing structure's recipes and its exits (`factions/gdo/structures.md > Barracks > Produces`,
/// `core/entities.md > Structure Type > Exit Doorway`).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ProductionDef {
    /// Where its units come into the game and which way they leave: by the doorway in the middle of an exit side,
    /// or from an energy doorway at the centre through its open sides, the sides as its SymmetryType letters them
    /// (`crate::sides::sides` says where each faces once the building is placed; `crate::production::exit`).
    pub exits: ExitRule,
    /// What it can produce, in the order of the design's list.
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
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ExitRule {
    /// One exit side, the only one its SymmetryType letters so, with the exit doorway in its middle (design change
    /// 0080): the Barracks' B ("Units exit from one of the short sides (B side ...)", `factions/gdo/structures.md >
    /// Barracks`).
    Side(SideLetter),
    /// An energy doorway at the centre (design change 0145, `factions/colonists/structures.md > Portals > Portal Exits
    /// (energy doorway)`: "Portals don't use exit sides"): a unit materializes at the centre and leaves in any
    /// direction that passes out through a side lettered with one of these, the **open** sides. The portals' A sides;
    /// the Heavy and Star Portals' are TBD.
    Centre(&'static [SideLetter]),
}

/// One entry of a producer's `Produces` list: "Peacekeeper: 50 Space Crystals, 80 frames (5 seconds)"; of a
/// constructor's `Constructs` list: "Generator: 100 Space Crystals, 240 frames (15 seconds)"; or of the Beacon's Warp
/// In list: "Egress Node | 75 Space Crystals | 320 frames (20 seconds)".
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
