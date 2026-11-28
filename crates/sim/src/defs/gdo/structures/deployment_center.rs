//! The Deployment Center — `factions/gdo/structures/deployment_center.md`.

use super::{advanced_mechanics, airfield, defense_system, factory, industrial_powerplant, radar_station};
use crate::defs::structures::{Recipe, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/gdo/structures/deployment_center.md`.
pub(crate) const DEPLOYMENT_CENTER: ObjectDef = ObjectDef {
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
    armor: Armor {
        point: 1.0, // PointArmor - 1
        full: 16.0, // FullArmor - 16
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    structure: Some(StructureDef {
        symmetry: SymmetryType::AAAA, // SymmetryType - AAAA
        build_radius_extension: 12,   // BuildRadiusExtension - 12 grid units
        power: 30,                    // Power - +30
        // `factions/gdo/structures/README.md > GDOBuildArea`: "Seeded by each Deployment Center's and each Outpost's
        // BuildRadiusExtension".
        seeds_build_area: true,
        // Constructs: the buildings in the game, each with its cost and time, in the design's order: the three of the
        // start, the Arsenal (slice M5-K, design change 0356) and the Career Campus (slice M5-N9, design change 0328),
        // whose "Requires: player owns at least one Barracks" is each type's own TechPrerequisite
        // (`StructureDef::requires`), and (M6-GR) the Radar Station and the Defense System, each with its own (a
        // Generator; a Radar Station and an Arsenal), and (M6-GF) the Factory, Advanced Mechanics and the Industrial
        // Powerplant, each its def module's entry, behind its own (a Barracks, a Factory, Advanced Mechanics), and
        // (M6-GH2) the Airfield, behind its own (Advanced Mechanics), all in the design's order. The design's others
        // (Supply Tower, Propulsion Labs) come with their slices, their Requires clauses (`core/tech_and_research.md >
        // Tech Prerequisites`) with them, held until then (`crate::defs::PLANNED_RECIPES`).
        constructs: &[
            Recipe {
                kind: ObjectKind::Generator,
                cost: 100,   // Generator: 100 Space Crystals,
                frames: 240, // 240 frames (15 seconds)
                requires: &[],
                supplies: 0,
            },
            Recipe {
                kind: ObjectKind::Barracks,
                cost: 200,   // Barracks: 200 Space Crystals,
                frames: 160, // 160 frames (10 seconds)
                requires: &[],
                supplies: 0,
            },
            Recipe {
                kind: ObjectKind::ExtractionFacility,
                cost: 200,   // Extraction Facility: 200 Space Crystals,
                frames: 320, // 320 frames (20 seconds)
                requires: &[],
                supplies: 0,
            },
            Recipe {
                kind: ObjectKind::Arsenal,
                cost: 150,     // Arsenal: 150 Space Crystals,
                frames: 320,   // 320 frames (20 seconds) (placeholder).
                requires: &[], // Requires: player owns at least one Barracks (the Arsenal's own)
                supplies: 0,
            },
            Recipe {
                kind: ObjectKind::CareerCampus,
                cost: 150,     // Career Campus: 150 Space Crystals,
                frames: 320,   // 320 frames (20 seconds) (placeholder).
                requires: &[], // Requires: player owns at least one Barracks (the Career Campus's own)
                supplies: 0,
            },
            // (M6-GF) "Factory: 250 Space Crystals, 480 frames (30 seconds) (placeholder). Requires: player owns at
            // least one Barracks."
            factory::RECIPE,
            // (M6-GR) Radar Station: 150 Space Crystals, 320 frames (20 seconds) (placeholder).
            radar_station::RECIPE,
            // (M6-GF) "Advanced Mechanics: 200 Space Crystals, 400 frames (25 seconds) (placeholder). Requires: player
            // owns at least one Factory."
            advanced_mechanics::RECIPE,
            // (M6-GH2) "Airfield: 250 Space Crystals, 480 frames (30 seconds) (placeholder). Requires: player owns at
            // least one Advanced Mechanics."
            airfield::RECIPE,
            // (M6-GF) "Industrial Powerplant: 250 Space Crystals, 480 frames (30 seconds) (placeholder). Requires:
            // player owns at least one Advanced Mechanics."
            industrial_powerplant::RECIPE,
            // (M6-GR) Defense System: 150 Space Crystals, 320 frames (20 seconds) (placeholder).
            defense_system::RECIPE,
        ],
        requires: &[], // No TechPrerequisite: none declared
        ..StructureDef::NONE
    }),
    ..ObjectDef::NONE
};

#[cfg(test)]
mod tests {
    use super::*;

    /// The stat block, and `> Constructs`: the eleven buildings in the game with each one's cost and time, in the
    /// design's order, the Arsenal and the Career Campus requiring a Barracks (each its own TechPrerequisite, which
    /// `tech::recipe_requires` adds to its entry's), (M6-GR) the Radar Station a Generator, the Defense System a Radar
    /// Station and an Arsenal, (M6-GF) the Factory a Barracks, Advanced Mechanics a Factory and the Industrial
    /// Powerplant Advanced Mechanics, and (M6-GH2) the Airfield Advanced Mechanics. It, the Extraction Facility
    /// (`factions/gdo/economy.md > ExtractionFacility > Constructs`) and (M6-GR) the Defense System are the three
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
            supplies: 0,
        };
        assert_eq!(
            structure.constructs,
            [
                recipe(ObjectKind::Generator, 100, 240),
                recipe(ObjectKind::Barracks, 200, 160),
                recipe(ObjectKind::ExtractionFacility, 200, 320),
                recipe(ObjectKind::Arsenal, 150, 320),
                recipe(ObjectKind::CareerCampus, 150, 320),
                recipe(ObjectKind::Factory, 250, 480),
                recipe(ObjectKind::RadarStation, 150, 320),
                recipe(ObjectKind::AdvancedMechanics, 200, 400),
                recipe(ObjectKind::Airfield, 250, 480),
                recipe(ObjectKind::IndustrialPowerplant, 250, 480),
                recipe(ObjectKind::DefenseSystem, 150, 320),
            ]
        );
        let requires = |i: usize| crate::tech::recipe_requires(&structure.constructs[i]).collect::<Vec<_>>();
        for needs_barracks in 3..6 {
            assert_eq!(requires(needs_barracks), [ObjectKind::Barracks]);
        }
        assert_eq!(requires(6), [ObjectKind::Generator]);
        assert_eq!(requires(7), [ObjectKind::Factory]);
        assert_eq!(requires(8), [ObjectKind::AdvancedMechanics]);
        assert_eq!(requires(9), [ObjectKind::AdvancedMechanics]);
        assert_eq!(requires(10), [ObjectKind::RadarStation, ObjectKind::Arsenal]);
        for &kind in ObjectKind::ALL {
            let constructs = kind.def().structure.as_ref().is_some_and(|s| !s.constructs.is_empty());
            let constructor = matches!(
                kind,
                ObjectKind::DeploymentCenter | ObjectKind::ExtractionFacility | ObjectKind::DefenseSystem
            );
            assert_eq!(constructs, constructor, "{kind:?}");
        }
    }
}
