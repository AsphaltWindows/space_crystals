//! The Storage — `factions/cults/economy.md > Storage`.

use crate::defs::structures::{StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/cults/economy.md > Storage`: "Drop-off point for Space Crystals collected by Recruits", and the Armory's
/// TechPrerequisite (`factions/cults/structures/armory.md`). Its drop-off comes with slice M5-D, its construction by
/// Recruits with slice M5-E, and the starting one with slice M5-B. "ObjectInterfaceState: None (info display only)".
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
    // PointArmor - 1, FullArmor - 6, as written. **Placeholder (Q0094)**: the building armour rule
    // (`core/entities.md > Structure Type > Building armor rule`) gives a 2x2 building 1 / 4 at n = 1; 1 / 6 is a
    // 3x2's, the Storage's size before it went "2x2 (down from 3x2)". The armour test lists it as an exception until
    // the answer.
    armor: Armor { point: 1.0, full: 6.0 },
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
        drop_off: None, // the Recruits' drop-off: slice M5-D
        underground: false,
        tunnel: None,
        site: None, // built by Recruits: slice M5-E
        expansion: None,
        requires: &[], // TechPrerequisite - None
        attack: None,
        popup: None,
        holds: None,
    }),
    default_settings: None,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::cults::tests::assert_structure;

    /// 2×2, AAAA, MaxHP 250, PointArmor 1, FullArmor 6 (Placeholder (Q0094): the rule's would be 4), SightRange 3,
    /// Groupable, no TechPrerequisite. Its drop-off comes with slice M5-D, its site with slice M5-E.
    #[test]
    fn the_storage_follows_the_design() {
        let structure = assert_structure(
            ObjectKind::Storage,
            (2, 2),
            SymmetryType::AAAA,
            250.0,
            (1.0, 6.0),
            3.0,
            true,
        );
        assert_eq!(ObjectKind::Storage.def().name, "Storage");
        assert_eq!(structure.requires, &[][..]);
        assert_eq!(
            (structure.production, structure.drop_off, structure.site),
            (None, None, None),
            "slices M5-D and M5-E"
        );
    }
}
