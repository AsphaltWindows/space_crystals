//! The Recruitment Hall — `factions/cults/structures/recruitment_hall.md`.

use crate::defs::structures::{
    BuilderGoes, ExitRule, ProductionDef, Recipe, RecruitmentDef, SiteDef, SitePrice, StructureDef, SymmetryType,
};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;
use crate::sides::SideLetter;

/// `factions/cults/structures/recruitment_hall.md`: "The Cults' starting structure. Automatically produces Recruits
/// from the surrounding population of Recruitable terrain." Its production, its Recruitment Area and its Unit Control
/// (`> RecruitmentHall > RecruitmentArea`, `> RecruitmentHall > Production Behavior`, `> RecruitmentHall > Unit
/// Control Tracking`) are slice M5-C's (`crate::recruitment`), its construction by Recruits slice M5-E's
/// ([`RECRUITMENT_HALL_SITE`]), and the starting one slice M5-B's.
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
            augment: &[],
        }),
        mining: None,
        onto: None,
        constructs: &[],
        warp_in: &[],
        drop_off: None,
        underground: false,
        tunnel: None,
        site: Some(RECRUITMENT_HALL_SITE),
        expansion: None,
        requires: &[], // TechPrerequisite - None
        attack: None,
        popup: None,
        holds: None,
        occupants_fire: None,
        research: None,
    }),
    default_settings: None,
    energy: None,
};

/// The Recruitment Hall's construction by Recruits (`StructureDef::site`, slice M5-E; `factions/cults/economy.md >
/// Cults Building Mechanics`: "Space Crystals cost and construction time for each buildable structure are defined in the
/// Recruit's Constructs list"). Its production starts once it is complete (`Sim::start_functions`), so it claims its
/// tiles from that tick (`Production::completed_at`; `> RecruitmentHall > TileClaiming`: "the hall that was constructed
/// first has priority").
pub(crate) const RECRUITMENT_HALL_SITE: SiteDef = SiteDef {
    // `factions/cults/units/recruit.md > Constructs`: "RecruitmentHall: 300 Space Crystals, 1440 frames (90 seconds)
    // BaseConstructionTime (with 1 Recruit)", paid once as the order is given (`factions/cults/economy.md > Build
    // Command`).
    price: SitePrice::Crystals(300),
    frames: 1440,
    start_hp: 0.1, // `> RecruitmentHall > ConstructionHP Rule - applies`: "start at 10% of their maximum HP"
    builder_goes: BuilderGoes::Outside,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::cults::tests::assert_structure;

    /// 4×4, AAAB (design change 0342), MaxHP 1000, PointArmor 1, FullArmor 16 (the building armour rule's n = 1),
    /// SightRange 6, not Groupable, no TechPrerequisite. It recruits Recruits for free, 192 frames each at 100%
    /// effectiveness, out of its one exit side, B (0342), with a 10x10 Recruitment Area and a capacity of 20 (slice
    /// M5-C). Recruits build it from outside for 300 Space Crystals in 1440 frames with one of them, from 10% HP (slice
    /// M5-E).
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
        assert_eq!(structure.drop_off, None);
        let site = structure.site.expect("Recruits build it");
        assert_eq!(
            (site.price, site.frames, site.start_hp),
            (SitePrice::Crystals(300), 1440, 0.1)
        );
        assert_eq!(site.builder_goes, BuilderGoes::Outside);
    }
}
