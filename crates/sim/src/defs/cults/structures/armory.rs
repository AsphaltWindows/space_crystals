//! The Armory — `factions/cults/structures/armory.md`.

use crate::defs::structures::{
    BuilderGoes, ExitRule, HoldsDef, ProductionDef, Recipe, SiteDef, SitePrice, StructureDef, SymmetryType,
};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;
use crate::sides::SideLetter;

/// `factions/cults/structures/armory.md`: the "Training building where Recruits are converted into combat units".
/// Its pool of Recruits (`> InternalRecruitCapacity - 10`, `> Eject All`) is its `holds` (slice M5-F1,
/// `crate::holding`), its training (`> Training`) its `production`, of the Recruits inside (slice M5-F2,
/// `sim/producers/pool.rs`), and Recruits build it from outside ([`ARMORY_SITE`], slice M5-E), its TechPrerequisite
/// checked as the Build is given (`Sim::check_build`).
pub(crate) static ARMORY: ObjectDef = ObjectDef {
    kind: ObjectKind::Armory,
    name: "Armory",
    faction: Some(Faction::Cults), // Faction - TheCults
    category: Category::Structure,
    // Size - 2x3 grid units (2 tall, 3 wide in the default orientation)
    footprint: Footprint::Tiles { w: 3, h: 2 },
    selectable: true,    // `core/entities.md > Object Type > Selectable - true`
    destructible: true,  // Destructible - true
    groupable: false, // Groupable - false (design change 0348 point 5: "Cults production buildings aren't groupable")
    sight_range: 4.0, // SightRange - 4 grid units
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
        // `> Training`: "Training is queueable and serial — orders are processed one at a time, in the order queued";
        // "the trained unit exits from the exit side" (`> ExitSide - opposite short side (C)`), as the units it holds do
        // (`holds`). Of the Recruits inside (slice M5-F2, `sim/producers/pool.rs`), so it recruits nothing.
        production: Some(ProductionDef {
            exits: ExitRule::Side(SideLetter::C),
            // In the KindMenus' order (`> ObjectInterfaceState[Armory]`: "**Q: Train Militant**, **W: Train Gunner**").
            recipes: &[
                // "**Militant** (SpaceCrystalsCost: 20, TrainingTime: 32 frames)"
                Recipe {
                    kind: ObjectKind::Militant,
                    cost: 20,
                    frames: 32,
                    requires: &[],
                },
                // "**Gunner** (SpaceCrystalsCost: 40, TrainingTime: 32 frames). TechPrerequisite: player owns at least
                // one Gunworks", checked as it is queued and never again (`core/tech_and_research.md > Tech
                // Prerequisites > Unit Prerequisites`, `crate::tech`; slice M5-G2).
                Recipe {
                    kind: ObjectKind::Gunner,
                    cost: 40,
                    frames: 32,
                    requires: &[ObjectKind::Gunworks],
                },
            ],
            recruitment: None,
            augment: &[],
        }),
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
        occupants_fire: None,
        research: None,
        abilities: None,
    }),
    default_settings: None,
    energy: None,
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
    /// not Groupable (design change 0348), behind a Storage. Recruits walk in at its short side B and come out of the
    /// opposite one, C, and it holds 10 of them. It trains Militants of them, 20 Space Crystals and 32 frames each, out
    /// of C too (slice M5-F2), and Gunners, 40 Space Crystals and 32 frames each, for a player who owns a Gunworks
    /// (slice M5-G2). Recruits build it from outside for 200 Space Crystals in 480 frames with one of them, from 10% HP
    /// (slice M5-E).
    #[test]
    fn the_armory_follows_the_design() {
        let structure = assert_structure(
            ObjectKind::Armory,
            (3, 2),
            SymmetryType::ABAC,
            300.0,
            (1.0, 6.0),
            4.0,
            false,
        );
        assert_eq!(ObjectKind::Armory.def().name, "Armory");
        assert_eq!(structure.requires, [ObjectKind::Storage], "TechPrerequisite: a Storage");
        assert_eq!(structure.drop_off, None);
        let production = structure.production.expect("it trains");
        assert_eq!(production.exits, ExitRule::Side(SideLetter::C));
        let recipe = |kind| production.recipe(kind).map(|r| (r.cost, r.frames, r.requires));
        assert_eq!(recipe(ObjectKind::Militant), Some((20, 32, &[][..])));
        assert_eq!(
            recipe(ObjectKind::Gunner),
            Some((40, 32, &[ObjectKind::Gunworks][..])),
            "TechPrerequisite: a Gunworks"
        );
        let kinds: Vec<ObjectKind> = production.recipes.iter().map(|r| r.kind).collect();
        assert_eq!(kinds, [ObjectKind::Militant, ObjectKind::Gunner], "Q, then W");
        assert_eq!(production.recruitment, None);
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
