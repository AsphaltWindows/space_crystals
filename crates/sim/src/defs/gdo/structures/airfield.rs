//! The Airfield — `factions/gdo/structures/airfield.md` (design change 0366, slice M6-GK): "GDO aircraft production
//! structure. Constructs aircraft and provides a landing/reloading zone for them." Its numbers are the design's
//! "Placeholder stats" (plan decision T84). Since slice M6-GH1 it trains the Hornet (`> Produces (so far)`), whose
//! aircraft appear over it (`ExitRule::Pad`), and supports at most 4 aircraft (`> Aircraft Support`, `AirfieldDef`;
//! `crate::aircraft`). Its landing slots and rearming are slice M6-GH2's, and so is its Deployment Center entry
//! ([`RECIPE`], held until then): until it, only a debug spawn places one.

use crate::defs::gdo::units::hornet;
use crate::defs::structures::{AirfieldDef, ExitRule, ProductionDef, Recipe, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/gdo/structures/airfield.md`: its placeholder stats (design change 0366).
pub(crate) const AIRFIELD: ObjectDef = ObjectDef {
    kind: ObjectKind::Airfield,
    name: "Airfield",
    faction: Some(Faction::Gdo), // Faction - GlobalDefenseOrganization
    category: Category::Structure,
    footprint: Footprint::Tiles { w: 3, h: 3 }, // Size 3x3
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    destructible: true,
    groupable: true,     // Groupable true
    sight_range: 5.0,    // SightRange 5 grid units
    max_hp: Some(500.0), // MaxHP 500
    armor: Armor {
        point: 1.0, // PointArmor 1
        full: 9.0,  // FullArmor 9
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    structure: Some(StructureDef {
        symmetry: SymmetryType::AAAA, // SymmetryType AAAA
        build_radius_extension: 2,    // BuildRadiusExtension 2 grid units
        power: -30,                   // Power -30
        // TechPrerequisite - player owns at least one Advanced Mechanics, wherever it is constructed (`crate::tech`).
        requires: &[ObjectKind::AdvancedMechanics],
        // (M6-GH1) `> Produces (so far)`: the Hornet. The Dragonfly ("Requires: player owns at least one Propulsion
        // Labs") is slice M6-GL's, held till then (`crate::defs::PLANNED_RECIPES`).
        production: Some(ProductionDef {
            exits: ExitRule::Pad,
            recipes: &[hornet::RECIPE],
            recruitment: None,
            augment: &[],
            default_rally: None,
        }),
        // (M6-GH1) "**Capacity: each Airfield supports at most 4 aircraft.**"
        airfield: Some(AirfieldDef { capacity: 4 }),
        ..StructureDef::NONE
    }),
    ..ObjectDef::NONE
};

/// `factions/gdo/structures/deployment_center.md > Constructs`: "Airfield: 250 Space Crystals, 480 frames (30 seconds)
/// (placeholder). Requires: player owns at least one Advanced Mechanics" (its own TechPrerequisite). Held until slice
/// M6-GH2, which lands its aircraft (`crate::defs::PLANNED_RECIPES`).
pub(crate) const RECIPE: Recipe = Recipe {
    kind: ObjectKind::Airfield,
    cost: 250,
    frames: 480,
    requires: &[],
    supplies: 0,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// The placeholder stat block, and its TechPrerequisite: Advanced Mechanics. (M6-GH1) It trains the Hornet, whose
    /// aircraft appear over its pads, and supports at most 4 aircraft; it holds and constructs nothing. Its Deployment
    /// Center entry, 250 Space Crystals and 480 frames, is held.
    #[test]
    fn the_airfield_follows_the_design() {
        let structure = super::super::assert_stat_block(
            ObjectKind::Airfield,
            (3, 3),
            SymmetryType::AAAA,
            500.0,
            (1.0, 9.0),
            5.0,
            true,
            2,
            -30,
        );
        assert_eq!(structure.requires, [ObjectKind::AdvancedMechanics]);
        let production = structure.production.expect("it trains aircraft");
        assert_eq!(production.exits, ExitRule::Pad);
        assert_eq!(production.recipes, [hornet::RECIPE]);
        assert_eq!(structure.airfield, Some(AirfieldDef { capacity: 4 }));
        assert!(structure.holds.is_none() && structure.constructs.is_empty());
        assert_eq!(RECIPE, super::super::held(ObjectKind::Airfield, 250, 480));
    }
}
