//! The Orbital Command — `factions/syndicate/structures/orbital_command.md` (design change 0367's placeholders; slice
//! M6-SK).

use crate::defs::structures::{ExpansionDef, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/syndicate/structures/orbital_command.md`: the "Syndicate facility allowing for control and coordination of
/// orbital forces", an underground expansion behind an Aerospace Assembly, and a prerequisite only ("No commands for now
/// (prerequisite only)"): the Prism's. Its numbers only (slice M6-SK, plan decision T84, by T69's rule): no Tunnel
/// builds one yet ([`ORBITAL_COMMAND_EXPANSION`], kept out of its def, for slice M6-SP), so its ExpandMenu entry stays
/// `Planned`. Its numbers are the design's "Placeholder stats".
pub(crate) const ORBITAL_COMMAND: ObjectDef = ObjectDef {
    kind: ObjectKind::OrbitalCommand,
    name: "Orbital Command",
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
        production: None,  // No commands for now (prerequisite only)
        underground: true, // Entity Type - Structure Type (Underground)
        expansion: None,   // with slice M6-SP: [`ORBITAL_COMMAND_EXPANSION`]
        // TechPrerequisite - player owns at least one Aerospace Assembly (anywhere in the Tunnel Network)
        requires: &[ObjectKind::AerospaceAssembly],
        ..StructureDef::NONE
    }),
    ..ObjectDef::NONE
};

/// The Orbital Command's construction by a Tunnel (`StructureDef::expansion`), for slice M6-SP to make live.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "slice M6-SP: a Tunnel builds the Orbital Command")
)]
pub(crate) const ORBITAL_COMMAND_EXPANSION: ExpansionDef = ExpansionDef {
    tier: 3,     // Tier Requirement - 3
    cost: 250,   // Cost 250 Space Crystals
    frames: 640, // Build Time 640 frames (40 seconds)
    // `factions/syndicate/tunnel_menus.md > Expansions under construction` (design change 0254): "the ConstructionHP
    // Rule applies to every expansion (starts at 10%, grows with progress)".
    start_hp: 0.1,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::syndicate::tests::assert_expansion;

    /// The stat block (design change 0367): an underground structure, 2×2, MaxHP 400, the expansions' 6 / 24, behind an
    /// Aerospace Assembly, with no commands; a Tier 3 Tunnel's for 250 Space Crystals in 640 frames.
    #[test]
    fn the_orbital_command_follows_the_design() {
        let expansion = ExpansionDef {
            tier: 3,
            cost: 250,
            frames: 640,
            start_hp: 0.1,
        };
        let requires = [ObjectKind::AerospaceAssembly];
        assert_expansion(ObjectKind::OrbitalCommand, "Orbital Command", 2, 400.0, &requires);
        assert_eq!(ORBITAL_COMMAND_EXPANSION, expansion);
    }
}
