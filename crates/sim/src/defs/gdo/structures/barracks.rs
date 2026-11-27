//! The Barracks — `factions/gdo/structures/barracks.md`.

use crate::defs::structures::{ExitRule, ProductionDef, Recipe, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;
use crate::sides::SideLetter;

/// `factions/gdo/structures/barracks.md`.
pub(crate) static BARRACKS: ObjectDef = ObjectDef {
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
    max_shields: None,
    armor: Armor {
        point: 1.0, // PointArmor - 1
        full: 6.0,  // FullArmor - 6
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    unit: None,
    structure: Some(StructureDef {
        symmetry: SymmetryType::ABAC, // SymmetryType - ABAC (top A, right B, bottom A, left C)
        build_radius_extension: 2,    // BuildRadiusExtension - 2 grid units
        power: -30,                   // Power - -30
        seeds_build_area: false,
        production: Some(ProductionDef {
            exits: ExitRule::Side(SideLetter::B), // "Units exit from one of the short sides (B side ...)"
            // Produces: the Peacekeeper, the Rocketman (slice M5-K) and the Field Engineer (slice M5-N9), in the
            // design's order. The Drone Operator has no unit yet; it joins with the slice that makes it (plan decision
            // T69).
            recipes: &[
                Recipe {
                    kind: ObjectKind::Peacekeeper, // "Peacekeeper: 50 Space Crystals, 80 frames (5 seconds)"
                    cost: 50,
                    frames: 80,
                    requires: &[],
                },
                Recipe {
                    // "Rocketman: 100 Space Crystals, 160 frames (10 seconds) (placeholder). Requires: player owns at
                    // least one Arsenal." (design change 0356)
                    kind: ObjectKind::Rocketman,
                    cost: 100,
                    frames: 160,
                    requires: &[ObjectKind::Arsenal],
                },
                Recipe {
                    // "Field Engineer: 75 Space Crystals, 160 frames (10 seconds) (placeholder). Requires: player owns
                    // at least one Career Campus." (design change 0328)
                    kind: ObjectKind::FieldEngineer,
                    cost: 75,
                    frames: 160,
                    requires: &[ObjectKind::CareerCampus],
                },
            ],
            recruitment: None,
            augment: &[],
            default_rally: None,
        }),
        mining: None,
        onto: None,
        constructs: &[],
        warp_in: &[],
        drop_off: None,
        underground: false,
        tunnel: None,
        site: None,
        expansion: None,
        // No TechPrerequisite (`factions/gdo/structures/deployment_center.md > Constructs`: no "Requires")
        requires: &[],
        attack: None,
        popup: None,
        holds: None,
        occupants_fire: None,
        research: None,
        abilities: None,
    }),
    default_settings: None,
    energy: None,
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
