//! The Recruitment Hall — `factions/cults/economy.md > RecruitmentHall`.

use crate::defs::structures::{StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/cults/economy.md > RecruitmentHall`: "The Cults' starting structure. Automatically produces Recruits from
/// the surrounding population of Recruitable terrain." Its production, its Recruitment Area and its Unit Control
/// (`> RecruitmentHall > RecruitmentArea`, `> RecruitmentHall > Production Behavior`, `> RecruitmentHall > Unit
/// Control Tracking`) come with slice M5-C, its construction by Recruits with slice M5-E, and the starting one with
/// slice M5-B.
pub(crate) static RECRUITMENT_HALL: ObjectDef = ObjectDef {
    kind: ObjectKind::RecruitmentHall,
    name: "Recruitment Hall",
    faction: Some(Faction::Cults), // Faction - TheCults
    category: Category::Structure,
    footprint: Footprint::Tiles { w: 4, h: 4 }, // Size - 4x4 grid units
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    destructible: true,                         // Destructible - true
    groupable: false,                           // Groupable - false
    sight_range: 6.0,                           // SightRange - 6 grid units
    max_hp: Some(1000.0),                       // MaxHP - 1000
    max_shields: None,
    armor: Armor {
        point: 1.0, // PointArmor - 1
        full: 16.0, // FullArmor - 16
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    unit: None,
    structure: Some(StructureDef {
        symmetry: SymmetryType::AAAA, // SymmetryType - AAAA
        // The Cults have neither Power nor a build area (`factions/cults/economy.md > Cults Building Mechanics`:
        // "Cults buildings are constructed on-site by Recruits").
        build_radius_extension: 0,
        power: 0,
        seeds_build_area: false,
        production: None, // `> RecruitmentHall > Production Behavior`: slice M5-C
        mining: None,
        onto: None,
        constructs: &[],
        warp_in: &[],
        drop_off: None,
        underground: false,
        tunnel: None,
        site: None, // built by Recruits: slice M5-E
        expansion: None,
        requires: &[], // TechPrerequisite - None
        attack: None,
        popup: None,
    }),
    default_settings: None,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::cults::tests::assert_structure;

    /// 4×4, AAAA, MaxHP 1000, PointArmor 1, FullArmor 16 (the building armour rule's n = 1), SightRange 6, not
    /// Groupable, no TechPrerequisite. Its production comes with slice M5-C, its site with slice M5-E.
    #[test]
    fn the_recruitment_hall_follows_the_design() {
        let structure = assert_structure(
            ObjectKind::RecruitmentHall,
            (4, 4),
            SymmetryType::AAAA,
            1000.0,
            (1.0, 16.0),
            6.0,
            false,
        );
        assert_eq!(ObjectKind::RecruitmentHall.def().name, "Recruitment Hall");
        assert_eq!(structure.requires, &[][..]);
        assert_eq!(
            (structure.production, structure.drop_off, structure.site),
            (None, None, None),
            "slices M5-C and M5-E"
        );
    }
}
