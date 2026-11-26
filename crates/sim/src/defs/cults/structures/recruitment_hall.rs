//! The Recruitment Hall — `factions/cults/economy.md > RecruitmentHall`.

use crate::defs::structures::{ExitRule, ProductionDef, Recipe, RecruitmentDef, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;
use crate::sides::SideLetter;

/// `factions/cults/economy.md > RecruitmentHall`: "The Cults' starting structure. Automatically produces Recruits from
/// the surrounding population of Recruitable terrain." Its production, its Recruitment Area and its Unit Control
/// (`> RecruitmentHall > RecruitmentArea`, `> RecruitmentHall > Production Behavior`, `> RecruitmentHall > Unit
/// Control Tracking`) are slice M5-C's (`crate::recruitment`), its construction by Recruits slice M5-E's, and the
/// starting one slice M5-B's.
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
        // SymmetryType - AAAB (design change 0342, answering Q0100: "It becomes AAAB with the exit on B (the side is a
        // placeholder)").
        symmetry: SymmetryType::AAAB,
        // The Cults have neither Power nor a build area (`factions/cults/economy.md > Cults Building Mechanics`:
        // "Cults buildings are constructed on-site by Recruits").
        build_radius_extension: 0,
        power: 0,
        seeds_build_area: false,
        // `> RecruitmentHall > Production Behavior`: "automatically and continuously produces Recruits at no resource
        // cost until LocalUnitControlUsage reaches LocalUnitControlCapacity" (plan decision T57).
        production: Some(ProductionDef {
            // "Produced Recruits come out of the exit side (B) and move to its rally point" (design change 0342:
            // "Recruits come out of it one at a time through the Exit Doorway").
            exits: ExitRule::Side(SideLetter::B),
            recipes: &[Recipe {
                kind: ObjectKind::Recruit,
                cost: 0,     // "at no resource cost"
                frames: 192, // BaseRecruitProductionTime - 12 seconds (192 frames) per Recruit, at 100% effectiveness
                requires: &[],
            }],
            recruitment: Some(RecruitmentDef {
                area: 10,     // RecruitmentArea: "A 10x10 grid unit area centered on the 4x4 building"
                capacity: 20, // BaseUnitControlCapacity - 20
            }),
        }),
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
        holds: None,
    }),
    default_settings: None,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::cults::tests::assert_structure;

    /// 4×4, AAAB (design change 0342), MaxHP 1000, PointArmor 1, FullArmor 16 (the building armour rule's n = 1),
    /// SightRange 6, not Groupable, no TechPrerequisite. It recruits Recruits for free, 192 frames each at 100%
    /// effectiveness, out of its one exit side, B (0342), with a 10x10 Recruitment Area and a capacity of 20 (slice
    /// M5-C). Its site comes with slice M5-E.
    #[test]
    fn the_recruitment_hall_follows_the_design() {
        let structure = assert_structure(
            ObjectKind::RecruitmentHall,
            (4, 4),
            SymmetryType::AAAB,
            1000.0,
            (1.0, 16.0),
            6.0,
            false,
        );
        assert_eq!(ObjectKind::RecruitmentHall.def().name, "Recruitment Hall");
        assert_eq!(structure.requires, &[][..]);
        let production = structure.production.expect("it produces");
        assert_eq!(production.exits, ExitRule::Side(SideLetter::B));
        let recipe = |kind| production.recipe(kind).map(|r| (r.cost, r.frames, r.requires));
        assert_eq!(recipe(ObjectKind::Recruit), Some((0, 192, &[][..])));
        assert_eq!(production.recipes.len(), 1);
        let recruitment = production.recruitment.expect("it recruits");
        assert_eq!(
            (recruitment.area, recruitment.capacity, recruitment.tiles()),
            (10, 20, 100)
        );
        assert_eq!((structure.drop_off, structure.site), (None, None), "slice M5-E");
    }
}
