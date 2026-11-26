//! The Gunworks — `factions/cults/structures/gunworks.md`.

use crate::defs::structures::{BuilderGoes, SiteDef, SitePrice, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/cults/structures/gunworks.md`: the "Tier 2 support building of the Tech/Weaponry branch", which "Unlocks
/// **Gunner** training at the Armory (Gunner's TechPrerequisite)": the Armory's Gunner recipe requires one (slice
/// M5-G2, `defs::cults::structures::armory`), as `crate::tech` counts it. Recruits build it from outside
/// ([`GUNWORKS_SITE`]). Its research isn't designed (`> Research`: "list, costs, BaseResearchTimes TBD"), and "**Until
/// its research is designed it has no panel commands and Recruits can't enter it** (a right-click on it with Recruits
/// is a Move; Q0093)": it holds no one, so the Recruits' right-click on it is a Move (`Sim::aimed`), and its panel is
/// none. Its numbers are the design's "Placeholder stats".
pub(crate) static GUNWORKS: ObjectDef = ObjectDef {
    kind: ObjectKind::Gunworks,
    name: "Gunworks",
    faction: Some(Faction::Cults), // Faction - TheCults
    category: Category::Structure,
    footprint: Footprint::Tiles { w: 2, h: 2 }, // Size 2x2
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    // Not listed; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`).
    destructible: true,
    groupable: true,     // Groupable true
    sight_range: 4.0,    // SightRange 4 grid units
    max_hp: Some(300.0), // MaxHP 300
    max_shields: None,
    armor: Armor {
        point: 1.0, // PointArmor 1
        full: 4.0,  // FullArmor 4
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    unit: None,
    structure: Some(StructureDef {
        // "SymmetryType ABAC, **EntranceSide B, ExitSide C** (like the Armory; user rule 2026-11-26, Q0116: research
        // buildings have an entrance and an exit, and staff leave one at a time)": its sides, for the staffing that
        // comes with its research.
        symmetry: SymmetryType::ABAC,
        // The Cults have neither Power nor a build area (`factions/cults/economy.md > Cults Building Mechanics`).
        build_radius_extension: 0,
        power: 0,
        seeds_build_area: false,
        production: None,
        mining: None,
        onto: None,
        constructs: &[],
        warp_in: &[],
        drop_off: None,
        underground: false,
        tunnel: None,
        site: Some(GUNWORKS_SITE),
        expansion: None,
        requires: &[ObjectKind::Armory], // TechPrerequisite - player owns at least one Armory
        attack: None,
        popup: None,
        // "Recruit staffing capacity 4; research queue 5", once it is enterable: none can enter it yet.
        holds: None,
        occupants_fire: None,
    }),
    default_settings: None,
    energy: None,
};

/// The Gunworks' construction by Recruits (`StructureDef::site`, slice M5-E's; "Space Crystals cost and construction
/// time are listed under the Recruit's Constructs list").
pub(crate) const GUNWORKS_SITE: SiteDef = SiteDef {
    // `factions/cults/units/recruit.md > Constructs`: "Gunworks: 150 Space Crystals, 480 frames (30 seconds)
    // BaseConstructionTime (placeholder). Requires: Armory.", paid once as the order is given
    // (`factions/cults/economy.md > Build Command`).
    price: SitePrice::Crystals(150),
    frames: 480,
    // `factions/cults/economy.md > Cults Building Mechanics`: a Cults building under construction is "vulnerable to
    // attack, same as the structure itself (ConstructionHP Rule)", whose sites "start at 10% of their maximum HP"
    // (`core/entities.md > ConstructionHP Rule`).
    start_hp: 0.1,
    builder_goes: BuilderGoes::Outside,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::cults::tests::assert_structure;

    /// 2×2, ABAC, MaxHP 300, PointArmor 1, FullArmor 4 (the building armour rule's n = 1), SightRange 4, Groupable,
    /// behind an Armory; it holds no one, trains nothing and drops nothing off. Recruits build it from outside for 150
    /// Space Crystals in 480 frames with one of them, from 10% HP.
    #[test]
    fn the_gunworks_follows_the_design() {
        let structure = assert_structure(
            ObjectKind::Gunworks,
            (2, 2),
            SymmetryType::ABAC,
            300.0,
            (1.0, 4.0),
            4.0,
            true,
        );
        assert_eq!(ObjectKind::Gunworks.def().name, "Gunworks");
        assert_eq!(structure.requires, [ObjectKind::Armory], "TechPrerequisite: an Armory");
        assert_eq!(
            (structure.production, structure.drop_off, structure.holds),
            (None, None, None),
            "Recruits can't enter it"
        );
        let site = structure.site.expect("Recruits build it");
        assert_eq!(
            (site.price, site.frames, site.start_hp, site.builder_goes),
            (SitePrice::Crystals(150), 480, 0.1, BuilderGoes::Outside)
        );
    }
}
