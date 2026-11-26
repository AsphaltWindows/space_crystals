//! The Chop Shop — `factions/cults/structures/chop_shop.md`.

use crate::defs::structures::{BuilderGoes, ExitRule, HoldsDef, SiteDef, SitePrice, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;
use crate::sides::SideLetter;

/// `factions/cults/structures/chop_shop.md`: the "Tier 2 major building of the Tech/Weaponry branch. A scavenger's
/// garage where Recruits are turned into vehicles". Its numbers, its site and its pool's capacity only (slice M5-G2,
/// plan decision T69): its production (`> Production`, the Seeker and the Hand of Heaven, of several Recruits each;
/// plan decision T71) comes with its slice, which adds it to the Recruits' builds, so it trains nothing and Recruits
/// don't build it yet. Its numbers are the design's "Placeholder stats".
pub(crate) static CHOP_SHOP: ObjectDef = ObjectDef {
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
    max_shields: None,
    armor: Armor {
        point: 1.0, // PointArmor 1
        full: 9.0,  // FullArmor 9
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    unit: None,
    structure: Some(StructureDef {
        // "SymmetryType ABAC; EntranceSide B, ExitSide C (like the Armory)": A on top and bottom, B on the right, C on
        // the left in the default orientation.
        symmetry: SymmetryType::ABAC,
        // The Cults have neither Power nor a build area (`factions/cults/economy.md > Cults Building Mechanics`).
        build_radius_extension: 0,
        power: 0,
        seeds_build_area: false,
        // Its vehicles: with its production's slice.
        production: None,
        mining: None,
        onto: None,
        constructs: &[],
        warp_in: &[],
        drop_off: None,
        underground: false,
        tunnel: None,
        site: Some(CHOP_SHOP_SITE),
        expansion: None,
        requires: &[ObjectKind::Armory], // TechPrerequisite - player owns at least one Armory
        attack: None,
        popup: None,
        // "EntranceSide B, ExitSide C (like the Armory)"; "InternalRecruitCapacity 10, counting reserved Recruits; so at
        // most 5 Seekers queued (bounded by capacity: reserved units count toward it; Q0104)", the pool buildings' rule
        // (design change 0347 point 5, slice M5-F2's); "Recruits enter and join an internal pool" (`> Production`).
        holds: Some(HoldsDef {
            entrance: SideLetter::B,
            exit: ExitRule::Side(SideLetter::C),
            capacity: 10,
            accepts: &[ObjectKind::Recruit],
        }),
    }),
    default_settings: None,
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
    /// reserved ones counted; it trains nothing yet. Recruits will build it from outside for 200 Space Crystals in 640
    /// frames with one of them, from 10% HP.
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
        assert_eq!((structure.production, structure.drop_off), (None, None));
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
