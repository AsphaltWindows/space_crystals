//! The Storage — `factions/cults/structures/storage.md`.

use crate::defs::structures::{BuilderGoes, DropOff, SiteDef, SitePrice, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/cults/structures/storage.md`: "Drop-off point for Space Crystals collected by Recruits", and the Armory's
/// TechPrerequisite (`factions/cults/structures/armory.md`). Recruits build it from outside ([`STORAGE_SITE`], slice
/// M5-E), and the starting one comes with slice M5-B. "ObjectInterfaceState: None (info display only)".
pub(crate) static STORAGE: ObjectDef = ObjectDef {
    kind: ObjectKind::Storage,
    name: "Storage",
    faction: Some(Faction::Cults), // Faction - TheCults
    category: Category::Structure,
    footprint: Footprint::Tiles { w: 2, h: 2 }, // Size - 2x2 grid units
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    destructible: true,                         // Destructible - true
    groupable: true,                            // Groupable - true
    sight_range: 3.0,                           // SightRange - 3 grid units
    max_hp: Some(250.0),                        // MaxHP - 250
    max_shields: None,
    // PointArmor - 1, FullArmor - 4 (building armor rule; was 6, a leftover from the old 3x2 size; Q0094): the rule's
    // 1 / 4 for a 2x2 at n = 1 (`core/entities.md > Structure Type > Building armor rule`; slice M5-G2).
    armor: Armor { point: 1.0, full: 4.0 },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    unit: None,
    structure: Some(StructureDef {
        symmetry: SymmetryType::AAAA, // SymmetryType - AAAA
        // The Cults have neither Power nor a build area (`factions/cults/economy.md > Cults Building Mechanics`).
        build_radius_extension: 0,
        power: 0,
        seeds_build_area: false,
        production: None,
        mining: None,
        onto: None,
        constructs: &[],
        warp_in: &[],
        // "Multiple Recruits can drop off Space Crystals simultaneously"; `factions/cults/units/recruit.md >
        // Gathering`: DropOffReach is measured to "the Storage's edge" (slice M5-D).
        drop_off: Some(DropOff::Anywhere),
        underground: false,
        tunnel: None,
        site: Some(STORAGE_SITE),
        expansion: None,
        requires: &[], // TechPrerequisite - None
        attack: None,
        popup: None,
        holds: None,
        occupants_fire: None,
    }),
    default_settings: None,
};

/// The Storage's construction by Recruits (`StructureDef::site`, slice M5-E; `factions/cults/economy.md > Cults
/// Building Mechanics`: "Space Crystals cost and construction time for each buildable structure are defined in the
/// Recruit's Constructs list").
pub(crate) const STORAGE_SITE: SiteDef = SiteDef {
    // `factions/cults/units/recruit.md > Constructs`: "Storage: 100 Space Crystals, 320 frames (20 seconds)
    // BaseConstructionTime (with 1 Recruit)", paid once as the order is given (`factions/cults/economy.md > Build
    // Command`: "the building's Space Crystals cost is deducted as soon as the build order is given").
    price: SitePrice::Crystals(100),
    frames: 320,
    start_hp: 0.1, // `> Storage > ConstructionHP Rule - applies`: "start at 10% of their maximum HP"
    builder_goes: BuilderGoes::Outside,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::cults::tests::assert_structure;

    /// 2×2, AAAA, MaxHP 250, PointArmor 1, FullArmor 4 (the building armour rule's n = 1, Q0094), SightRange 3,
    /// Groupable, no TechPrerequisite; Recruits drop off anywhere at it, any number at once. Recruits build it from
    /// outside for 100 Space Crystals in 320 frames with one of them, from 10% HP (slice M5-E).
    #[test]
    fn the_storage_follows_the_design() {
        let structure = assert_structure(
            ObjectKind::Storage,
            (2, 2),
            SymmetryType::AAAA,
            250.0,
            (1.0, 4.0),
            3.0,
            true,
        );
        assert_eq!(ObjectKind::Storage.def().name, "Storage");
        assert_eq!(structure.requires, &[][..]);
        assert_eq!(
            (structure.production, structure.drop_off),
            (None, Some(DropOff::Anywhere))
        );
        let site = structure.site.expect("Recruits build it");
        assert_eq!(
            (site.price, site.frames, site.start_hp),
            (SitePrice::Crystals(100), 320, 0.1)
        );
        assert_eq!(site.builder_goes, BuilderGoes::Outside);
    }
}
