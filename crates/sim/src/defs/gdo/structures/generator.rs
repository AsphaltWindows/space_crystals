//! The Generator — `factions/gdo/economy.md > Generator`.

use crate::defs::structures::{StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/gdo/economy.md > Generator`.
pub(crate) static GENERATOR: ObjectDef = ObjectDef {
    kind: ObjectKind::Generator,
    name: "Generator",
    faction: Some(Faction::Gdo), // Faction - GlobalDefenseOrganization
    category: Category::Structure,
    footprint: Footprint::Tiles { w: 2, h: 2 }, // Size - 2x2 grid units
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    destructible: true,                         // Destructible - true
    groupable: true,                            // Groupable - true
    sight_range: 3.0,                           // SightRange - 3 grid units
    max_hp: Some(350.0),                        // MaxHP - 350
    max_shields: None,
    armor: Armor {
        point: 1.0, // PointArmor - 1
        full: 4.0,  // FullArmor - 4
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    unit: None,
    structure: Some(StructureDef {
        symmetry: SymmetryType::AAAA, // SymmetryType - AAAA
        build_radius_extension: 1,    // BuildRadiusExtension - 1 grid unit
        power: 20,                    // Power - +20
        seeds_build_area: false,
        production: None,
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
    }),
    default_settings: None,
    energy: None,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_generator_follows_the_design() {
        super::super::assert_stat_block(
            ObjectKind::Generator,
            (2, 2),
            SymmetryType::AAAA,
            350.0,
            (1.0, 4.0),
            3.0,
            true,
            1,
            20,
        );
    }
}
