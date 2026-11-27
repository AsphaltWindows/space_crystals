//! The Factory — `factions/gdo/structures/factory.md` (design change 0366, slice M6-GK): "GDO vehicle production
//! structure. Allows the building of vehicles." Its numbers are the design's "Placeholder stats" (plan decision T84).
//! No player constructs one, and it produces nothing, until slice M6-GF: its production ("Production as the Barracks
//! (queue 20, refunds, rally)"), with the default RallyPoint of a building that releases vehicles through an exit side
//! (`core/entities.md > Entrance and exit > Exit Doorway` point 7, design changes 0446 and 0447: 3 grid units out,
//! `ProductionDef::default_rally`), and its Deployment Center entry ([`RECIPE`], held) are that slice's.

use crate::defs::structures::{Recipe, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/gdo/structures/factory.md`: its placeholder stats (design change 0366).
pub(crate) const FACTORY: ObjectDef = ObjectDef {
    kind: ObjectKind::Factory,
    name: "Factory",
    faction: Some(Faction::Gdo), // Faction - GlobalDefenseOrganization
    category: Category::Structure,
    footprint: Footprint::Tiles { w: 3, h: 3 }, // Size 3x3
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    destructible: true,
    groupable: true,     // Groupable true
    sight_range: 4.0,    // SightRange 4 grid units
    max_hp: Some(500.0), // MaxHP 500
    armor: Armor {
        point: 1.0, // PointArmor 1
        full: 9.0,  // FullArmor 9
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    structure: Some(StructureDef {
        symmetry: SymmetryType::ABAC, // SymmetryType ABAC (exit side B, like the Barracks)
        build_radius_extension: 2,    // BuildRadiusExtension 2 grid units
        power: -30,                   // Power -30
        // TechPrerequisite - player owns at least one Barracks, wherever it is constructed (`crate::tech`).
        requires: &[ObjectKind::Barracks],
        ..StructureDef::NONE
    }),
    ..ObjectDef::NONE
};

/// `factions/gdo/structures/deployment_center.md > Constructs`: "Factory: 250 Space Crystals, 480 frames (30 seconds)
/// (placeholder). Requires: player owns at least one Barracks" (its own TechPrerequisite). Held until slice M6-GF
/// (`crate::defs::PLANNED_RECIPES`).
pub(crate) const RECIPE: Recipe = Recipe {
    kind: ObjectKind::Factory,
    cost: 250,
    frames: 480,
    requires: &[],
    supplies: 0,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// The placeholder stat block, ABAC, and its TechPrerequisite: a Barracks. It produces and constructs nothing yet;
    /// its Deployment Center entry, 250 Space Crystals and 480 frames, is held.
    #[test]
    fn the_factory_follows_the_design() {
        let structure = super::super::assert_stat_block(
            ObjectKind::Factory,
            (3, 3),
            SymmetryType::ABAC,
            500.0,
            (1.0, 9.0),
            4.0,
            true,
            2,
            -30,
        );
        assert_eq!(structure.requires, [ObjectKind::Barracks]);
        assert!(structure.production.is_none() && structure.constructs.is_empty());
        assert_eq!(RECIPE, super::super::held(ObjectKind::Factory, 250, 480));
    }
}
