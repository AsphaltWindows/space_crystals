//! The Shrine — `factions/cults/structures/shrine.md`.

use crate::defs::structures::{BuilderGoes, SiteDef, SitePrice, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/cults/structures/shrine.md`: "Tier 3 of the Mystical/Human Enhancement branch (sibling of the Cathedral).
/// Recruits gather inside the Shrine and take part in a ritual that charges it with energy". Its numbers and its site
/// only (slice M5-G2, plan decision T69): its ritual (`> Ritual Participation`), its energy (`> Energy`) and its
/// abilities (`> Abilities`) come with its slice, which adds it to the Recruits' builds, so it holds no one and
/// Recruits don't build it yet. Its numbers are the design's "Placeholder stats".
pub(crate) static SHRINE: ObjectDef = ObjectDef {
    kind: ObjectKind::Shrine,
    name: "Shrine",
    faction: Some(Faction::Cults), // Faction - TheCults
    category: Category::Structure,
    footprint: Footprint::Tiles { w: 2, h: 2 }, // Size 2x2
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    // Not listed; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`), and "Evacuation on
    // Destruction applies" (`> Ritual Participation`).
    destructible: true,
    groupable: false, // Groupable **false** (user, 2026-11-26; Q0118: each press casts from one Shrine)
    sight_range: 4.0, // SightRange 4 grid units
    max_hp: Some(300.0), // MaxHP 300
    max_shields: None,
    armor: Armor {
        point: 1.0, // PointArmor 1
        full: 4.0,  // FullArmor 4
    },
    needs_flat_ground: true, // `core/map.md > Elevation > Structures only on flat terrain`
    unit: None,
    structure: Some(StructureDef {
        // "SymmetryType ABAC, **EntranceSide B, ExitSide C** (user, 2026-11-26: like the production and research
        // buildings; participants leave one at a time)": its sides, for its ritual's slice.
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
        site: Some(SHRINE_SITE),
        expansion: None,
        requires: &[ObjectKind::Temple], // TechPrerequisite - player owns at least one Temple
        attack: None,
        popup: None,
        // "Recruit capacity 8": with its ritual's slice.
        holds: None,
        occupants_fire: None,
        research: None,
    }),
    default_settings: None,
};

/// The Shrine's construction by Recruits (`StructureDef::site`, slice M5-E's; "Space Crystals cost and construction
/// time are listed under the Recruit's Constructs list").
pub(crate) const SHRINE_SITE: SiteDef = SiteDef {
    // `factions/cults/units/recruit.md > Constructs`: "Shrine: 200 Space Crystals, 480 frames (30 seconds)
    // BaseConstructionTime (placeholder). Requires: Temple.", paid once as the order is given
    // (`factions/cults/economy.md > Build Command`).
    price: SitePrice::Crystals(200),
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

    /// 2×2, ABAC, MaxHP 300, PointArmor 1, FullArmor 4 (the building armour rule's n = 1), SightRange 4, not Groupable
    /// (Q0118), behind a Temple; it holds no one and casts nothing yet. Recruits will build it from outside for 200
    /// Space Crystals in 480 frames with one of them, from 10% HP.
    #[test]
    fn the_shrine_follows_the_design() {
        let structure = assert_structure(
            ObjectKind::Shrine,
            (2, 2),
            SymmetryType::ABAC,
            300.0,
            (1.0, 4.0),
            4.0,
            false,
        );
        assert_eq!(ObjectKind::Shrine.def().name, "Shrine");
        assert_eq!(structure.requires, [ObjectKind::Temple], "TechPrerequisite: a Temple");
        assert_eq!(
            (structure.production, structure.drop_off, structure.holds),
            (None, None, None)
        );
        let site = structure.site.expect("Recruits build it");
        assert_eq!(
            (site.price, site.frames, site.start_hp, site.builder_goes),
            (SitePrice::Crystals(200), 480, 0.1, BuilderGoes::Outside)
        );
    }
}
