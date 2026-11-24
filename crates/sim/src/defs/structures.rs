//! The attributes of structure types (`core/entities.md > Structure Type`). The faction modules (`defs/gdo.rs`, …)
//! hold the structure types themselves.

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
