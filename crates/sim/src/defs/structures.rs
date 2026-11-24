//! The attributes of structure types (`core/entities.md > Structure Type`). The faction modules (`defs/gdo.rs`, …)
//! hold the structure types themselves.

use crate::defs::ObjectKind;
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
    /// [`StructureDef::constructs`], which makes its type a GDO-style constructor (plan decision T14). Data only
    /// until slice M3-E warps them in.
    pub warp_in: &'static [Recipe],
    /// Whether workers deliver their loads to the type (`factions/colonists/economy.md > Egress Node > Drop Off`:
    /// "the only way resources enter the player's resource pool"; `crate::economy::gathering`).
    pub drop_off: bool,
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

/// A producing structure's recipes and its exit (`factions/gdo/structures.md > Barracks > Produces`,
/// `core/entities.md > Structure Type > Exit Doorway`).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ProductionDef {
    /// The side its units leave through, as its SymmetryType letters it (`crate::sides::sides` says where that
    /// side faces once the building is placed): the exit doorway is in the middle of this side. A letter that
    /// several sides share (the portals' A) names the first of them clockwise from north as placed: a placeholder
    /// until slice M3-D gives the portals their exit rule (plan decision T19).
    pub exit: SideLetter,
    /// What it can produce, in the order of the design's list.
    pub recipes: &'static [Recipe],
}

impl ProductionDef {
    /// The recipe for a unit of `kind`, if the type produces it.
    pub fn recipe(&self, kind: ObjectKind) -> Option<&'static Recipe> {
        self.recipes.iter().find(|recipe| recipe.kind == kind)
    }
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
