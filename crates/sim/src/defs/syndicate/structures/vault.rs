//! The Vault — `factions/syndicate/structures/vault.md` (design change 0367's placeholders; slice M6-SK).

use crate::defs::structures::{ExpansionDef, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/syndicate/structures/vault.md`: the Syndicate's "equipment storage warehouse for re-arming Syndicate
/// forces", an underground expansion, the Shadebringer's prerequisite and, with a Chemical Plant, the Specter's. Its
/// numbers only (slice M6-SK, plan decision T84, by T69's rule): no Tunnel builds one yet, its construction
/// ([`VAULT_EXPANSION`]) being kept out of its def, so the gate refuses its Expand (`Reject::CannotConstruct`) and its
/// ExpandMenu entry stays `Planned`; its re-arming and its refilling of tanks, network-wide and two units at a time
/// (`> Placeholder stats`: "**Throughput**"), come with slice M6-SB. Its numbers are the design's "Placeholder stats".
pub(crate) const VAULT: ObjectDef = ObjectDef {
    kind: ObjectKind::Vault,
    name: "Vault",
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
        expansion: None,   // with slice M6-SB: [`VAULT_EXPANSION`]
        // TechPrerequisite - player owns at least one Hideout (anywhere in the Tunnel Network)
        requires: &[ObjectKind::Hideout],
        ..StructureDef::NONE
    }),
    ..ObjectDef::NONE
};

/// The Vault's construction by a Tunnel (`StructureDef::expansion`), for slice M6-SB to make live: until then no Tunnel
/// builds a Vault.
#[cfg_attr(not(test), expect(dead_code, reason = "slice M6-SB: a Tunnel builds the Vault"))]
pub(crate) const VAULT_EXPANSION: ExpansionDef = ExpansionDef {
    tier: 1,     // Tier Requirement - 1
    cost: 150,   // Cost 150 Space Crystals
    frames: 400, // Build Time 400 frames (25 seconds)
    // `factions/syndicate/tunnel_menus.md > Expansions under construction` (design change 0254): "the ConstructionHP
    // Rule applies to every expansion (starts at 10%, grows with progress)".
    start_hp: 0.1,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::syndicate::tests::assert_expansion;

    /// The stat block (design change 0367): an underground structure, 2×2, MaxHP 400, the expansions' 6 / 24, behind a
    /// Hideout; a Tier 1 Tunnel's for 150 Space Crystals in 400 frames, once slice M6-SB makes it live.
    #[test]
    fn the_vault_follows_the_design() {
        let expansion = ExpansionDef {
            tier: 1,
            cost: 150,
            frames: 400,
            start_hp: 0.1,
        };
        assert_expansion(ObjectKind::Vault, "Vault", 2, 400.0, &[ObjectKind::Hideout]);
        assert_eq!(VAULT_EXPANSION, expansion);
    }
}
