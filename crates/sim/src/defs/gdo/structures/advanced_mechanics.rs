//! Advanced Mechanics — `factions/gdo/structures/advanced_mechanics.md` (design change 0366, slice M6-GK): "GDO
//! vehicle upgrade structure [...] Unlocks the **Prospector** (expansion vehicle) and the **Artillery Rig** at the
//! Factory." Its numbers are the design's "Placeholder stats" (plan decision T84); it has no panel ("Panel: none yet
//! (its upgrades are not designed)"). (M6-GF) The Deployment Center constructs it ([`RECIPE`], in its `constructs`),
//! behind a Factory; it is the TechPrerequisite of the Industrial Powerplant, and of the Factory's Prospector and
//! Artillery Rig, whose entries are held until slices M6-GP and M6-GA (`crate::defs::PLANNED_RECIPES`).

use crate::defs::structures::{Recipe, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/gdo/structures/advanced_mechanics.md`: its placeholder stats (design change 0366).
pub(crate) const ADVANCED_MECHANICS: ObjectDef = ObjectDef {
    kind: ObjectKind::AdvancedMechanics,
    name: "Advanced Mechanics",
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
        // TechPrerequisite - player owns at least one Factory, wherever it is constructed (`crate::tech`).
        requires: &[ObjectKind::Factory],
        ..StructureDef::NONE
    }),
    ..ObjectDef::NONE
};

/// `factions/gdo/structures/deployment_center.md > Constructs`: "Advanced Mechanics: 200 Space Crystals, 400 frames
/// (25 seconds) (placeholder). Requires: player owns at least one Factory" (its own TechPrerequisite). (M6-GF) In the
/// Deployment Center's `constructs`.
pub(crate) const RECIPE: Recipe = Recipe {
    kind: ObjectKind::AdvancedMechanics,
    cost: 200,
    frames: 400,
    requires: &[],
    supplies: 0,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// The placeholder stat block, and its TechPrerequisite: a Factory. It produces, constructs and researches nothing;
    /// its Deployment Center entry, 200 Space Crystals and 400 frames, is live (M6-GF).
    #[test]
    fn advanced_mechanics_follows_the_design() {
        let structure = super::super::assert_stat_block(
            ObjectKind::AdvancedMechanics,
            (2, 2),
            SymmetryType::AAAA,
            350.0,
            (1.0, 4.0),
            4.0,
            true,
            2,
            -20,
        );
        assert_eq!(structure.requires, [ObjectKind::Factory]);
        assert!(structure.production.is_none() && structure.research.is_none() && structure.constructs.is_empty());
        let recipe = Recipe {
            kind: ObjectKind::AdvancedMechanics,
            cost: 200,
            frames: 400,
            requires: &[],
            supplies: 0,
        };
        assert_eq!(RECIPE, recipe);
        let dc = ObjectKind::DeploymentCenter.def().structure.as_ref().unwrap();
        assert!(dc.constructs.contains(&RECIPE), "the Deployment Center constructs it");
    }
}
