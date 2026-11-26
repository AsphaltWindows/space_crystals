//! The Scrap Hangar — `factions/cults/structures/scrap_hangar.md`.

use crate::defs::structures::{BuilderGoes, SiteDef, SitePrice, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/cults/structures/scrap_hangar.md`: the "Tier 3 support building of the Tech/Weaponry branch (sits beside
/// the Airstrip)", which "Unlocks production of advanced aircraft at the Airstrip", none of them designed yet (`>
/// Unlocks`). Recruits build it from outside ([`SCRAP_HANGAR_SITE`], slice M5-G2), once their player owns an Airstrip.
/// "No panel commands (it only unlocks)", and no one enters it, so the Recruits' right-click on it is a Move
/// (`Sim::aimed`). Its numbers are the design's "Placeholder stats".
pub(crate) static SCRAP_HANGAR: ObjectDef = ObjectDef {
    kind: ObjectKind::ScrapHangar,
    name: "Scrap Hangar",
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
        symmetry: SymmetryType::AAAA, // SymmetryType AAAA
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
        site: Some(SCRAP_HANGAR_SITE),
        expansion: None,
        requires: &[ObjectKind::Airstrip], // TechPrerequisite - player owns at least one Airstrip
        attack: None,
        popup: None,
        holds: None,
        occupants_fire: None,
        research: None,
    }),
    default_settings: None,
};

/// The Scrap Hangar's construction by Recruits (`StructureDef::site`, slice M5-E's; "Space Crystals cost and
/// construction time are listed under the Recruit's Constructs list").
pub(crate) const SCRAP_HANGAR_SITE: SiteDef = SiteDef {
    // `factions/cults/units/recruit.md > Constructs`: "Scrap Hangar: 200 Space Crystals, 640 frames (40 seconds)
    // BaseConstructionTime (placeholder). Requires: Airstrip.", paid once as the order is given
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

    /// 3×3, AAAA, MaxHP 450, PointArmor 1, FullArmor 9 (the building armour rule's n = 1), SightRange 4, Groupable,
    /// behind an Airstrip; it holds no one, trains nothing and drops nothing off. Recruits build it from outside for 200
    /// Space Crystals in 640 frames with one of them, from 10% HP.
    #[test]
    fn the_scrap_hangar_follows_the_design() {
        let structure = assert_structure(
            ObjectKind::ScrapHangar,
            (3, 3),
            SymmetryType::AAAA,
            450.0,
            (1.0, 9.0),
            4.0,
            true,
        );
        assert_eq!(ObjectKind::ScrapHangar.def().name, "Scrap Hangar");
        assert_eq!(
            structure.requires,
            [ObjectKind::Airstrip],
            "TechPrerequisite: an Airstrip"
        );
        assert_eq!(
            (structure.production, structure.drop_off, structure.holds),
            (None, None, None),
            "it only unlocks"
        );
        let site = structure.site.expect("Recruits build it");
        assert_eq!(
            (site.price, site.frames, site.start_hp, site.builder_goes),
            (SitePrice::Crystals(200), 640, 0.1, BuilderGoes::Outside)
        );
    }
}
