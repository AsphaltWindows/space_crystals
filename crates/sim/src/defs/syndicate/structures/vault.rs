//! The Vault — `factions/syndicate/structures/vault.md` (design change 0367's placeholders; slice M6-SK, made live by
//! slice M6-SB).

use crate::defs::structures::{ExpansionDef, StructureDef, SymmetryType, VaultDef};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/syndicate/structures/vault.md`: the Syndicate's "equipment storage warehouse for re-arming Syndicate
/// forces", an underground expansion, the Shadebringer's prerequisite and, with a Chemical Plant, the Specter's. Slice
/// M6-SK registered its numbers; since slice M6-SB a Tier 1 Tunnel builds it ([`VAULT_EXPANSION`], its ExpandMenu
/// entry a `Kind`), and it refills the tanks of the units in its owner's network, two units at a time
/// ([`VAULT_SERVICE`], `network::services`; "a Vault re-arms and recharges units **anywhere in the Tunnel
/// Network**"). Its numbers are the design's "Placeholder stats".
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
        expansion: Some(VAULT_EXPANSION),
        // TechPrerequisite - player owns at least one Hideout (anywhere in the Tunnel Network)
        requires: &[ObjectKind::Hideout],
        vault: Some(VAULT_SERVICE),
        ..StructureDef::NONE
    }),
    ..ObjectDef::NONE
};

/// The Vault's construction by a Tunnel (`StructureDef::expansion`), live since slice M6-SB.
pub(crate) const VAULT_EXPANSION: ExpansionDef = ExpansionDef {
    tier: 1,     // Tier Requirement - 1
    cost: 150,   // Cost 150 Space Crystals
    frames: 400, // Build Time 400 frames (25 seconds)
    // `factions/syndicate/tunnel_menus.md > Expansions under construction` (design change 0254): "the ConstructionHP
    // Rule applies to every expansion (starts at 10%, grows with progress)".
    start_hp: 0.1,
};

/// (M6-SB) Its service of the Tunnel Network (`StructureDef::vault`, plan decision T98, `network::services`):
/// `> Placeholder stats`, "**Throughput**: each Vault re-arms or recharges **2 units at a time**; others wait their
/// turn (network-wide, first come first served)".
pub(crate) const VAULT_SERVICE: VaultDef = VaultDef { slots: 2 };

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::syndicate::tests::assert_expansion;

    /// The stat block (design change 0367): an underground structure, 2×2, MaxHP 400, the expansions' 6 / 24, behind a
    /// Hideout; a Tier 1 Tunnel's for 150 Space Crystals in 400 frames (slice M6-SB); serving 2 units at a time, the
    /// one type with a Vault's service.
    #[test]
    fn the_vault_follows_the_design() {
        let expansion = ExpansionDef {
            tier: 1,
            cost: 150,
            frames: 400,
            start_hp: 0.1,
        };
        assert_expansion(ObjectKind::Vault, "Vault", 2, 400.0, &[ObjectKind::Hideout]);
        let structure = ObjectKind::Vault.def().structure.as_ref().unwrap();
        assert_eq!((structure.expansion, VAULT_EXPANSION), (Some(expansion), expansion));
        assert_eq!(structure.vault, Some(VaultDef { slots: 2 }));
        let vaults: Vec<ObjectKind> = (ObjectKind::ALL.iter().copied())
            .filter(|kind| kind.def().structure.as_ref().is_some_and(|s| s.vault.is_some()))
            .collect();
        assert_eq!(vaults, [ObjectKind::Vault]);
    }
}
