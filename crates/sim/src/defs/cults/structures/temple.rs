//! The Temple — `factions/cults/structures/temple.md`.

use crate::defs::structures::{BuilderGoes, SiteDef, SitePrice, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/cults/structures/temple.md`: "Tier 2 of the Mystical/Human Enhancement branch. Researches technologies for
/// the Cults", staffed by Recruits. Its numbers and its site only (slice M5-G2, plan decision T69): its research (`>
/// Research (first set)`) and its staff (`> Recruit Staffing`) come with its slice (plan decision T70), which adds it to
/// the Recruits' builds, so it holds no one and Recruits don't build it yet. Its numbers are the design's "Placeholder
/// stats".
pub(crate) static TEMPLE: ObjectDef = ObjectDef {
    kind: ObjectKind::Temple,
    name: "Temple",
    faction: Some(Faction::Cults), // Faction - TheCults
    category: Category::Structure,
    footprint: Footprint::Tiles { w: 3, h: 3 }, // Size 3x3
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    // Not listed; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`).
    destructible: true,
    groupable: true,     // Groupable true
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
        // "SymmetryType ABAC, **EntranceSide B, ExitSide C** (like the Armory; user rule 2026-11-26, Q0116: research
        // buildings have an entrance and an exit, and staff leave one at a time)": its sides, for its staff's slice.
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
        site: Some(TEMPLE_SITE),
        expansion: None,
        requires: &[ObjectKind::Altar], // TechPrerequisite - player owns at least one Altar
        attack: None,
        popup: None,
        // "Recruit staffing capacity 6; research queue 5": with its research's slice.
        holds: None,
        occupants_fire: None,
    }),
    default_settings: None,
    energy: None,
};

/// The Temple's construction by Recruits (`StructureDef::site`, slice M5-E's; "Space Crystals cost and construction
/// time are listed under the Recruit's Constructs list").
pub(crate) const TEMPLE_SITE: SiteDef = SiteDef {
    // `factions/cults/units/recruit.md > Constructs`: "Temple: 200 Space Crystals, 640 frames (40 seconds)
    // BaseConstructionTime (placeholder). Requires: Altar.", paid once as the order is given
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

    /// 3×3, ABAC, MaxHP 450, PointArmor 1, FullArmor 9 (the building armour rule's n = 1), SightRange 4, Groupable,
    /// behind an Altar; it holds no one and researches nothing yet. Recruits will build it from outside for 200 Space
    /// Crystals in 640 frames with one of them, from 10% HP.
    #[test]
    fn the_temple_follows_the_design() {
        let structure = assert_structure(
            ObjectKind::Temple,
            (3, 3),
            SymmetryType::ABAC,
            450.0,
            (1.0, 9.0),
            4.0,
            true,
        );
        assert_eq!(ObjectKind::Temple.def().name, "Temple");
        assert_eq!(structure.requires, [ObjectKind::Altar], "TechPrerequisite: an Altar");
        assert_eq!(
            (structure.production, structure.drop_off, structure.holds),
            (None, None, None)
        );
        let site = structure.site.expect("Recruits build it");
        assert_eq!(
            (site.price, site.frames, site.start_hp, site.builder_goes),
            (SitePrice::Crystals(200), 640, 0.1, BuilderGoes::Outside)
        );
    }
}
