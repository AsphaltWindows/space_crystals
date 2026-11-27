//! The Radar Station — `factions/gdo/structures/radar_station.md` (design change 0366, slice M6-GK): "GDO
//! reconnaissance structure. Has an ability to scan and grant vision of an area on the map on a cooldown." Its numbers
//! are the design's "Placeholder stats" (plan decision T84). No player constructs one until slice M6-GR, which brings
//! its Scan ("reveals a circle of radius **6 grid units** for **8 seconds** (128 frames); **cooldown 60 seconds** (960
//! frames)") and makes its Deployment Center entry ([`RECIPE`], held) live.

use crate::defs::structures::{Recipe, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/gdo/structures/radar_station.md`: its placeholder stats (design change 0366).
pub(crate) const RADAR_STATION: ObjectDef = ObjectDef {
    kind: ObjectKind::RadarStation,
    name: "Radar Station",
    faction: Some(Faction::Gdo), // Faction - GlobalDefenseOrganization
    category: Category::Structure,
    footprint: Footprint::Tiles { w: 2, h: 2 }, // Size 2x2
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    destructible: true,
    // "**Groupable false** (user, 2026-11-26; Q0156: each Scan press scans from one station, as the Shrine; Q, Tab, Q
    // scans two places)" (design change 0422).
    groupable: false,
    sight_range: 8.0,    // SightRange 8 grid units
    max_hp: Some(300.0), // MaxHP 300
    armor: Armor {
        point: 1.0, // PointArmor 1
        full: 4.0,  // FullArmor 4
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    structure: Some(StructureDef {
        symmetry: SymmetryType::AAAA, // SymmetryType AAAA
        build_radius_extension: 2,    // BuildRadiusExtension 2 grid units
        power: -20,                   // Power -20
        // TechPrerequisite - player owns at least one Generator, wherever it is constructed (`crate::tech`).
        requires: &[ObjectKind::Generator],
        ..StructureDef::NONE
    }),
    ..ObjectDef::NONE
};

/// `factions/gdo/structures/deployment_center.md > Constructs`: "Radar Station: 150 Space Crystals, 320 frames (20
/// seconds) (placeholder). Requires: player owns at least one Generator" (its own TechPrerequisite). Held until slice
/// M6-GR (`crate::defs::PLANNED_RECIPES`).
pub(crate) const RECIPE: Recipe = Recipe {
    kind: ObjectKind::RadarStation,
    cost: 150,
    frames: 320,
    requires: &[],
    supplies: 0,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// The placeholder stat block, not Groupable (design change 0422), and its TechPrerequisite: a Generator. It has
    /// no ability yet; its Deployment Center entry, 150 Space Crystals and 320 frames, is held.
    #[test]
    fn the_radar_station_follows_the_design() {
        let structure = super::super::assert_stat_block(
            ObjectKind::RadarStation,
            (2, 2),
            SymmetryType::AAAA,
            300.0,
            (1.0, 4.0),
            8.0,
            false,
            2,
            -20,
        );
        assert_eq!(structure.requires, [ObjectKind::Generator]);
        assert!(structure.abilities.is_none());
        assert_eq!(RECIPE, super::super::held(ObjectKind::RadarStation, 150, 320));
    }
}
