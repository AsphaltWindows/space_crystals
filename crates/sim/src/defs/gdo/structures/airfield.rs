//! The Airfield — `factions/gdo/structures/airfield.md` (design change 0366, slice M6-GK): "GDO aircraft production
//! structure. Constructs aircraft and provides a landing/reloading zone for them." Its numbers are the design's
//! "Placeholder stats" (plan decision T84). Since slice M6-GH1 it trains the Hornet (`> Produces (so far)`), and
//! supports at most 4 aircraft (`> Aircraft Support`, `AirfieldDef`; `crate::aircraft`). (M6-GH2) Its aircraft land on
//! its 4 landing slots and rearm there, a new one appearing landed on a free slot (`ExitRule::Pad`, `crate::landing`),
//! and the Deployment Center constructs it ([`RECIPE`], behind Advanced Mechanics). (M6-GL) It trains the Dragonfly
//! too, behind Propulsion Labs, which lands on its slots and takes off from them vertically.

use crate::defs::gdo::units::{dragonfly, hornet};
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
        // (M6-GH1) `> Produces (so far)`: the Hornet, and (M6-GL) the Dragonfly ("Requires: player owns at least one
        // Propulsion Labs", its Unit Prerequisite), in the design's order.
        production: Some(ProductionDef {
            exits: ExitRule::Pad,
            recipes: &[hornet::RECIPE, dragonfly::RECIPE],
            recruitment: None,
            augment: &[],
            default_rally: None,
        }),
        // (M6-GH1) "**Capacity: each Airfield supports at most 4 aircraft.**" (M6-GH2) "An Airfield has **4 landing
        // slots**, one per possible assignment"; "Landing and take-off: **16 frames (1 s)** each (placeholder)"; the
        // slots' layout the designer's placeholder ("a 2x2 grid"), the quarters' centres.
        airfield: Some(AIRFIELD_SUPPORT),
        ..StructureDef::NONE
    }),
    ..ObjectDef::NONE
};

/// (M6-GH2) Its support of its aircraft (`AirfieldDef`): "**Capacity: each Airfield supports at most 4 aircraft**",
/// on "**4 landing slots**", landing and taking off in "**16 frames (1 s)** each (placeholder)"
/// (`factions/gdo/structures/airfield.md > Aircraft Support`, `systems/mechanics/aircraft_landing.md > Assigned base`),
/// the slots 48 su from its centre along both axes (**Placeholder (Q0189 point 4)**, the designer's: "**Slot layout**
/// [...]: a 2x2 grid, one slot near each corner of the footprint").
pub(crate) const AIRFIELD_SUPPORT: AirfieldDef = AirfieldDef {
    capacity: 4,
    landing_ticks: 16,
    slot_offset_su: 48.0,
};

/// `factions/gdo/structures/deployment_center.md > Constructs`: "Airfield: 250 Space Crystals, 480 frames (30 seconds)
/// (placeholder). Requires: player owns at least one Advanced Mechanics" (its own TechPrerequisite). The Deployment
/// Center's since slice M6-GH2, which lands its aircraft (`super::deployment_center`).
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

    /// The placeholder stat block, and its TechPrerequisite: Advanced Mechanics. (M6-GH1) It trains the Hornet, (M6-GL)
    /// then the Dragonfly, and supports at most 4 aircraft; it holds and constructs nothing. (M6-GH2) Its aircraft land
    /// on its 4 slots in 16 frames, and appear landed on them (`ExitRule::Pad`); its slots lie 48 su from its centre
    /// along both axes, each a Hornet's Silhouette inside its footprint. Its Deployment Center entry: 250 Space
    /// Crystals and 480 frames.
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
        assert_eq!(production.recipes, [hornet::RECIPE, dragonfly::RECIPE]);
        let support = AirfieldDef {
            capacity: 4,
            landing_ticks: 16,
            slot_offset_su: 48.0,
        };
        assert_eq!(structure.airfield, Some(support));
        let half_footprint = 3.0 * crate::units::SU_PER_GU / 2.0;
        let hornet = ObjectKind::Hornet.def().half_extents().expect("a Silhouette");
        assert!(
            support.slot_offset_su + hornet.max_element() <= half_footprint,
            "inside the footprint"
        );
        assert!(
            2.0 * support.slot_offset_su >= 2.0 * hornet.max_element(),
            "apart from the next"
        );
        assert!(structure.holds.is_none() && structure.constructs.is_empty());
        assert_eq!(RECIPE, super::super::held(ObjectKind::Airfield, 250, 480));
    }
}
