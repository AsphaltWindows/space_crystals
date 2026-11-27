//! The Supply Tower — `factions/gdo/economy.md > SupplyTower` (design change 0366, slice M6-GK): "GDO supply
//! collection structure. Serves as the drop-off point for supplies gathered by Supply Choppers. Produces Supply
//! Choppers and comes with one free Supply Chopper when placed." Its numbers are the design's (plan decision T84). No
//! player constructs one, and it produces nothing, until slice M6-GS: its production (`> Produces`: "Supply Chopper:
//! 100 Space Crystals, 160 frames (10 seconds)", held: `crate::defs::gdo::units::supply_chopper::RECIPE`), its drop-off
//! and free chopper, and its Deployment Center entry ([`RECIPE`], held) are that slice's.

use crate::defs::structures::{Recipe, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/gdo/economy.md > SupplyTower`.
pub(crate) const SUPPLY_TOWER: ObjectDef = ObjectDef {
    kind: ObjectKind::SupplyTower,
    name: "Supply Tower",
    faction: Some(Faction::Gdo), // Faction - GlobalDefenseOrganization
    category: Category::Structure,
    footprint: Footprint::Tiles { w: 3, h: 3 }, // Size - 3x3 grid units
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    destructible: true,                         // Destructible - true
    groupable: true,                            // Groupable - true
    sight_range: 4.0,                           // SightRange - 4 grid units
    max_hp: Some(400.0),                        // MaxHP - 400
    armor: Armor {
        point: 1.0, // PointArmor - 1
        full: 9.0,  // FullArmor - 9
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    structure: Some(StructureDef {
        symmetry: SymmetryType::AAAA, // SymmetryType - AAAA
        build_radius_extension: 1,    // BuildRadiusExtension - 1 grid unit
        power: -15,                   // Power - -15
        // TechPrerequisite - player owns at least one Radar Station, wherever it is constructed (`crate::tech`).
        requires: &[ObjectKind::RadarStation],
        ..StructureDef::NONE
    }),
    ..ObjectDef::NONE
};

/// `factions/gdo/structures/deployment_center.md > Constructs`: "Supply Tower: 150 Space Crystals, 320 frames (20
/// seconds) (placeholder). Requires: player owns at least one Radar Station" (its own TechPrerequisite). Held until
/// slice M6-GS (`crate::defs::PLANNED_RECIPES`).
pub(crate) const RECIPE: Recipe = Recipe {
    kind: ObjectKind::SupplyTower,
    cost: 150,
    frames: 320,
    requires: &[],
    supplies: 0,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// The stat block, and its TechPrerequisite: a Radar Station. It produces nothing and takes no drop-off yet; its
    /// Deployment Center entry, 150 Space Crystals and 320 frames, is held.
    #[test]
    fn the_supply_tower_follows_the_design() {
        let structure = super::super::assert_stat_block(
            ObjectKind::SupplyTower,
            (3, 3),
            SymmetryType::AAAA,
            400.0,
            (1.0, 9.0),
            4.0,
            true,
            1,
            -15,
        );
        assert_eq!(structure.requires, [ObjectKind::RadarStation]);
        assert!(structure.production.is_none() && structure.drop_off.is_none());
        assert_eq!(RECIPE, super::super::held(ObjectKind::SupplyTower, 150, 320));
    }
}
