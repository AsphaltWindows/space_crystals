//! The Deployment Center — `factions/gdo/structures/deployment_center.md`.

use crate::defs::structures::{Recipe, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/gdo/structures/deployment_center.md`.
pub(crate) static DEPLOYMENT_CENTER: ObjectDef = ObjectDef {
    kind: ObjectKind::DeploymentCenter,
    name: "Deployment Center",
    faction: Some(Faction::Gdo), // Faction - GlobalDefenseOrganization
    category: Category::Structure,
    footprint: Footprint::Tiles { w: 4, h: 4 }, // Size - 4x4 grid units
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    destructible: true,                         // Destructible - true
    groupable: false,                           // Groupable - false
    sight_range: 6.0,                           // SightRange - 6 grid units
    max_hp: Some(1000.0),                       // MaxHP - 1000
    max_shields: None,
    armor: Armor {
        point: 1.0, // PointArmor - 1
        full: 16.0, // FullArmor - 16
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    unit: None,
    structure: Some(StructureDef {
        symmetry: SymmetryType::AAAA, // SymmetryType - AAAA
        build_radius_extension: 12,   // BuildRadiusExtension - 12 grid units
        power: 30,                    // Power - +30
        // `factions/gdo/structures/README.md > GDOBuildArea`: "Seeded by each Deployment Center's and each Outpost's
        // BuildRadiusExtension".
        seeds_build_area: true,
        production: None,
        mining: None,
        onto: None,
        // Constructs: the buildings in the game, each with its cost and time: the three of the start, and the Career
        // Campus (slice M5-N9, design change 0328), whose "Requires: player owns at least one Barracks" is its type's
        // TechPrerequisite (`StructureDef::requires`). The design's others (Supply Tower, Arsenal, …) come with their
        // slices, their Requires clauses (`core/tech_and_research.md > Tech Prerequisites`) with them.
        constructs: &[
            Recipe {
                kind: ObjectKind::Generator,
                cost: 100,   // Generator: 100 Space Crystals,
                frames: 240, // 240 frames (15 seconds)
                requires: &[],
            },
            Recipe {
                kind: ObjectKind::Barracks,
                cost: 200,   // Barracks: 200 Space Crystals,
                frames: 160, // 160 frames (10 seconds)
                requires: &[],
            },
            Recipe {
                kind: ObjectKind::ExtractionFacility,
                cost: 200,   // Extraction Facility: 200 Space Crystals,
                frames: 320, // 320 frames (20 seconds)
                requires: &[],
            },
            Recipe {
                kind: ObjectKind::CareerCampus,
                cost: 150,     // Career Campus: 150 Space Crystals,
                frames: 320,   // 320 frames (20 seconds) (placeholder).
                requires: &[], // Requires: player owns at least one Barracks (the Career Campus's own)
            },
        ],
        warp_in: &[],
        drop_off: None,
        underground: false,
        tunnel: None,
        site: None,
        expansion: None,
        requires: &[], // No TechPrerequisite: none declared
        attack: None,
        popup: None,
        holds: None,
        occupants_fire: None,
    }),
    default_settings: None,
    energy: None,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// The stat block, and `> Constructs`: the four buildings in the game with each one's cost and time, the Career
    /// Campus requiring a Barracks (its own TechPrerequisite, which `tech::recipe_requires` adds to its entry's). It
    /// and the Extraction Facility (`factions/gdo/economy.md > ExtractionFacility > Constructs`) are the two
    /// constructors.
    #[test]
    fn the_deployment_center_follows_the_design() {
        let structure = super::super::assert_stat_block(
            ObjectKind::DeploymentCenter,
            (4, 4),
            SymmetryType::AAAA,
            1000.0,
            (1.0, 16.0),
            6.0,
            false,
            12,
            30,
        );
        let recipe = |kind, cost, frames| Recipe {
            kind,
            cost,
            frames,
            requires: &[],
        };
        assert_eq!(
            structure.constructs,
            [
                recipe(ObjectKind::Generator, 100, 240),
                recipe(ObjectKind::Barracks, 200, 160),
                recipe(ObjectKind::ExtractionFacility, 200, 320),
                recipe(ObjectKind::CareerCampus, 150, 320),
            ]
        );
        let campus = &structure.constructs[3];
        assert_eq!(
            crate::tech::recipe_requires(campus).collect::<Vec<_>>(),
            [ObjectKind::Barracks]
        );
        for &kind in ObjectKind::ALL {
            let constructs = kind.def().structure.as_ref().is_some_and(|s| !s.constructs.is_empty());
            let constructor = matches!(kind, ObjectKind::DeploymentCenter | ObjectKind::ExtractionFacility);
            assert_eq!(constructs, constructor, "{kind:?}");
        }
    }
}
