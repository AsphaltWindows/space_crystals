//! The production capability of structure types (`StructureDef::production`): what a producing structure makes
//! ([`ProductionDef`], its [`Recipe`]s and an Altar's [`AugmentRecipe`]s), a Recruitment Hall's recruiting
//! ([`RecruitmentDef`]), and how a structure releases units ([`ExitRule`]). Moved out of `defs/structures.rs` whole
//! (slice M5-N3, plan decision T28), unchanged.

use serde::{Deserialize, Serialize};

use crate::augmentation::AugmentOrder;
use crate::defs::ObjectKind;
use crate::sides::SideLetter;

/// A producing structure's recipes and its exits (`factions/gdo/structures/barracks.md > Produces`,
/// `core/entities.md > Structure Type > Exit Doorway`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductionDef {
    /// Where its units come into the game and which way they leave: by the doorway in the middle of an exit side,
    /// or from an energy doorway at the centre through its open sides, the sides as its SymmetryType letters them
    /// (`crate::sides::sides` says where each faces once the building is placed; `crate::production::exit`).
    pub exits: ExitRule,
    /// What it can produce, in the order of the design's list.
    #[serde(deserialize_with = "crate::defs::leak::slice")]
    pub recipes: &'static [Recipe],
    /// A Recruitment Hall's automatic production (`factions/cults/structures/recruitment_hall.md`; plan decision T57,
    /// slice M5-C; `crate::recruitment`): it trains nothing on command, and produces its first recipe's unit by
    /// itself, for free, while the units of its lineage use less than its capacity. `None` for a producer that trains
    /// what it is told. Optional in files.
    #[serde(default)]
    pub recruitment: Option<RecruitmentDef>,
    /// (M5-N1b) The augmentation orders it takes of the units inside, the Altar's (`factions/cults/augmentation.md >
    /// Altar > Augmentation Orders`; `QueueEntry::Augment`), in the order of the design's list. Empty for every other
    /// producer. Optional in files.
    #[serde(default, deserialize_with = "crate::defs::leak::slice")]
    pub augment: &'static [AugmentRecipe],
}

impl ProductionDef {
    /// The recipe for a unit of `kind`, if the type produces it.
    pub fn recipe(&self, kind: ObjectKind) -> Option<&'static Recipe> {
        self.recipes.iter().find(|recipe| recipe.kind == kind)
    }

    /// (M5-N1b) The recipe for an `order`, if the type takes it.
    pub fn augment_recipe(&self, order: AugmentOrder) -> Option<&'static AugmentRecipe> {
        self.augment.iter().find(|recipe| recipe.order == order)
    }
}

/// How a Recruitment Hall recruits (`factions/cults/structures/recruitment_hall.md`; `crate::recruitment`): the size of
/// the area it claims tiles in and its capacity at full effectiveness. What it recruits, in how many frames at full
/// effectiveness and at what cost, is its `ProductionDef`'s recipe.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecruitmentDef {
    /// The side of its Recruitment Area, in tiles, centred on its footprint
    /// (`factions/cults/structures/recruitment_hall.md > RecruitmentArea`: "A 10x10 grid unit area centered on the 4x4
    /// building").
    pub area: u32,
    /// Its LocalUnitControlCapacity at 100% effectiveness (`factions/cults/structures/recruitment_hall.md >
    /// BaseUnitControlCapacity - 20`).
    pub capacity: u32,
}

impl RecruitmentDef {
    /// "The total number of tiles in Recruitment Area" (`factions/cults/structures/recruitment_hall.md >
    /// Effectiveness`): the area's side squared. The whole area lies on the map (design change 0341 point 1,
    /// `crate::placement`), but for a Hall that takes no placement check, a debug spawn's, whose tiles off the map
    /// count among these (**Placeholder (not in the design: 0341 rules such a Hall out)**).
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
    Centre(#[serde(deserialize_with = "crate::defs::leak::slice")] &'static [SideLetter]),
    /// Into its owner's Tunnel Network (`factions/syndicate/structures/README.md > Rally Point Behavior`, plan
    /// decision T35; `crate::network`): the unit never appears at the building, which is underground and has no doorway
    /// (`core/entities.md > Exit Doorway`: "units released underground (e.g. Syndicate Tunnels) are not covered by
    /// this rule"). It stays in the network, or comes out of the parent Tunnel's exit path for a rally point on the
    /// surface. The Hideout's.
    Network,
    /// (M5-N2) One door, the only side its SymmetryType letters so, with its doorway in its middle as
    /// [`ExitRule::Side`] has, which is the building's entrance too: the Garrison's B
    /// (`factions/cults/structures/garrison.md`: "SymmetryType AAAB, **the door on side B** (one door, entrance and
    /// exit"). Everyone it lets out leaves **at once** (`> Placeholder stats > Leaving`: "units leave **through the
    /// door**; on **Eject All** they all come out **at once**, ignoring collision with each other until they are clear
    /// of the footprint"; `Sim::step_holding`). Only for a building that holds units and produces none
    /// (`defs_are_consistent`).
    AtOnce(SideLetter),
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
    #[serde(deserialize_with = "crate::defs::leak::slice")]
    pub requires: &'static [ObjectKind],
}

/// (M5-N1b) One of the orders an Altar takes (`ProductionDef::augment`; `factions/cults/augmentation.md > Altar >
/// Augmentation Orders`): "Cost and time per order type".
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AugmentRecipe {
    pub order: AugmentOrder,
    /// Space Crystals, paid when the order is queued ("Queuing an order immediately deducts its Space Crystals cost").
    pub cost: u32,
    /// The time it takes, ticks.
    pub frames: u32,
}
