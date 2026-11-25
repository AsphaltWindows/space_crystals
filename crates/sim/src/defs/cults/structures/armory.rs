//! The Armory — `factions/cults/structures/armory.md`.

use crate::defs::structures::{StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/cults/structures/armory.md`: the "Training building where Recruits are converted into combat units".
/// Its pool of Recruits (`> InternalRecruitCapacity - 10`, `> Eject All`) comes with slice M5-F1, its training
/// (`> Training`) with slice M5-F2, and its construction by Recruits with slice M5-E, which checks its
/// TechPrerequisite: until then, `requires` is data only.
pub(crate) static ARMORY: ObjectDef = ObjectDef {
    kind: ObjectKind::Armory,
    name: "Armory",
    faction: Some(Faction::Cults), // Faction - TheCults
    category: Category::Structure,
    // Size - 2x3 grid units (2 tall, 3 wide in the default orientation)
    footprint: Footprint::Tiles { w: 3, h: 2 },
    selectable: true,    // `core/entities.md > Object Type > Selectable - true`
    destructible: true,  // Destructible - true
    groupable: true,     // Groupable - true
    sight_range: 4.0,    // SightRange - 4 grid units
    max_hp: Some(300.0), // MaxHP - 300
    max_shields: None,
    armor: Armor {
        point: 1.0, // PointArmor - 1
        full: 6.0,  // FullArmor - 6
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    unit: None,
    structure: Some(StructureDef {
        // SymmetryType - ABAC (top A, right B, bottom A, left C): `> EntranceSide - one short side (B)`, `> ExitSide -
        // opposite short side (C)`.
        symmetry: SymmetryType::ABAC,
        // The Cults have neither Power nor a build area (`factions/cults/economy.md > Cults Building Mechanics`).
        build_radius_extension: 0,
        power: 0,
        seeds_build_area: false,
        production: None, // `> Training`: slice M5-F2
        mining: None,
        onto: None,
        constructs: &[],
        warp_in: &[],
        drop_off: None,
        underground: false,
        tunnel: None,
        site: None, // built by Recruits: slice M5-E
        expansion: None,
        // TechPrerequisite - player owns at least one Storage
        requires: &[ObjectKind::Storage],
        attack: None,
        popup: None,
    }),
    default_settings: None,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::cults::tests::assert_structure;

    /// 2 tall and 3 wide, ABAC, MaxHP 300, PointArmor 1, FullArmor 6 (the building armour rule's n = 1), SightRange 4,
    /// Groupable, behind a Storage. Its pool, its training and its site come with slices M5-F1, M5-F2 and M5-E.
    #[test]
    fn the_armory_follows_the_design() {
        let structure = assert_structure(
            ObjectKind::Armory,
            (3, 2),
            SymmetryType::ABAC,
            300.0,
            (1.0, 6.0),
            4.0,
            true,
        );
        assert_eq!(ObjectKind::Armory.def().name, "Armory");
        assert_eq!(structure.requires, [ObjectKind::Storage], "TechPrerequisite: a Storage");
        assert_eq!(
            (structure.production, structure.drop_off, structure.site),
            (None, None, None),
            "slices M5-F2 and M5-E"
        );
    }
}
