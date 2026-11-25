//! The Optics Lab — `factions/syndicate/structures/optics_lab.md` (design change 0229, answering Q0059: "Placeholder
//! stats ... the Hideout's values with a lower cost"; slice M4-K).

use crate::defs::structures::{ExpansionDef, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/syndicate/structures/optics_lab.md`: "Syndicate tech building enabling the Syndicate's laser weaponry", an
/// underground expansion (plan decision T32) that is only a prerequisite so far ("Commands - none for now: it is only a
/// prerequisite (laser research TBD)"): the Hideout's Guard's (`Recipe::requires`, plan decision T37), and the Laser
/// Turret's (`StructureDef::requires`, slice M4-S1). Hidden from the other players while owned, walked over, and
/// destroyed as any structure. A Tunnel builds it in its Tunnel Area ([`OPTICS_LAB_EXPANSION`]); it counts as a
/// prerequisite once complete (`crate::tech`).
pub(crate) static OPTICS_LAB: ObjectDef = ObjectDef {
    kind: ObjectKind::OpticsLab,
    name: "Optics Lab",
    faction: Some(Faction::Syndicate),          // `factions/syndicate/`
    category: Category::Structure,              // Entity Type - Structure Type (Underground)
    footprint: Footprint::Tiles { w: 2, h: 2 }, // Size - 2x2
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    // Not listed; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`).
    destructible: true,
    groupable: true, // Groupable - true
    // Not listed: an expansion's, `factions/syndicate/tunnels.md > Expansions under construction` (design change 0254):
    // "Sight: none (underground)".
    sight_range: 0.0,
    max_hp: Some(400.0), // HP - 400
    max_shields: None,
    // PointArmor - 6, FullArmor - 24: "underground expansion, n = 6" (`factions/syndicate/structures/README.md > Tunnel
    // Expansions > High armor underground`, design change 0264: "this replaces the 1/4 listed in individual stat
    // blocks").
    armor: Armor { point: 6.0, full: 24.0 },
    // `core/map.md > Elevation` (design change 0005): "Syndicate underground expansions (inside a Tunnel Area) may be
    // placed on sloped tiles; the flatness rule doesn't apply to them".
    needs_flat_ground: false,
    unit: None,
    structure: Some(StructureDef {
        // Not given (plan decision T29): AAAA, since none of its sides has a function.
        symmetry: SymmetryType::AAAA,
        build_radius_extension: 0,
        power: 0,
        seeds_build_area: false,
        production: None, // Commands - none for now
        mining: None,
        onto: None,
        constructs: &[],
        warp_in: &[],
        drop_off: None,
        underground: true, // Entity Type - Structure Type (Underground)
        tunnel: None,
        site: None,
        expansion: Some(OPTICS_LAB_EXPANSION),
        requires: &[], // TechPrerequisite - None
        attack: None,
    }),
    default_settings: None,
};

/// The Optics Lab's construction by a Tunnel (`StructureDef::expansion`, slice M4-F's).
pub(crate) const OPTICS_LAB_EXPANSION: ExpansionDef = ExpansionDef {
    tier: 1,     // Tier Requirement - 1
    cost: 150,   // Cost - 150 Space Crystals (no Supplies)
    frames: 400, // Build Time - 400 frames (25 seconds)
    // `factions/syndicate/tunnels.md > Expansions under construction` (design change 0254): "the ConstructionHP Rule
    // applies to every expansion (starts at 10%, grows with progress)".
    start_hp: 0.1,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// The stat block (design change 0229): an underground structure, 2×2, HP 400, Groupable, no commands and no
    /// TechPrerequisite; built by a Tier 1 Tunnel for 150 Space Crystals in 400 frames, from 10% of its HP (design
    /// change 0254). An expansion's high armor, 6 / 24 (design change 0264), no sight, and slopes allowed (design change
    /// 0005). Not given, so a placeholder: SymmetryType AAAA (plan decision T29).
    #[test]
    fn the_optics_lab_follows_the_design() {
        let def = ObjectKind::OpticsLab.def();
        let structure = def.structure.as_ref().unwrap();
        assert_eq!(
            (def.name, def.category, def.faction, def.tile_size(), structure.symmetry),
            (
                "Optics Lab",
                Category::Structure,
                Some(Faction::Syndicate),
                Some((2, 2)),
                SymmetryType::AAAA
            )
        );
        let armor = Armor { point: 6.0, full: 24.0 };
        assert_eq!(
            (def.max_hp, def.max_shields, def.armor, def.sight_range),
            (Some(400.0), None, armor, 0.0)
        );
        assert_eq!(
            (def.selectable, def.destructible, def.groupable, def.needs_flat_ground),
            (true, true, true, false)
        );
        assert_eq!(
            (structure.underground, structure.production, structure.requires),
            (true, None, &[][..])
        );
        let expected = ExpansionDef {
            tier: 1,
            cost: 150,
            frames: 400,
            start_hp: 0.1,
        };
        assert_eq!((structure.expansion, structure.site), (Some(expected), None));
    }
}
