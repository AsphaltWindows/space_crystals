//! The Radar Station — `factions/gdo/structures/radar_station.md` (design change 0366, slice M6-GK): "GDO
//! reconnaissance structure. Has an ability to scan and grant vision of an area on the map on a cooldown." Its numbers
//! are the design's "Placeholder stats" (plan decision T84). Slice M6-GR brings its Scan ([`SCAN`],
//! `AreaAbility::Scan`, plan decision T92: "reveals a circle of radius **6 grid units** for **8 seconds** (128
//! frames); **cooldown 60 seconds** (960 frames)"), whose numbers are the GDO's rules' (`GdoRules::scan`), and makes
//! its Deployment Center entry ([`RECIPE`]) live.

use crate::defs::cults::abilities::{AbilityArea, AbilityDef, Affects, AreaAbility, BuffEffect};
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
        // (M6-GR) "**Scan** (hotkey **Q**, targeted ...)", cast from the station (`Command::Cast`).
        abilities: Some(&[AreaAbility::Scan]),
        ..StructureDef::NONE
    }),
    ..ObjectDef::NONE
};

/// `factions/gdo/structures/deployment_center.md > Constructs`: "Radar Station: 150 Space Crystals, 320 frames (20
/// seconds) (placeholder). Requires: player owns at least one Generator" (its own TechPrerequisite). Live since slice
/// M6-GR, in the Deployment Center's `constructs`.
pub(crate) const RECIPE: Recipe = Recipe {
    kind: ObjectKind::RadarStation,
    cost: 150,
    frames: 320,
    requires: &[],
    supplies: 0,
};

/// (M6-GR) `> Placeholder stats`: "**Scan** (hotkey **Q**, targeted: left-click any point on the map): reveals a circle
/// of radius **6 grid units** for **8 seconds** (128 frames); **cooldown 60 seconds** (960 frames); free. Each Radar
/// Station has its own cooldown" (`GdoRules::scan`; `crate::vision::scans`, `Object::cooldown`).
pub(crate) const SCAN: AbilityDef = AbilityDef {
    name: "Scan",
    cost: 0.0,                   // free
    area: AbilityArea::AnyPoint, // targeted: left-click any point on the map
    radius: 6.0,                 // a circle of radius 6 grid units
    // A scan is sight, not a buff: it reaches no unit.
    affects: Affects::Own(&[]),
    effect: BuffEffect::Sight, // reveals a circle
    duration: 8.0,             // for 8 seconds (128 frames)
    cooldown: 60.0,            // cooldown 60 seconds (960 frames)
};

#[cfg(test)]
mod tests {
    use super::*;

    /// The placeholder stat block, not Groupable (design change 0422), and its TechPrerequisite: a Generator. (M6-GR)
    /// Its Scan, a circle of 6 grid units round any point, seen for 128 ticks, free, on a cooldown of 960, and no buff;
    /// its Deployment Center entry, 150 Space Crystals and 320 frames, is live.
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
        assert_eq!(structure.abilities, Some(&[AreaAbility::Scan][..]));
        let scan = AreaAbility::Scan;
        assert_eq!(scan.def(), &SCAN);
        assert_eq!((scan.def().name, scan.def().radius), ("Scan", 6.0));
        assert_eq!((scan.duration(), scan.cooldown()), (128, 960));
        assert_eq!((scan.def().cost, scan.def().area), (0.0, AbilityArea::AnyPoint));
        assert_eq!(scan.def().effect, BuffEffect::Sight);
        assert!(ObjectKind::ALL.iter().all(|&kind| !scan.affects(kind)), "no buff");
        assert_eq!(RECIPE, super::super::held(ObjectKind::RadarStation, 150, 320));
        let dc = ObjectKind::DeploymentCenter.def().structure.as_ref().unwrap();
        assert!(dc.constructs.contains(&RECIPE));
    }

    /// T101's checklist (`Ruleset::structural_diff`): Scan's numbers (its radius, duration, cooldown and cost) apply
    /// live; where it is cast and what it does, and the Radar Station's list of abilities, are structure. The Cults'
    /// table of abilities (`CultsRules::abilities`) has no entry for Scan, whose def is the GDO's (`GdoRules::scan`): a
    /// file that gives it one is refused.
    #[test]
    fn scan_s_numbers_apply_live_and_the_rest_is_structure() {
        use crate::ruleset::Ruleset;
        let builtin = Ruleset::builtin();
        let changed = |change: &dyn Fn(&mut Ruleset)| {
            let mut rules = Ruleset::builtin();
            change(&mut rules);
            builtin.structural_diff(&rules).map(|at| at.field)
        };
        assert_eq!(changed(&|r| r.factions.gdo.scan.radius = 9.0), None);
        assert_eq!(changed(&|r| r.factions.gdo.scan.duration = 1.0), None);
        assert_eq!(changed(&|r| r.factions.gdo.scan.cooldown = 5.0), None);
        assert_eq!(changed(&|r| r.factions.gdo.scan.cost = 10.0), None);
        assert_eq!(
            changed(&|r| r.factions.gdo.scan.area = AbilityArea::AtPoint),
            Some("area")
        );
        assert_eq!(
            changed(&|r| r.factions.gdo.scan.effect = BuffEffect::HoldFire),
            Some("effect")
        );
        fn station(r: &mut Ruleset) -> &mut StructureDef {
            r.defs.get_mut(ObjectKind::RadarStation).structure.as_mut().unwrap()
        }
        assert_eq!(changed(&|r| station(r).abilities = None), Some("abilities"));
        let mut json = serde_json::to_value(Ruleset::builtin()).unwrap();
        let abilities = &mut json["factions"]["cults"]["abilities"];
        assert!(abilities.get("Inspire").is_some() && abilities.get("Scan").is_none());
        abilities["Scan"] = serde_json::to_value(SCAN).unwrap();
        let refused = serde_json::from_value::<Ruleset>(json).unwrap_err();
        assert!(
            refused.to_string().contains("Scan isn't one of the table's keys"),
            "{refused}"
        );
    }
}
