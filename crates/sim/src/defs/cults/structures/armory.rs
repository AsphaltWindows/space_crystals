//! The Armory — `factions/cults/structures/armory.md`.

use crate::defs::structures::{BuilderGoes, ExitRule, HoldsDef, SiteDef, SitePrice, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;
use crate::sides::SideLetter;

/// `factions/cults/structures/armory.md`: the "Training building where Recruits are converted into combat units".
/// Its pool of Recruits (`> InternalRecruitCapacity - 10`, `> Eject All`) is its `holds` (slice M5-F1,
/// `crate::holding`), its training (`> Training`) comes with slice M5-F2, and Recruits build it from outside
/// ([`ARMORY_SITE`], slice M5-E), its TechPrerequisite checked as the Build is given (`Sim::check_build`).
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
        site: Some(ARMORY_SITE),
        expansion: None,
        // TechPrerequisite - player owns at least one Storage
        requires: &[ObjectKind::Storage],
        attack: None,
        popup: None,
        // `> EntranceSide - one short side (B)`, `> ExitSide - opposite short side (C)`, `> InternalRecruitCapacity -
        // 10`, and `systems/controls/commands.md > Enter`: "Cults — Armory, Cathedral: Recruits only (to be trained)".
        holds: Some(HoldsDef {
            entrance: SideLetter::B,
            exit: ExitRule::Side(SideLetter::C),
            capacity: 10,
            accepts: &[ObjectKind::Recruit],
        }),
    }),
    default_settings: None,
};

/// The Armory's construction by Recruits (`StructureDef::site`, slice M5-E; `factions/cults/economy.md > Cults Building
/// Mechanics`: "Space Crystals cost and construction time for each buildable structure are defined in the Recruit's
/// Constructs list").
pub(crate) const ARMORY_SITE: SiteDef = SiteDef {
    // `factions/cults/units/recruit.md > Constructs`: "Armory: 200 Space Crystals, 480 frames (30 seconds)
    // BaseConstructionTime (with 1 Recruit). Requires: Storage.", paid once as the order is given
    // (`factions/cults/economy.md > Build Command`).
    price: SitePrice::Crystals(200),
    frames: 480,
    start_hp: 0.1, // `> ConstructionHP Rule - applies`: "start at 10% of their maximum HP"
    builder_goes: BuilderGoes::Outside,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::cults::tests::assert_structure;

    /// 2 tall and 3 wide, ABAC, MaxHP 300, PointArmor 1, FullArmor 6 (the building armour rule's n = 1), SightRange 4,
    /// Groupable, behind a Storage. Recruits walk in at its short side B and come out of the opposite one, C, and its
    /// pool holds 10 of them. Recruits build it from outside for 200 Space Crystals in 480 frames with one of them, from
    /// 10% HP (slice M5-E). Its training comes with slice M5-F2.
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
        assert_eq!((structure.production, structure.drop_off), (None, None), "slice M5-F2");
        let site = structure.site.expect("Recruits build it");
        assert_eq!(
            (site.price, site.frames, site.start_hp),
            (SitePrice::Crystals(200), 480, 0.1)
        );
        assert_eq!(site.builder_goes, BuilderGoes::Outside);
        let holds = structure.holds.expect("the Armory holds Recruits");
        assert_eq!(
            (holds.entrance, holds.exit),
            (SideLetter::B, ExitRule::Side(SideLetter::C))
        );
        assert_eq!((holds.capacity, holds.accepts), (10, &[ObjectKind::Recruit][..]));
    }
}
