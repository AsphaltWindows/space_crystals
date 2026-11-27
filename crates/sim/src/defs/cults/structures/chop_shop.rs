//! The Chop Shop — `factions/cults/structures/chop_shop.md`.

use crate::defs::structures::{
    BuilderGoes, ExitRule, HoldsDef, ProductionDef, Recipe, SiteDef, SitePrice, StructureDef, SymmetryType,
};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;
use crate::sides::SideLetter;

/// `factions/cults/structures/chop_shop.md`: the "Tier 2 major building of the Tech/Weaponry branch. A scavenger's
/// garage where Recruits are turned into vehicles". Its numbers, its site and its pool (slice M5-G2, plan decision
/// T69), and (M5-N6) its production: the Seeker, trained of two pooled Recruits as the Armory trains a Militant of one
/// (plan decision T71: one pool path for every such building, `sim/producers/pool.rs`), which drives out of the exit
/// doorway on side C; (M5-N10) and the Hand of Heaven of three (`> Production`: "missile platform; 3 Recruits"), its
/// recipe after the Seeker's, in the panel's order (design change 0444: "**Q: Build Seeker**, **W: Build Hand of
/// Heaven**, **E: Build Prayer-mobile**"); (M5-N11) and the Prayer-mobile of two (`> Production`: "wheeled caster; 2
/// Recruits; requires a Temple"), its recipe after the Hand of Heaven's, once its player owns a Temple. Recruits build
/// it (`factions/cults/units/recruit.md > Constructs`) once their player owns an Armory. Its numbers are the design's
/// "Placeholder stats".
pub(crate) const CHOP_SHOP: ObjectDef = ObjectDef {
    kind: ObjectKind::ChopShop,
    name: "Chop Shop",
    faction: Some(Faction::Cults), // Faction - TheCults
    category: Category::Structure,
    footprint: Footprint::Tiles { w: 3, h: 3 }, // Size 3x3
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    // Not listed; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`).
    destructible: true,
    // Groupable **false** (user, 2026-11-26; Q0105: Cults production buildings aren't groupable; design change 0348
    // point 5).
    groupable: false,
    sight_range: 4.0,    // SightRange 4 grid units
    max_hp: Some(450.0), // MaxHP 450
    armor: Armor {
        point: 1.0, // PointArmor 1
        full: 9.0,  // FullArmor 9
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    structure: Some(StructureDef {
        // "SymmetryType ABAC; EntranceSide B, ExitSide C (like the Armory)": A on top and bottom, B on the right, C on
        // the left in the default orientation.
        symmetry: SymmetryType::ABAC,
        // The Cults have neither Power nor a build area (`factions/cults/economy.md > Cults Building Mechanics`).
        build_radius_extension: 0,
        power: 0,
        // `> Production`: "**Production works like the Armory** (user, 2026-11-26; factions/cults/structures/armory.md >
        // Training): Recruits enter and join an internal pool; orders are queueable and serial; queuing an order
        // deducts its Space Crystals cost and **reserves the vehicle's number of Recruits** from the pool (an order
        // can only be queued if enough unreserved Recruits are inside); when it finishes, they are consumed and the
        // vehicle exits to the RallyPoint". Of the Recruits inside (`sim/producers/pool.rs`), so it recruits nothing;
        // out of its exit side, C ("EntranceSide B, ExitSide C (like the Armory)"), as its pool's units are; its queue
        // bounded by its capacity ("so at most 5 Seekers queued").
        production: Some(ProductionDef {
            exits: ExitRule::Side(SideLetter::C),
            recipes: &[
                // "**Seeker**: 2 Recruits, 75 Space Crystals, 320 frames (20 seconds)" ("**Q: Build Seeker**"): its 2
                // Recruits are the Seeker's Unit Control cost (`QueueEntry::units_reserved`;
                // `factions/cults/units/seeker.md`: "consumes **2 Recruits**; Unit Control cost **2**"). It has no
                // TechPrerequisite of its own ("Cost: Chop Shop, 2 Recruits, 75 Space Crystals, 320 frames").
                Recipe {
                    kind: ObjectKind::Seeker,
                    cost: 75,
                    frames: 320,
                    requires: &[],
                    supplies: 0,
                },
                // (M5-N10) "**W: Build Hand of Heaven**" (design change 0444, answering Q0174), "missile platform; 3
                // Recruits": its 3 Recruits are its Unit Control cost, as the Seeker's are
                // (`factions/cults/units/hand_of_heaven.md > Produced at - Chop Shop (consumes **3 Recruits**; Unit
                // Control cost **3**)`), and "Cost: Chop Shop, 3 Recruits, **150 Space Crystals, 480 frames (30 s)**",
                // with no TechPrerequisite of its own.
                Recipe {
                    kind: ObjectKind::HandOfHeaven,
                    cost: 150,
                    frames: 480,
                    requires: &[],
                    supplies: 0,
                },
                // (M5-N11) `factions/cults/units/prayer_mobile.md > Placeholder stats`: "Cost: Chop Shop, 2 Recruits,
                // **125 Space Crystals, 400 frames (25 s)**; requires a Temple" ("**E: Build Prayer-mobile** (greyed
                // "Requires a Temple" until one stands"): its 2 Recruits its Unit Control cost ("consumes **2
                // Recruits**; Unit Control cost **2**"), its TechPrerequisite its own ("player owns at least one
                // **Temple** (core/tech_and_research.md > Unit Prerequisites)").
                Recipe {
                    kind: ObjectKind::PrayerMobile,
                    cost: 125,
                    frames: 400,
                    requires: &[ObjectKind::Temple],
                    supplies: 0,
                },
            ],
            recruitment: None,
            augment: &[],
            // (M5-S8) `core/entities.md > Entrance and exit > Exit Doorway` point 7: "idle wheeled and tracked vehicles
            // don't step aside [...], so a vehicle stopping just outside the exit would block the next one. Every
            // building that releases **wheeled or tracked vehicles** through an exit side (today the Cults **Chop
            // Shop** and the GDO **Factory**) therefore **starts with a RallyPoint 3 grid units straight out from the
            // middle of its exit side** (placeholder distance)" (design changes 0446 and 0447, answering Q0175).
            default_rally: Some(3),
        }),
        site: Some(CHOP_SHOP_SITE),
        requires: &[ObjectKind::Armory], // TechPrerequisite - player owns at least one Armory
        // "EntranceSide B, ExitSide C (like the Armory)"; "InternalRecruitCapacity 10, counting reserved Recruits; so at
        // most 5 Seekers queued (bounded by capacity: reserved units count toward it; Q0104)", the pool buildings' rule
        // (design change 0347 point 5, slice M5-F2's); "Recruits enter and join an internal pool" (`> Production`).
        holds: Some(HoldsDef {
            entrance: SideLetter::B,
            exit: ExitRule::Side(SideLetter::C),
            capacity: 10,
            accepts: &[ObjectKind::Recruit],
        }),
        ..StructureDef::NONE
    }),
    ..ObjectDef::NONE
};

/// The Chop Shop's construction by Recruits (`StructureDef::site`, slice M5-E's; "Space Crystals cost and construction
/// time are listed under the Recruit's Constructs list").
pub(crate) const CHOP_SHOP_SITE: SiteDef = SiteDef {
    // `factions/cults/units/recruit.md > Constructs`: "Chop Shop: 200 Space Crystals, 640 frames (40 seconds)
    // BaseConstructionTime (placeholder). Requires: Armory.", paid once as the order is given
    // (`factions/cults/economy.md > Build Command`).
    price: SitePrice::Crystals(200),
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

    /// 3×3, ABAC, MaxHP 450, PointArmor 1, FullArmor 9 (the building armour rule's n = 1), SightRange 4, not Groupable
    /// (design change 0348), behind an Armory. Its pool: Recruits walk in at side B and come out of side C, 10 of them,
    /// reserved ones counted. It trains the Seeker of two Recruits for 75 Space Crystals in 320 frames, (M5-N10) the
    /// Hand of Heaven of three for 150 in 480 and (M5-N11) the Prayer-mobile of two for 125 in 400 once a Temple
    /// stands, in that order, out of side C. Recruits build it from outside for 200 Space Crystals in 640 frames with
    /// one of them, from 10% HP.
    #[test]
    fn the_chop_shop_follows_the_design() {
        let structure = assert_structure(
            ObjectKind::ChopShop,
            (3, 3),
            SymmetryType::ABAC,
            450.0,
            (1.0, 9.0),
            4.0,
            false,
        );
        assert_eq!(ObjectKind::ChopShop.def().name, "Chop Shop");
        assert_eq!(structure.requires, [ObjectKind::Armory], "TechPrerequisite: an Armory");
        assert_eq!(structure.drop_off, None);
        let production = structure.production.expect("it trains vehicles");
        assert_eq!(production.exits, ExitRule::Side(SideLetter::C));
        assert_eq!(
            production.recipes,
            [
                Recipe {
                    kind: ObjectKind::Seeker,
                    cost: 75,
                    frames: 320,
                    requires: &[],
                    supplies: 0,
                },
                Recipe {
                    kind: ObjectKind::HandOfHeaven,
                    cost: 150,
                    frames: 480,
                    requires: &[],
                    supplies: 0,
                },
                Recipe {
                    kind: ObjectKind::PrayerMobile,
                    cost: 125,
                    frames: 400,
                    requires: &[ObjectKind::Temple],
                    supplies: 0,
                }
            ]
        );
        assert_eq!((production.recruitment, production.augment), (None, &[][..]));
        let recruits = |kind: ObjectKind| kind.def().unit.as_ref().and_then(|u| u.unit_control_cost);
        assert_eq!(recruits(ObjectKind::Seeker), Some(2), "2 Recruits");
        assert_eq!(recruits(ObjectKind::HandOfHeaven), Some(3), "3 Recruits");
        assert_eq!(recruits(ObjectKind::PrayerMobile), Some(2), "2 Recruits");
        let holds = structure.holds.expect("its pool");
        assert_eq!(
            (holds.entrance, holds.exit),
            (SideLetter::B, ExitRule::Side(SideLetter::C))
        );
        assert_eq!((holds.capacity, holds.accepts), (10, &[ObjectKind::Recruit][..]));
        let site = structure.site.expect("Recruits build it");
        assert_eq!(
            (site.price, site.frames, site.start_hp, site.builder_goes),
            (SitePrice::Crystals(200), 640, 0.1, BuilderGoes::Outside)
        );
    }
}
