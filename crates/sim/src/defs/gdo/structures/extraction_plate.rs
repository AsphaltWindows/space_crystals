//! The Extraction Plate — `factions/gdo/economy.md > ExtractionPlate`.

use crate::defs::structures::{MiningDef, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/gdo/economy.md > ExtractionPlate`: placed onto a Space Crystals Patch (`StructureDef::onto`), which it
/// shares its tile with (`ObjectKind::may_overlap`).
pub(crate) static EXTRACTION_PLATE: ObjectDef = ObjectDef {
    kind: ObjectKind::ExtractionPlate,
    name: "Extraction Plate",
    faction: Some(Faction::Gdo), // Faction - GlobalDefenseOrganization
    category: Category::Structure,
    footprint: Footprint::Tiles { w: 1, h: 1 }, // Size - 1x1 grid unit
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    destructible: true,                         // Destructible - true
    groupable: true,                            // Groupable - true
    sight_range: 0.0,                           // SightRange - 0
    max_hp: Some(85.0),                         // MaxHP - 85
    max_shields: None,
    armor: Armor {
        point: 2.0, // PointArmor - 2
        full: 2.0,  // FullArmor - 2
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    unit: None,
    structure: Some(StructureDef {
        symmetry: SymmetryType::AAAA, // SymmetryType - AAAA
        build_radius_extension: 0,    // BuildRadiusExtension - 0
        power: -3,                    // Power - -3
        seeds_build_area: false,
        production: None,
        mining: Some(MiningDef {
            load: 10,        // MiningRate - 10 Space Crystals per 48 frames (3 seconds)
            residual: 1,     // ResidualMiningRate - 1 Space Crystal per 48 frames (3 seconds)
            cycle_ticks: 48, // "per 48 frames"
        }),
        // "Placed onto a Space Crystal Patch by an Extraction Facility", exactly over it (`placement::onto`).
        onto: Some(ObjectKind::CrystalPatch),
        constructs: &[],
        warp_in: &[],
        drop_off: None,
        underground: false,
        tunnel: None,
        site: None,
        expansion: None,
        requires: &[], // No TechPrerequisite: none declared
        attack: None,
        popup: None,
    }),
    default_settings: None,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// The stat block, and `> MiningRate - 10 Space Crystals per 48 frames`, `> ResidualMiningRate - 1 Space Crystal
    /// per 48 frames`: the plate is the one structure that mines.
    #[test]
    fn the_extraction_plate_follows_the_design() {
        let structure = super::super::assert_stat_block(
            ObjectKind::ExtractionPlate,
            (1, 1),
            SymmetryType::AAAA,
            85.0,
            (2.0, 2.0),
            0.0,
            true,
            0,
            -3,
        );
        let mining = MiningDef {
            load: 10,
            residual: 1,
            cycle_ticks: 48,
        };
        assert_eq!(structure.mining, Some(mining));
        for &kind in ObjectKind::ALL {
            let mines = kind.def().structure.as_ref().is_some_and(|s| s.mining.is_some());
            assert_eq!(mines, kind == ObjectKind::ExtractionPlate, "{kind:?}");
        }
    }
}
