//! The First Aid Tent — `factions/gdo/structures/first_aid_tent.md` (design change 0366, slice M6-GK): "GDO support
//! structure set up by a Field Engineer. Heals nearby infantry." Its numbers are the design's "Placeholder stats"
//! (plan decision T84). No Field Engineer sets one up, and it heals nobody, until slice M6-GB: its site, its healing
//! ("whose centre is within **3 grid units**: **+2 HP per second**") and its Field Engineer entry ([`RECIPE`], held)
//! are that slice's.

use crate::defs::structures::{Recipe, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/gdo/structures/first_aid_tent.md`: its placeholder stats (design change 0366).
pub(crate) const FIRST_AID_TENT: ObjectDef = ObjectDef {
    kind: ObjectKind::FirstAidTent,
    name: "First Aid Tent",
    faction: Some(Faction::Gdo), // Faction - GlobalDefenseOrganization
    category: Category::Structure,
    footprint: Footprint::Tiles { w: 2, h: 2 }, // Size 2x2
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    destructible: true,
    groupable: true,     // Groupable true
    sight_range: 4.0,    // SightRange 4 grid units
    max_hp: Some(200.0), // MaxHP 200
    armor: Armor {
        point: 1.0, // PointArmor 1
        full: 4.0,  // FullArmor 4
    },
    // "Can be placed on **any Buildable terrain**, like the Bunker": a building's, on flat ground (`core/map.md >
    // Elevation > Structures only on flat terrain`).
    needs_flat_ground: true,
    structure: Some(StructureDef {
        symmetry: SymmetryType::AAAA, // SymmetryType AAAA
        build_radius_extension: 0,    // BuildRadiusExtension - 0 (does not extend the GDO build area)
        power: 0,                     // Power 0
        requires: &[],
        ..StructureDef::NONE
    }),
    ..ObjectDef::NONE
};

/// `> Placeholder stats`: "Cost: Field Engineer, 75 Space Crystals, 240 frames (15 seconds)". Held until slice M6-GB
/// (`crate::defs::PLANNED_RECIPES`).
pub(crate) const RECIPE: Recipe = Recipe {
    kind: ObjectKind::FirstAidTent,
    cost: 75,
    frames: 240,
    requires: &[],
    supplies: 0,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// The placeholder stat block; it extends no build area and has no site yet; its Field Engineer entry, 75 Space
    /// Crystals and 240 frames, is held.
    #[test]
    fn the_first_aid_tent_follows_the_design() {
        let structure = super::super::assert_stat_block(
            ObjectKind::FirstAidTent,
            (2, 2),
            SymmetryType::AAAA,
            200.0,
            (1.0, 4.0),
            4.0,
            true,
            0,
            0,
        );
        assert!(structure.site.is_none() && structure.abilities.is_none());
        assert_eq!(RECIPE, super::super::held(ObjectKind::FirstAidTent, 75, 240));
    }
}
