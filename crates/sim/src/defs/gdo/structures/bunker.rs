//! The Bunker — `factions/gdo/structures/bunker.md` (design change 0366, slice M6-GK): "GDO field defensive
//! structure built by a Field Engineer. Infantry garrison inside and fire out." Its numbers are the design's
//! "Placeholder stats" (plan decision T84). No Field Engineer builds one, and it holds nobody, until slice M6-GB: its
//! site, its garrison (`> Garrison`: "up to **4 units**", firing out) and its Field Engineer entry ([`RECIPE`], held)
//! are that slice's.

use crate::defs::structures::{Recipe, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/gdo/structures/bunker.md`: its placeholder stats (design change 0366).
pub(crate) const BUNKER: ObjectDef = ObjectDef {
    kind: ObjectKind::Bunker,
    name: "Bunker",
    faction: Some(Faction::Gdo), // Faction - GlobalDefenseOrganization
    category: Category::Structure,
    footprint: Footprint::Tiles { w: 2, h: 2 }, // Size 2x2
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    destructible: true,
    groupable: true,     // Groupable true
    sight_range: 6.0,    // SightRange 6 grid units
    max_hp: Some(500.0), // MaxHP 500
    armor: Armor {
        point: 2.0, // PointArmor 2
        full: 8.0,  // FullArmor 8
    },
    // "Can be placed on **any Buildable terrain**": a building's, on flat ground (`core/map.md > Elevation >
    // Structures only on flat terrain`).
    needs_flat_ground: true,
    structure: Some(StructureDef {
        symmetry: SymmetryType::AAAB, // SymmetryType AAAB with **one door on side B** (as the Cults Garrison)
        // BuildRadiusExtension - 0 (does not extend the GDO build area, so Field Engineers cannot use Bunkers to
        // expand)
        build_radius_extension: 0,
        power: 0, // Power 0
        requires: &[],
        ..StructureDef::NONE
    }),
    ..ObjectDef::NONE
};

/// `> Placeholder stats`: "Cost: Field Engineer, 100 Space Crystals, 320 frames (20 seconds)". Held until slice
/// M6-GB (`crate::defs::PLANNED_RECIPES`).
pub(crate) const RECIPE: Recipe = Recipe {
    kind: ObjectKind::Bunker,
    cost: 100,
    frames: 320,
    requires: &[],
    supplies: 0,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// The placeholder stat block, its one door on B; it extends no build area, holds nobody and has no site yet; its
    /// Field Engineer entry, 100 Space Crystals and 320 frames, is held.
    #[test]
    fn the_bunker_follows_the_design() {
        let structure = super::super::assert_stat_block(
            ObjectKind::Bunker,
            (2, 2),
            SymmetryType::AAAB,
            500.0,
            (2.0, 8.0),
            6.0,
            true,
            0,
            0,
        );
        assert!(structure.holds.is_none() && structure.site.is_none());
        assert_eq!(RECIPE, super::super::held(ObjectKind::Bunker, 100, 320));
    }
}
