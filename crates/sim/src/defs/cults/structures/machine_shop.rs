//! The Machine Shop — `factions/cults/structures/machine_shop.md`.

use crate::defs::structures::{BuilderGoes, SiteDef, SitePrice, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/cults/structures/machine_shop.md`: the "Tier 2 support building of the Tech/Weaponry branch (sits beside the
/// Chop Shop)", which "Researches upgrades for vehicles and aircraft". Recruits build it from outside
/// ([`MACHINE_SHOP_SITE`], slice M5-G2), once their player owns a Chop Shop. Its research isn't designed (`> Research`:
/// "list, costs, BaseResearchTimes TBD"), and "Until its research is designed it has no panel commands and Recruits
/// can't enter it": it holds no one, so the Recruits' right-click on it is a Move (`Sim::aimed`), and its panel is
/// none. Its numbers are the design's "Placeholder stats".
pub(crate) const MACHINE_SHOP: ObjectDef = ObjectDef {
    kind: ObjectKind::MachineShop,
    name: "Machine Shop",
    faction: Some(Faction::Cults), // Faction - TheCults
    category: Category::Structure,
    footprint: Footprint::Tiles { w: 2, h: 2 }, // Size 2x2
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    // Not listed; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`).
    destructible: true,
    groupable: true,     // Groupable true
    sight_range: 4.0,    // SightRange 4 grid units
    max_hp: Some(300.0), // MaxHP 300
    armor: Armor {
        point: 1.0, // PointArmor 1
        full: 4.0,  // FullArmor 4
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    structure: Some(StructureDef {
        // "SymmetryType ABAC, **EntranceSide B, ExitSide C** (like the Armory; user rule 2026-11-26, Q0116: research
        // buildings have an entrance and an exit, and staff leave one at a time)": its sides, for the staffing that
        // comes with its research.
        symmetry: SymmetryType::ABAC,
        // The Cults have neither Power nor a build area (`factions/cults/economy.md > Cults Building Mechanics`).
        build_radius_extension: 0,
        power: 0,
        site: Some(MACHINE_SHOP_SITE),
        requires: &[ObjectKind::ChopShop], // TechPrerequisite - player owns at least one Chop Shop
        // "Recruit staffing capacity 4; research queue 5", once it is enterable: none can enter it yet.
        holds: None,
        ..StructureDef::NONE
    }),
    ..ObjectDef::NONE
};

/// The Machine Shop's construction by Recruits (`StructureDef::site`, slice M5-E's; "Space Crystals cost and
/// construction time are listed under the Recruit's Constructs list").
pub(crate) const MACHINE_SHOP_SITE: SiteDef = SiteDef {
    // `factions/cults/units/recruit.md > Constructs`: "Machine Shop: 150 Space Crystals, 480 frames (30 seconds)
    // BaseConstructionTime (placeholder). Requires: Chop Shop.", paid once as the order is given
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
    /// behind a Chop Shop; it holds no one, trains nothing and drops nothing off. Recruits build it from outside for 150
    /// Space Crystals in 480 frames with one of them, from 10% HP.
    #[test]
    fn the_machine_shop_follows_the_design() {
        let structure = assert_structure(
            ObjectKind::MachineShop,
            (2, 2),
            SymmetryType::ABAC,
            300.0,
            (1.0, 4.0),
            4.0,
            true,
        );
        assert_eq!(ObjectKind::MachineShop.def().name, "Machine Shop");
        assert_eq!(
            structure.requires,
            [ObjectKind::ChopShop],
            "TechPrerequisite: a Chop Shop"
        );
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
