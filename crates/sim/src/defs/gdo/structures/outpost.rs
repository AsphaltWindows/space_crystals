//! The Outpost — `factions/gdo/structures/outpost.md` (design change 0366, slice M6-GK): "GDO expansion structure
//! created when a Prospector deploys. Its only function is to provide a **build radius**". Its numbers are the
//! design's "Placeholder stats" (plan decision T84). Nothing constructs it ("Created by - Prospector (Deploy into
//! Outpost); not built by the Deployment Center"), and no Prospector deploys into one until slice M6-GP.

use crate::defs::structures::{StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/gdo/structures/outpost.md`: its placeholder stats (design change 0366).
pub(crate) const OUTPOST: ObjectDef = ObjectDef {
    kind: ObjectKind::Outpost,
    name: "Outpost",
    faction: Some(Faction::Gdo), // Faction - GlobalDefenseOrganization
    category: Category::Structure,
    footprint: Footprint::Tiles { w: 3, h: 3 }, // Size 3x3 (the Prospector deploys in place)
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    destructible: true,
    // **Placeholder (not in the design: the Outpost's Groupable)**: true, as every GDO building that constructs
    // nothing (only the constructors, the Deployment Center and the Extraction Facility, and the Radar Station aren't).
    groupable: true,
    sight_range: 6.0,    // SightRange 6 grid units
    max_hp: Some(600.0), // MaxHP 600
    armor: Armor {
        point: 1.0, // PointArmor 1
        full: 9.0,  // FullArmor 9
    },
    // "Placement - any Buildable terrain (where the Prospector deploys)": a building's, on flat ground
    // (`core/map.md > Elevation > Structures only on flat terrain`).
    needs_flat_ground: true,
    structure: Some(StructureDef {
        symmetry: SymmetryType::AAAA, // SymmetryType AAAA
        build_radius_extension: 6,    // **BuildRadiusExtension 6 grid units**
        power: -10,                   // Power - −10 (placeholder)
        // `factions/gdo/structures/README.md > GDOBuildArea`: "Seeded by each Deployment Center's and each Outpost's
        // BuildRadiusExtension".
        seeds_build_area: true,
        // "Provides **only** the build radius — it does not construct anything, produce units, or act as a drop-off."
        requires: &[],
        ..StructureDef::NONE
    }),
    ..ObjectDef::NONE
};

#[cfg(test)]
mod tests {
    use super::*;

    /// The placeholder stat block; it seeds the GDO build area, as a Deployment Center does, and does nothing else: it
    /// constructs and produces nothing, takes no drop-off, and no constructor's list has it.
    #[test]
    fn the_outpost_follows_the_design() {
        let structure = super::super::assert_stat_block(
            ObjectKind::Outpost,
            (3, 3),
            SymmetryType::AAAA,
            600.0,
            (1.0, 9.0),
            6.0,
            true,
            6,
            -10,
        );
        assert!(structure.seeds_build_area);
        assert!(structure.constructs.is_empty() && structure.production.is_none() && structure.drop_off.is_none());
        assert!(
            !crate::defs::PLANNED_RECIPES
                .iter()
                .any(|(_, recipe)| recipe.kind == ObjectKind::Outpost)
        );
    }
}
