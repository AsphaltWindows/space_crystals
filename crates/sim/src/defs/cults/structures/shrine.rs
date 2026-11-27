//! The Shrine — `factions/cults/structures/shrine.md`.

use crate::defs::structures::{BuilderGoes, ExitRule, HoldsDef, SiteDef, SitePrice, StructureDef, SymmetryType};
use crate::defs::{AreaAbility, Armor, Category, EnergyDef, EnergyRegen, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;
use crate::sides::SideLetter;

/// `factions/cults/structures/shrine.md`: "Tier 3 of the Mystical/Human Enhancement branch (sibling of the Cathedral).
/// Recruits gather inside the Shrine and take part in a ritual that charges it with energy. The Shrine spends that
/// energy on area abilities that support Cults infantry". Its numbers and its site are slice M5-G2's (plan decision
/// T69), and its design's "Placeholder stats"; its ritual (`> Ritual Participation`: the Recruits it holds, `holds`),
/// its energy (`> Energy`, `energy`, `crate::energy`) and its abilities (`> Abilities`, `abilities`, `crate::buffs`)
/// slice M5-N5's. Recruits build it from outside ([`SHRINE_SITE`]).
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
        // buildings; participants leave one at a time)": its sides, `holds`'.
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
        // `> Ritual Participation`: "Recruits enter the Shrine to take part in the ritual [...] Only Recruits can
        // participate. Participating Recruits are not consumed"; "Recruit capacity 8", in at B, out of C one at a
        // time.
        holds: Some(HoldsDef {
            entrance: SideLetter::B,
            exit: ExitRule::Side(SideLetter::C),
            capacity: 8,
            accepts: &[ObjectKind::Recruit],
        }),
        occupants_fire: None,
        research: None,
        // `> Abilities`, the two of its table in the order of its keys: "**Health regeneration** (hotkey Q)",
        // "**Energy regeneration** (hotkey W)".
        abilities: Some(&[AreaAbility::HealthRegeneration, AreaAbility::EnergyRegeneration]),
    }),
    default_settings: None,
    // `> Energy`: "**MaxEnergy**: 200"; `> Placeholder stats`: "MaxEnergy 200; starts at 0; regeneration **0.25
    // energy per second per Recruit inside** (2 per second when full)"; "With no Recruits inside, the Shrine does not
    // regenerate energy".
    energy: Some(EnergyDef {
        max: 200.0,
        start: 0.0,
        regen: EnergyRegen::PerUnitInside(0.25),
    }),
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
    /// (Q0118), behind a Temple. It holds 8 Recruits, in at B and out of C, produces nothing, has a pool of 200 energy
    /// from 0 that regenerates 0.25 a second for each Recruit inside, and casts Health and Energy regeneration.
    /// Recruits build it from outside for 200 Space Crystals in 480 frames with one of them, from 10% HP.
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
        assert_eq!((structure.production, structure.drop_off), (None, None));
        let holds = structure.holds.expect("its ritual's Recruits");
        assert_eq!(
            (holds.entrance, holds.exit, holds.capacity, holds.accepts),
            (
                SideLetter::B,
                ExitRule::Side(SideLetter::C),
                8,
                &[ObjectKind::Recruit][..]
            )
        );
        let abilities = [AreaAbility::HealthRegeneration, AreaAbility::EnergyRegeneration];
        assert_eq!(structure.abilities, Some(&abilities[..]));
        let energy = ObjectKind::Shrine.def().energy.expect("its pool");
        assert_eq!(
            (energy.max, energy.start, energy.regen),
            (200.0, 0.0, EnergyRegen::PerUnitInside(0.25))
        );
        let site = structure.site.expect("Recruits build it");
        assert_eq!(
            (site.price, site.frames, site.start_hp, site.builder_goes),
            (SitePrice::Crystals(200), 480, 0.1, BuilderGoes::Outside)
        );
    }
}
