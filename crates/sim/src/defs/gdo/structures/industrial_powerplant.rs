//! The Industrial Powerplant — `factions/gdo/structures/industrial_powerplant.md` (design change 0366, slice M6-GK):
//! "GDO power generation structure. Provides power to the grid in larger quantities than the Generator." Its numbers
//! are the design's "Placeholder stats" (plan decision T84). (M6-GF) The Deployment Center constructs it ([`RECIPE`], in
//! its `constructs`), behind Advanced Mechanics, and its +60 Power counts in its owner's Power summary as a
//! Generator's +20 does (`crate::economy::power`).

use crate::defs::structures::{Recipe, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/gdo/structures/industrial_powerplant.md`: its placeholder stats (design change 0366).
pub(crate) const INDUSTRIAL_POWERPLANT: ObjectDef = ObjectDef {
    kind: ObjectKind::IndustrialPowerplant,
    name: "Industrial Powerplant",
    faction: Some(Faction::Gdo), // Faction - GlobalDefenseOrganization
    category: Category::Structure,
    footprint: Footprint::Tiles { w: 3, h: 3 }, // Size 3x3
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    destructible: true,
    groupable: true,     // Groupable true
    sight_range: 4.0,    // SightRange 4 grid units
    max_hp: Some(600.0), // MaxHP 600
    armor: Armor {
        point: 1.0, // PointArmor 1
        full: 9.0,  // FullArmor 9
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    structure: Some(StructureDef {
        symmetry: SymmetryType::AAAA, // SymmetryType AAAA
        build_radius_extension: 2,    // BuildRadiusExtension 2 grid units
        power: 60,                    // Power **+60** (three Generators' worth, in one building)
        // TechPrerequisite - player owns at least one Advanced Mechanics, wherever it is constructed (`crate::tech`).
        requires: &[ObjectKind::AdvancedMechanics],
        ..StructureDef::NONE
    }),
    ..ObjectDef::NONE
};

/// `factions/gdo/structures/deployment_center.md > Constructs`: "Industrial Powerplant: 250 Space Crystals, 480 frames
/// (30 seconds) (placeholder). Requires: player owns at least one Advanced Mechanics" (its own TechPrerequisite).
/// (M6-GF) In the Deployment Center's `constructs`.
pub(crate) const RECIPE: Recipe = Recipe {
    kind: ObjectKind::IndustrialPowerplant,
    cost: 250,
    frames: 480,
    requires: &[],
    supplies: 0,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// The placeholder stat block, a producer of Power (+60, three Generators'), and its TechPrerequisite: Advanced
    /// Mechanics. Its Deployment Center entry, 250 Space Crystals and 480 frames, is live (M6-GF).
    #[test]
    fn the_industrial_powerplant_follows_the_design() {
        let structure = super::super::assert_stat_block(
            ObjectKind::IndustrialPowerplant,
            (3, 3),
            SymmetryType::AAAA,
            600.0,
            (1.0, 9.0),
            4.0,
            true,
            2,
            60,
        );
        let generator = ObjectKind::Generator.def().structure.as_ref().unwrap();
        assert_eq!(structure.power, 3 * generator.power, "three Generators' worth");
        assert_eq!(structure.requires, [ObjectKind::AdvancedMechanics]);
        assert!(structure.production.is_none() && structure.constructs.is_empty());
        let recipe = Recipe {
            kind: ObjectKind::IndustrialPowerplant,
            cost: 250,
            frames: 480,
            requires: &[],
            supplies: 0,
        };
        assert_eq!(RECIPE, recipe);
        let dc = ObjectKind::DeploymentCenter.def().structure.as_ref().unwrap();
        assert!(dc.constructs.contains(&RECIPE), "the Deployment Center constructs it");
    }
}
