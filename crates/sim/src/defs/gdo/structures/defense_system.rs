//! The Defense System — `factions/gdo/structures/defense_system.md` (design change 0366, slice M6-GK): "GDO main
//! defense-related structure. Constructs defensive structure(s)." Its numbers are the design's "Placeholder stats"
//! (plan decision T84). Slice M6-GR makes its Deployment Center entry ([`RECIPE`]) live, and its Missile Turret: it
//! constructs one at a time, as a constructor does (`crate::construction`), "**Works like the Extraction Facility**
//! [...] one slot, Q, no build menu" (`super::missile_turret::RECIPE`), each placed in the GDO build area.

use super::missile_turret;
use crate::defs::structures::{Recipe, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/gdo/structures/defense_system.md`: its placeholder stats (design change 0366).
pub(crate) const DEFENSE_SYSTEM: ObjectDef = ObjectDef {
    kind: ObjectKind::DefenseSystem,
    name: "Defense System",
    faction: Some(Faction::Gdo), // Faction - GlobalDefenseOrganization
    category: Category::Structure,
    footprint: Footprint::Tiles { w: 2, h: 2 }, // Size 2x2
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    destructible: true,
    groupable: true,     // Groupable true
    sight_range: 4.0,    // SightRange 4 grid units
    max_hp: Some(350.0), // MaxHP 350
    armor: Armor {
        point: 1.0, // PointArmor 1
        full: 4.0,  // FullArmor 4
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    structure: Some(StructureDef {
        symmetry: SymmetryType::AAAA, // SymmetryType AAAA
        build_radius_extension: 2,    // BuildRadiusExtension 2 grid units
        power: -20,                   // Power -20
        // TechPrerequisite - player owns at least one Radar Station and at least one Arsenal, wherever it is
        // constructed (`crate::tech`).
        requires: &[ObjectKind::RadarStation, ObjectKind::Arsenal],
        // (M6-GR) `## Constructs: - Missile Turret: 100 Space Crystals, 240 frames (15 seconds) (placeholder)`.
        constructs: &[missile_turret::RECIPE],
        ..StructureDef::NONE
    }),
    ..ObjectDef::NONE
};

/// `factions/gdo/structures/deployment_center.md > Constructs`: "Defense System: 150 Space Crystals, 320 frames (20
/// seconds) (placeholder). Requires: player owns at least one Radar Station and at least one Arsenal" (its own
/// TechPrerequisite). Live since slice M6-GR, in the Deployment Center's `constructs`.
pub(crate) const RECIPE: Recipe = Recipe {
    kind: ObjectKind::DefenseSystem,
    cost: 150,
    frames: 320,
    requires: &[],
    supplies: 0,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// The placeholder stat block, and its TechPrerequisite: a Radar Station and an Arsenal. (M6-GR) It constructs the
    /// Missile Turret, and nothing else, as a constructor (no producer); its Deployment Center entry, 150 Space
    /// Crystals and 320 frames, is live.
    #[test]
    fn the_defense_system_follows_the_design() {
        let structure = super::super::assert_stat_block(
            ObjectKind::DefenseSystem,
            (2, 2),
            SymmetryType::AAAA,
            350.0,
            (1.0, 4.0),
            4.0,
            true,
            2,
            -20,
        );
        assert_eq!(structure.requires, [ObjectKind::RadarStation, ObjectKind::Arsenal]);
        assert_eq!(structure.constructs, [missile_turret::RECIPE]);
        assert!(structure.production.is_none());
        assert_eq!(RECIPE, super::super::held(ObjectKind::DefenseSystem, 150, 320));
        let dc = ObjectKind::DeploymentCenter.def().structure.as_ref().unwrap();
        assert!(dc.constructs.contains(&RECIPE));
    }
}
