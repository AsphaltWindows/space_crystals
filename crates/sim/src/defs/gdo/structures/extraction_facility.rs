//! The Extraction Facility — `factions/gdo/economy.md > ExtractionFacility`.

use crate::defs::structures::{Recipe, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/gdo/economy.md > ExtractionFacility`.
pub(crate) const EXTRACTION_FACILITY: ObjectDef = ObjectDef {
    kind: ObjectKind::ExtractionFacility,
    name: "Extraction Facility",
    faction: Some(Faction::Gdo), // Faction - GlobalDefenseOrganization
    category: Category::Structure,
    footprint: Footprint::Tiles { w: 3, h: 3 }, // Size - 3x3 grid units
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    destructible: true,                         // Destructible - true
    groupable: false,                           // Groupable - false
    sight_range: 3.0,                           // SightRange - 3 grid units
    max_hp: Some(500.0),                        // MaxHP - 500
    armor: Armor {
        point: 1.0, // PointArmor - 1
        full: 9.0,  // FullArmor - 9
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    structure: Some(StructureDef {
        symmetry: SymmetryType::AAAA, // SymmetryType - AAAA
        build_radius_extension: 2,    // BuildRadiusExtension - 2 grid units
        power: -15,                   // Power - -15
        // Constructs
        constructs: &[Recipe {
            kind: ObjectKind::ExtractionPlate,
            cost: 75,    // Extraction Plate: 75 Space Crystals,
            frames: 400, // 400 frames (25 seconds)
            requires: &[],
        }],
        // No TechPrerequisite (`factions/gdo/structures/deployment_center.md > Constructs`: no "Requires")
        requires: &[],
        ..StructureDef::NONE
    }),
    ..ObjectDef::NONE
};

#[cfg(test)]
mod tests {
    use super::*;

    /// The stat block, and `> Constructs`: the plate, with its cost and time.
    #[test]
    fn the_extraction_facility_follows_the_design() {
        let structure = super::super::assert_stat_block(
            ObjectKind::ExtractionFacility,
            (3, 3),
            SymmetryType::AAAA,
            500.0,
            (1.0, 9.0),
            3.0,
            false,
            2,
            -15,
        );
        let plate = Recipe {
            kind: ObjectKind::ExtractionPlate,
            cost: 75,
            frames: 400,
            requires: &[],
        };
        assert_eq!(structure.constructs, [plate]);
    }
}
