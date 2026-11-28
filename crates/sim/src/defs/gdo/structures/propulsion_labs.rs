//! Propulsion Labs — `factions/gdo/structures/propulsion_labs.md` (design change 0366, slice M6-GK): "GDO advanced
//! aircraft structure [...] Unlocks the **Dragonfly** at the Airfield." Its numbers are the design's "Placeholder
//! stats" (plan decision T84); it has no panel ("Panel: none yet"). (M6-GL) The Deployment Center constructs it
//! ([`RECIPE`]), behind an Airfield, and it is the Dragonfly's Unit Prerequisite at the Airfield
//! (`super::super::units::dragonfly::RECIPE`).

use crate::defs::structures::{Recipe, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/gdo/structures/propulsion_labs.md`: its placeholder stats (design change 0366).
pub(crate) const PROPULSION_LABS: ObjectDef = ObjectDef {
    kind: ObjectKind::PropulsionLabs,
    name: "Propulsion Labs",
    faction: Some(Faction::Gdo), // Faction - GlobalDefenseOrganization
    category: Category::Structure,
    footprint: Footprint::Tiles { w: 2, h: 2 }, // Size 2x2
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    destructible: true,
    groupable: true,     // Groupable true
    sight_range: 4.0,    // SightRange 4 grid units
    max_hp: Some(350.0), // MaxHP 350
    armor: Armor {
        point: 1.0, // PointArmor 1
        full: 4.0,  // FullArmor 4
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    structure: Some(StructureDef {
        symmetry: SymmetryType::AAAA, // SymmetryType AAAA
        build_radius_extension: 2,    // BuildRadiusExtension 2 grid units
        power: -20,                   // Power -20
        // TechPrerequisite - player owns at least one Airfield, wherever it is constructed (`crate::tech`).
        requires: &[ObjectKind::Airfield],
        ..StructureDef::NONE
    }),
    ..ObjectDef::NONE
};

/// `factions/gdo/structures/deployment_center.md > Constructs`: "Propulsion Labs: 200 Space Crystals, 400 frames (25
/// seconds) (placeholder). Requires: player owns at least one Airfield" (its own TechPrerequisite). The Deployment
/// Center's since slice M6-GL (`super::deployment_center`).
pub(crate) const RECIPE: Recipe = Recipe {
    kind: ObjectKind::PropulsionLabs,
    cost: 200,
    frames: 400,
    requires: &[],
    supplies: 0,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// The placeholder stat block, and its TechPrerequisite: an Airfield. It produces and constructs nothing; its
    /// Deployment Center entry costs 200 Space Crystals and takes 400 frames, with no Requires or Supplies of its own.
    #[test]
    fn propulsion_labs_follows_the_design() {
        let structure = super::super::assert_stat_block(
            ObjectKind::PropulsionLabs,
            (2, 2),
            SymmetryType::AAAA,
            350.0,
            (1.0, 4.0),
            4.0,
            true,
            2,
            -20,
        );
        assert_eq!(structure.requires, [ObjectKind::Airfield]);
        assert!(structure.production.is_none() && structure.constructs.is_empty());
        assert_eq!(RECIPE, super::super::held(ObjectKind::PropulsionLabs, 200, 400));
    }
}
