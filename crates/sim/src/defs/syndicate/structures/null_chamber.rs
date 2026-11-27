//! The Null Chamber — `factions/syndicate/structures/null_chamber.md` (design change 0367's placeholders; slice M6-SK).

use crate::defs::structures::{ExpansionDef, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/syndicate/structures/null_chamber.md`: "A shielded dome that works like a Faraday cage against imprinting",
/// an underground expansion behind a Hideout. Its numbers only (slice M6-SK, plan decision T84, by T69's rule): no
/// Tunnel builds one yet ([`NULL_CHAMBER_EXPANSION`], kept out of its def), so its ExpandMenu entry stays `Planned`; its
/// exit buff (`> Null Chamber Exit Buff`: 20 s, +2 grid units of SightRange, 20% less damage taken, four units at a
/// time) comes with slice M6-SN. Its numbers are the design's "Placeholder stats".
pub(crate) const NULL_CHAMBER: ObjectDef = ObjectDef {
    kind: ObjectKind::NullChamber,
    name: "Null Chamber",
    faction: Some(Faction::Syndicate),          // `factions/syndicate/`
    category: Category::Structure,              // Entity Type - Structure Type (Underground)
    footprint: Footprint::Tiles { w: 2, h: 2 }, // Size 2x2
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    // Not listed; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`).
    destructible: true,
    // Not listed: every expansion's, `factions/syndicate/tunnel_menus.md > Expansions under construction` (design change
    // 0254): "Sight: none (underground). **Groupable**: true."
    groupable: true,
    sight_range: 0.0,
    max_hp: Some(400.0),                     // MaxHP 400
    armor: Armor { point: 6.0, full: 24.0 }, // PointArmor 6, FullArmor 24 (underground expansion, n = 6)
    // `core/map.md > Elevation` (design change 0005): "Syndicate underground expansions (inside a Tunnel Area) may be
    // placed on sloped tiles".
    needs_flat_ground: false,
    structure: Some(StructureDef {
        // Not given (plan decision T29): AAAA, since none of its sides has a function.
        symmetry: SymmetryType::AAAA,
        build_radius_extension: 0,
        power: 0,
        underground: true, // Entity Type - Structure Type (Underground)
        expansion: None,   // with slice M6-SN: [`NULL_CHAMBER_EXPANSION`]
        // TechPrerequisite - player owns at least one Hideout (anywhere in the Tunnel Network)
        requires: &[ObjectKind::Hideout],
        ..StructureDef::NONE
    }),
    ..ObjectDef::NONE
};

/// The Null Chamber's construction by a Tunnel (`StructureDef::expansion`), for slice M6-SN to make live.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "slice M6-SN: a Tunnel builds the Null Chamber")
)]
pub(crate) const NULL_CHAMBER_EXPANSION: ExpansionDef = ExpansionDef {
    tier: 2,     // Tier Requirement - 2
    cost: 200,   // Cost 200 Space Crystals
    frames: 480, // Build Time 480 frames (30 seconds)
    // `factions/syndicate/tunnel_menus.md > Expansions under construction` (design change 0254): "the ConstructionHP
    // Rule applies to every expansion (starts at 10%, grows with progress)".
    start_hp: 0.1,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::syndicate::tests::assert_expansion;

    /// The stat block (design change 0367): an underground structure, 2×2, MaxHP 400, the expansions' 6 / 24, behind a
    /// Hideout; a Tier 2 Tunnel's for 200 Space Crystals in 480 frames.
    #[test]
    fn the_null_chamber_follows_the_design() {
        let expansion = ExpansionDef {
            tier: 2,
            cost: 200,
            frames: 480,
            start_hp: 0.1,
        };
        assert_expansion(
            ObjectKind::NullChamber,
            "Null Chamber",
            2,
            400.0,
            &[ObjectKind::Hideout],
        );
        assert_eq!(NULL_CHAMBER_EXPANSION, expansion);
    }
}
