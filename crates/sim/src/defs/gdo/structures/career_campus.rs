//! The Career Campus — `factions/gdo/structures/career_campus.md` (design change 0328, slice M5-N9): the Field
//! Engineer's Unit Prerequisite at the Barracks (`factions/gdo/structures/barracks.md > Produces`). It has no panel
//! yet ("Panel: none yet (its ability research is not designed)").

use crate::defs::structures::{StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/gdo/structures/career_campus.md`: its placeholder stats (design change 0328).
pub(crate) const CAREER_CAMPUS: ObjectDef = ObjectDef {
    kind: ObjectKind::CareerCampus,
    name: "Career Campus",
    faction: Some(Faction::Gdo), // Faction - GlobalDefenseOrganization
    category: Category::Structure,
    footprint: Footprint::Tiles { w: 2, h: 2 }, // Size 2x2
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    destructible: true,
    groupable: true,     // Groupable true
    sight_range: 4.0,    // SightRange 4 grid units
    max_hp: Some(300.0), // MaxHP 300
    armor: Armor {
        point: 1.0, // PointArmor 1
        full: 4.0,  // FullArmor 4
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    structure: Some(StructureDef {
        symmetry: SymmetryType::AAAA, // SymmetryType AAAA
        build_radius_extension: 2,    // BuildRadiusExtension 2 grid units
        power: -20,                   // Power −20
        // TechPrerequisite - player owns at least one Barracks (`factions/gdo/structures/deployment_center.md >
        // Constructs`: "Requires: player owns at least one Barracks"), wherever it is constructed (`crate::tech`).
        requires: &[ObjectKind::Barracks],
        ..StructureDef::NONE
    }),
    ..ObjectDef::NONE
};

#[cfg(test)]
mod tests {
    use super::*;

    /// The placeholder stat block, and its TechPrerequisite: a Barracks.
    #[test]
    fn the_career_campus_follows_the_design() {
        let structure = super::super::assert_stat_block(
            ObjectKind::CareerCampus,
            (2, 2),
            SymmetryType::AAAA,
            300.0,
            (1.0, 4.0),
            4.0,
            true,
            2,
            -20,
        );
        assert_eq!(structure.requires, [ObjectKind::Barracks]);
        assert!(structure.production.is_none() && structure.constructs.is_empty());
    }
}
