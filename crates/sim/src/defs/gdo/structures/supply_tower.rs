//! The Supply Tower — `factions/gdo/economy.md > SupplyTower` (design change 0366, slice M6-GK): "GDO supply
//! collection structure. Serves as the drop-off point for supplies gathered by Supply Choppers. Produces Supply
//! Choppers and comes with one free Supply Chopper when placed." Its numbers are the design's (plan decision T84).
//! The Deployment Center constructs it ([`RECIPE`]), it produces Supply Choppers (`> Produces`: "Supply
//! Chopper: 100 Space Crystals, 160 frames (10 seconds)"), comes with one, attached, and takes their Supplies
//! (`crate::supply_run`).

use crate::defs::gdo::units::supply_chopper;
use crate::defs::structures::{ExitRule, ProductionDef, Recipe, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind, SupplyTowerDef};
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
        // `> Produces`: "Supply Chopper: 100 Space Crystals, 160 frames (10 seconds)". **Placeholder (not in
        // the design: where a new chopper appears)**: in the air over the tower's centre (`ExitRule::Pad` with no
        // Airfield's slots, `Sim::release_onto_slot`), its spot being its attached chopper's.
        production: Some(ProductionDef {
            exits: ExitRule::Pad,
            recipes: &[supply_chopper::RECIPE],
            recruitment: None,
            augment: &[],
            default_rally: None,
        }),
        // `> On Placement` and `> Landing and Attachment` (`crate::supply_run`).
        supply_tower: Some(SUPPLY_LOOP),
        ..StructureDef::NONE
    }),
    ..ObjectDef::NONE
};

/// Its part in the supply loop (`SupplyTowerDef`): `> On Placement`, "One Supply Chopper is spawned on the
/// tower at no additional cost [...] automatically attached to the tower"; one landing spot, at its centre
/// (**Placeholder (not in the design: where on the tower a chopper lands)**).
pub(crate) const SUPPLY_LOOP: SupplyTowerDef = SupplyTowerDef {
    chopper: ObjectKind::SupplyChopper,
    spot_offset_su: [0.0, 0.0],
};

/// `factions/gdo/structures/deployment_center.md > Constructs`: "Supply Tower: 150 Space Crystals, 320 frames (20
/// seconds) (placeholder). Requires: player owns at least one Radar Station" (its own TechPrerequisite), in the
/// Deployment Center's list (`super::deployment_center`).
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

    /// The stat block, and its TechPrerequisite: a Radar Station. It produces the Supply Chopper, 100 Space
    /// Crystals and 160 frames, released over its centre, and comes with one (`SupplyTowerDef`), landing on its centre;
    /// it takes no worker's drop-off. Its Deployment Center entry is 150 Space Crystals and 320 frames.
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
        let production = structure.production.as_ref().expect("it produces");
        assert_eq!(production.exits, ExitRule::Pad);
        assert_eq!(production.recipes, [supply_chopper::RECIPE]);
        assert!(structure.drop_off.is_none() && structure.airfield.is_none());
        assert_eq!(
            structure.supply_tower,
            Some(SupplyTowerDef {
                chopper: ObjectKind::SupplyChopper,
                spot_offset_su: [0.0, 0.0],
            })
        );
        assert_eq!(RECIPE, super::super::held(ObjectKind::SupplyTower, 150, 320));
    }
}
