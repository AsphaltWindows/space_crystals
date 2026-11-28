//! The Barracks — `factions/gdo/structures/barracks.md`.

use crate::defs::gdo::units::drone_operator;
use crate::defs::structures::{ExitRule, ProductionDef, Recipe, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;
use crate::sides::SideLetter;

/// `factions/gdo/structures/barracks.md`.
pub(crate) const BARRACKS: ObjectDef = ObjectDef {
    kind: ObjectKind::Barracks,
    name: "Barracks",
    faction: Some(Faction::Gdo), // Faction - GlobalDefenseOrganization
    category: Category::Structure,
    // Size - 2x3 grid units (2 tall, 3 wide in the default orientation; design change 0079).
    footprint: Footprint::Tiles { w: 3, h: 2 },
    selectable: true,    // `core/entities.md > Object Type > Selectable - true`
    destructible: true,  // Destructible - true
    groupable: true,     // Groupable - true
    sight_range: 4.0,    // SightRange - 4 grid units
    max_hp: Some(300.0), // MaxHP - 300
    armor: Armor {
        point: 1.0, // PointArmor - 1
        full: 6.0,  // FullArmor - 6
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    structure: Some(StructureDef {
        symmetry: SymmetryType::ABAC, // SymmetryType - ABAC (top A, right B, bottom A, left C)
        build_radius_extension: 2,    // BuildRadiusExtension - 2 grid units
        power: -30,                   // Power - -30
        production: Some(ProductionDef {
            exits: ExitRule::Side(SideLetter::B), // "Units exit from one of the short sides (B side ...)"
            // Produces: the Peacekeeper, the Rocketman (slice M5-K), the Drone Operator (slice M6-GD, its def module's
            // recipe, with its Supply) and the Field Engineer (slice M5-N9), in the design's order.
            recipes: &[
                Recipe {
                    kind: ObjectKind::Peacekeeper, // "Peacekeeper: 50 Space Crystals, 80 frames (5 seconds)"
                    cost: 50,
                    frames: 80,
                    requires: &[],
                    supplies: 0,
                },
                Recipe {
                    // "Rocketman: 100 Space Crystals, 160 frames (10 seconds) (placeholder). Requires: player owns at
                    // least one Arsenal." (design change 0356)
                    kind: ObjectKind::Rocketman,
                    cost: 100,
                    frames: 160,
                    requires: &[ObjectKind::Arsenal],
                    supplies: 0,
                },
                // "Drone Operator: 100 Space Crystals, 160 frames (10 seconds) (placeholder). Requires: player owns at
                // least one Career Campus." and "**R: Build Drone Operator** [...] deducts 100 Space Crystals and 1
                // Supply" (design changes 0421 and 0441).
                drone_operator::RECIPE,
                Recipe {
                    // "Field Engineer: 75 Space Crystals, 160 frames (10 seconds) (placeholder). Requires: player owns
                    // at least one Career Campus." (design change 0328), and its Supply
                    // (`factions/gdo/units/field_engineer.md > Placeholder stats`).
                    kind: ObjectKind::FieldEngineer,
                    cost: 75,
                    frames: 160,
                    requires: &[ObjectKind::CareerCampus],
                    supplies: 1,
                },
            ],
            recruitment: None,
            augment: &[],
            default_rally: None,
        }),
        // No TechPrerequisite (`factions/gdo/structures/deployment_center.md > Constructs`: no "Requires")
        requires: &[],
        ..StructureDef::NONE
    }),
    ..ObjectDef::NONE
};

#[cfg(test)]
mod tests {
    use super::*;

    /// `> Size - 2x3 grid units (2 tall, 3 wide in the default orientation)` (design change 0079).
    #[test]
    fn the_barracks_follows_the_design() {
        super::super::assert_stat_block(
            ObjectKind::Barracks,
            (3, 2),
            SymmetryType::ABAC,
            300.0,
            (1.0, 6.0),
            4.0,
            true,
            2,
            -30,
        );
    }
}
