//! The Arsenal — `factions/gdo/structures/arsenal.md` (design change 0356, slice M5-K): the Rocketman's Unit
//! Prerequisite at the Barracks (`factions/gdo/structures/barracks.md > Produces`: "Rocketman [...] Requires: player
//! owns at least one Arsenal"). It has no panel yet ("Panel: none yet (its research is not designed; it only unlocks
//! the Rocketman for now)").

use crate::defs::structures::{StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/gdo/structures/arsenal.md`: its placeholder stats (design change 0356).
pub(crate) static ARSENAL: ObjectDef = ObjectDef {
    kind: ObjectKind::Arsenal,
    name: "Arsenal",
    faction: Some(Faction::Gdo), // Faction - GlobalDefenseOrganization
    category: Category::Structure,
    footprint: Footprint::Tiles { w: 2, h: 2 }, // Size 2x2
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    destructible: true,
    groupable: true,     // Groupable true
    sight_range: 4.0,    // SightRange 4 grid units
    max_hp: Some(350.0), // MaxHP 350
    max_shields: None,
    armor: Armor {
        point: 1.0, // PointArmor 1
        full: 4.0,  // FullArmor 4
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    unit: None,
    structure: Some(StructureDef {
        symmetry: SymmetryType::AAAA, // SymmetryType AAAA
        build_radius_extension: 2,    // BuildRadiusExtension 2 grid units
        power: -20,                   // Power −20
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
        // TechPrerequisite - player owns at least one Barracks (`factions/gdo/structures/deployment_center.md >
        // Constructs`: "Arsenal: [...] Requires: player owns at least one Barracks"), wherever it is constructed
        // (`crate::tech`).
        requires: &[ObjectKind::Barracks],
        attack: None,
        popup: None,
        holds: None,
        occupants_fire: None,
        // "Allows research of attack and weapon upgrades": none is designed yet.
        research: None,
        abilities: None,
    }),
    default_settings: None,
    energy: None,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// The placeholder stat block, and its TechPrerequisite: a Barracks. It produces and constructs nothing.
    #[test]
    fn the_arsenal_follows_the_design() {
        let structure = super::super::assert_stat_block(
            ObjectKind::Arsenal,
            (2, 2),
            SymmetryType::AAAA,
            350.0,
            (1.0, 4.0),
            4.0,
            true,
            2,
            -20,
        );
        assert_eq!(structure.requires, [ObjectKind::Barracks]);
        assert!(structure.production.is_none() && structure.constructs.is_empty());
        assert!(structure.research.is_none() && structure.attack.is_none());
    }
}
