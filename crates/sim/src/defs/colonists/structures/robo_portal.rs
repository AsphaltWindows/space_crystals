//! The Robo Portal — `factions/colonists/structures/robo_portal.md`.

use crate::defs::structures::{ExitRule, ProductionDef, Recipe, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;
use crate::sides::SideLetter;

/// `factions/colonists/structures/robo_portal.md`: the Drones' producer.
pub(crate) static ROBO_PORTAL: ObjectDef = ObjectDef {
    kind: ObjectKind::RoboPortal,
    name: "Robo Portal",
    faction: Some(Faction::Colonists),          // Faction - Colonists
    category: Category::Structure,              // Entity Type - Structure Type
    footprint: Footprint::Tiles { w: 4, h: 4 }, // Size - 4x4 grid units
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    destructible: true,                         // Destructible - true
    groupable: true,                            // Groupable - true
    sight_range: 4.0,                           // SightRange - 4 grid units
    max_hp: Some(600.0),                        // MaxHP - 600
    max_shields: Some(20.0),                    // MaxShields - 20
    armor: Armor {
        point: 1.0, // PointArmor - 1
        full: 16.0, // FullArmor - 16
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    unit: None,
    structure: Some(StructureDef {
        symmetry: SymmetryType::AAAA, // SymmetryType - AAAA
        build_radius_extension: 0,    // none (see `defs/colonists.rs`)
        power: 0,                     // none (plan decision T21)
        seeds_build_area: false,
        production: Some(ProductionDef {
            // The energy doorway at the centre (design change 0145, `factions/colonists/structures/README.md > Portals
            // > Portal Exits (energy doorway)`): "Since the Robo Portal is fully symmetric (AAAA), units may leave in
            // any direction": its four A sides are open, 360°.
            exits: ExitRule::Centre(&[SideLetter::A]),
            recipes: &[Recipe {
                kind: ObjectKind::Drone, // Produces: "Drone: 50 Space Crystals, 160 frames (10 seconds)"
                cost: 50,
                frames: 160,
                requires: &[],
            }],
            recruitment: None,
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
        // `factions/colonists/structures/README.md > Tech Model`: "Colonists buildings have no TechPrerequisites"
        requires: &[],
        attack: None,
        popup: None,
        holds: None,
    }),
    default_settings: None,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// It produces Drones, which leave its energy doorway in any direction, out through its four sides (all A, all
    /// open: design change 0145).
    #[test]
    fn the_robo_portal_follows_the_design() {
        let structure = crate::defs::colonists::tests::assert_structure(
            ObjectKind::RoboPortal,
            (4, 4),
            SymmetryType::AAAA,
            600.0,
            20.0,
            (1.0, 16.0),
            4.0,
            true,
        );
        assert!(structure.warp_in.is_empty());
        // Produces: "Drone: 50 Space Crystals, 160 frames (10 seconds)" (the Scout's cost and time are TBD).
        let production = structure.production.unwrap();
        assert_eq!(
            (production.exits, production.recipes),
            (
                ExitRule::Centre(&[SideLetter::A]),
                &[Recipe {
                    kind: ObjectKind::Drone,
                    cost: 50,
                    frames: 160,
                    requires: &[],
                }][..]
            )
        );
    }
}
