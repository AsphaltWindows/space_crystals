//! The Temple — `factions/cults/structures/temple.md`.

use crate::defs::structures::{
    BuilderGoes, ExitRule, HoldsDef, ProductionDef, ResearchDef, SiteDef, SitePrice, StructureDef, SymmetryType,
};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind, Tech};
use crate::player::Faction;
use crate::sides::SideLetter;

/// `factions/cults/structures/temple.md`: "Tier 2 of the Mystical/Human Enhancement branch. Researches technologies for
/// the Cults. Research follows the shared rules in core/tech_and_research.md > Research, including the Cults exception:
/// research only progresses while Recruits are staffing the building (speed scales with the number of Recruits
/// inside)". Its numbers and its site are slice M5-G2's (plan decision T69), and its design's "Placeholder stats"; its
/// research (`> Research (first set)`, `StructureDef::research`) runs on its queue (`production`, plan decision T70),
/// staffed by the Recruits it holds (`holds`, `> Recruit Staffing`; slice M5-N3, `sim/producers/research.rs`). Recruits
/// build it from outside ([`TEMPLE_SITE`]).
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
        // Its research queue (plan decision T70: "one queue engine"), which produces no unit: `research` says what it
        // takes. Its staff come out of the exit side as the units of a building that holds units do (`holds`).
        production: Some(ProductionDef {
            exits: ExitRule::Side(SideLetter::C),
            recipes: &[],
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
        site: Some(TEMPLE_SITE),
        expansion: None,
        requires: &[ObjectKind::Altar], // TechPrerequisite - player owns at least one Altar
        attack: None,
        popup: None,
        // `> Recruit Staffing`: "Recruits enter the Temple to staff its research (right-click own Temple with Recruits
        // selected → Enter Temple). Only Recruits can staff research"; "Recruit staffing capacity 6"; "EntranceSide B,
        // ExitSide C", "staff leave one at a time".
        holds: Some(HoldsDef {
            entrance: SideLetter::B,
            exit: ExitRule::Side(SideLetter::C),
            capacity: 6,
            accepts: &[ObjectKind::Recruit],
        }),
        occupants_fire: None,
        // `> Research (first set)`, the four of its table in its order; "research queue 5".
        research: Some(ResearchDef {
            techs: &Tech::ALL,
            queue: 5,
        }),
    }),
    default_settings: None,
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
    /// behind an Altar. Six Recruits staff it, walking in at B and out of C; it researches the four technologies of
    /// its table, five of them queued at most, and trains no unit (slice M5-N3). Recruits build it from outside for 200
    /// Space Crystals in 640 frames with one of them, from 10% HP.
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
        assert_eq!(structure.drop_off, None);
        let holds = structure.holds.expect("Recruits staff it");
        assert_eq!(
            (holds.entrance, holds.exit, holds.capacity, holds.accepts),
            (
                SideLetter::B,
                ExitRule::Side(SideLetter::C),
                6,
                &[ObjectKind::Recruit][..]
            )
        );
        let research = structure.research.expect("it researches");
        assert_eq!((research.techs, research.queue), (&Tech::ALL[..], 5));
        let production = structure.production.expect("on its queue");
        assert!(production.recipes.is_empty() && production.augment.is_empty() && production.recruitment.is_none());
        assert_eq!(production.exits, ExitRule::Side(SideLetter::C));
        let site = structure.site.expect("Recruits build it");
        assert_eq!(
            (site.price, site.frames, site.start_hp, site.builder_goes),
            (SitePrice::Crystals(200), 640, 0.1, BuilderGoes::Outside)
        );
    }
}
