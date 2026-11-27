//! The Airstrip — `factions/cults/structures/airstrip.md`.

use crate::defs::structures::{
    BuilderGoes, ExitRule, HoldsDef, ProductionDef, Recipe, SiteDef, SitePrice, StructureDef, SymmetryType,
};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;
use crate::sides::SideLetter;

/// `factions/cults/structures/airstrip.md`: the "Tier 3 major building of the Tech/Weaponry branch. A makeshift runway
/// where Recruits are turned into aircraft". Its numbers, its site and its pool (slice M5-G2, plan decision T69), and
/// (M5-N7) its production: the Sparrow, trained of a pooled Recruit as at the Armory, which "**takes off from the
/// Airstrip**" along its runway (plan decision T71's `ExitRule::Runway`). Its numbers are the design's "Placeholder
/// stats". The Advanced aircraft (`> Production`: "a large paratrooper plane and an airship/zeppelin [...] require the
/// player to own a Scrap Hangar") are TBD, and so not here.
pub(crate) static AIRSTRIP: ObjectDef = ObjectDef {
    kind: ObjectKind::Airstrip,
    name: "Airstrip",
    faction: Some(Faction::Cults), // Faction - TheCults
    category: Category::Structure,
    // "Size 2x4 (a runway)": 2 tall and 4 wide in the default orientation, so that its short sides, B and C, are the
    // left and right ones, as the Armory's 2x3 has them.
    footprint: Footprint::Tiles { w: 4, h: 2 },
    selectable: true, // `core/entities.md > Object Type > Selectable - true`
    // Not listed; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`).
    destructible: true,
    // Groupable **false** (user, 2026-11-26; Q0105: Cults production buildings aren't groupable; design change 0348
    // point 5).
    groupable: false,
    sight_range: 4.0,    // SightRange 4 grid units
    max_hp: Some(400.0), // MaxHP 400
    max_shields: None,
    armor: Armor {
        point: 1.0, // PointArmor 1
        full: 8.0,  // FullArmor 8
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    unit: None,
    structure: Some(StructureDef {
        // "SymmetryType ABAC with the short sides B and C": A on top and bottom, B on the right, C on the left.
        symmetry: SymmetryType::ABAC,
        // The Cults have neither Power nor a build area (`factions/cults/economy.md > Cults Building Mechanics`).
        build_radius_extension: 0,
        power: 0,
        seeds_build_area: false,
        // `> Production`: "**Production works like the Armory** [...]: Recruits enter an internal pool, orders are serial
        // and queueable and reserve the aircraft's number of Recruits, which are consumed on completion. A finished
        // aircraft **takes off from the Airstrip** to its RallyPoint"; "a finished aircraft is placed **directly on the
        // runway** and takes off along it toward C (the exception to exit-opposite-entrance, Q0104)": over its B end,
        // the end the Recruits walk in at, toward C (`ExitRule::Runway`). Of the Recruits inside (`sim/producers/pool.rs`),
        // so it recruits nothing; its queue is bounded by its capacity ("so at most 10 orders queued").
        production: Some(ProductionDef {
            exits: ExitRule::Runway {
                from: SideLetter::B,
                toward: SideLetter::C,
            },
            recipes: &[
                // "**Sparrow**: 1 Recruit, 75 Space Crystals, 320 frames (20 seconds)", the one aircraft it builds so
                // far ("**Q: Build Sparrow**"): 1 Recruit, the Sparrow's Unit Control cost (`QueueEntry::units_reserved`).
                // It has no TechPrerequisite of its own (`factions/cults/units/sparrow.md`: "Cost: Airstrip, 1 Recruit,
                // 75 Space Crystals, 320 frames").
                Recipe {
                    kind: ObjectKind::Sparrow,
                    cost: 75,
                    frames: 320,
                    requires: &[],
                },
            ],
            recruitment: None,
            augment: &[],
            default_rally: None,
        }),
        mining: None,
        onto: None,
        constructs: &[],
        warp_in: &[],
        drop_off: None,
        underground: false,
        tunnel: None,
        site: Some(AIRSTRIP_SITE),
        expansion: None,
        requires: &[ObjectKind::ChopShop], // TechPrerequisite - player owns at least one Chop Shop
        attack: None,
        popup: None,
        // "EntranceSide B (Recruits walk in)"; "InternalRecruitCapacity 10, counting reserved Recruits; so at most 10
        // orders queued (bounded by capacity: reserved units count toward it; Q0104)", the pool buildings' rule (design
        // change 0347 point 5, slice M5-F2's); and `systems/controls/commands.md > Enter`: Recruits only. Its aircraft
        // take off "along it toward C" (`production`). **Placeholder (not in the design: where its ejected and released
        // Recruits come out)**: the exit doorway of side C, the end its aircraft take off toward, opposite its entrance
        // as every pool building's exit is (`factions/cults/structures/README.md > Choosing Recruits, Release and
        // Eject All`: "every unreserved unit leaves, one at a time through the exit").
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

/// The Airstrip's construction by Recruits (`StructureDef::site`, slice M5-E's; "Space Crystals cost and construction
/// time are listed under the Recruit's Constructs list").
pub(crate) const AIRSTRIP_SITE: SiteDef = SiteDef {
    // `factions/cults/units/recruit.md > Constructs`: "Airstrip: 250 Space Crystals, 640 frames (40 seconds)
    // BaseConstructionTime (placeholder). Requires: Chop Shop.", paid once as the order is given
    // (`factions/cults/economy.md > Build Command`).
    price: SitePrice::Crystals(250),
    frames: 640,
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

    /// 2 tall and 4 wide (a runway), ABAC, MaxHP 400, PointArmor 1, FullArmor 8 (the building armour rule's n = 1),
    /// SightRange 4, not Groupable (design change 0348), behind a Chop Shop. Its pool: Recruits walk in at side B, 10 of
    /// them, reserved ones counted, and come out of side C (placeholder). It trains the Sparrow of one Recruit for 75
    /// Space Crystals in 320 frames, which takes off along its runway from B toward C. Recruits build it from outside
    /// for 250 Space Crystals in 640 frames with one of them, from 10% HP.
    #[test]
    fn the_airstrip_follows_the_design() {
        let structure = assert_structure(
            ObjectKind::Airstrip,
            (4, 2),
            SymmetryType::ABAC,
            400.0,
            (1.0, 8.0),
            4.0,
            false,
        );
        assert_eq!(ObjectKind::Airstrip.def().name, "Airstrip");
        assert_eq!(
            structure.requires,
            [ObjectKind::ChopShop],
            "TechPrerequisite: a Chop Shop"
        );
        assert_eq!(structure.drop_off, None);
        let production = structure.production.expect("it trains aircraft");
        assert_eq!(
            production.exits,
            ExitRule::Runway {
                from: SideLetter::B,
                toward: SideLetter::C
            }
        );
        assert_eq!(
            production.recipes,
            [Recipe {
                kind: ObjectKind::Sparrow,
                cost: 75,
                frames: 320,
                requires: &[]
            }]
        );
        assert_eq!((production.recruitment, production.augment), (None, &[][..]));
        assert_eq!(
            ObjectKind::Sparrow
                .def()
                .unit
                .as_ref()
                .and_then(|u| u.unit_control_cost),
            Some(1),
            "1 Recruit"
        );
        let holds = structure.holds.expect("its pool");
        assert_eq!(
            (holds.entrance, holds.exit),
            (SideLetter::B, ExitRule::Side(SideLetter::C))
        );
        assert_eq!((holds.capacity, holds.accepts), (10, &[ObjectKind::Recruit][..]));
        let site = structure.site.expect("Recruits build it");
        assert_eq!(
            (site.price, site.frames, site.start_hp, site.builder_goes),
            (SitePrice::Crystals(250), 640, 0.1, BuilderGoes::Outside)
        );
    }
}
