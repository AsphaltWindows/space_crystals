//! The Aerospace Assembly — `factions/syndicate/structures/aerospace_assembly.md` (design change 0367's placeholders;
//! slice M6-SK).

use crate::defs::structures::{ExpansionDef, Recipe, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/syndicate/structures/aerospace_assembly.md`: the "Syndicate production building that assembles Syndicate
/// aviation and orbital forces", an underground expansion behind a Forge, and the Orbital Command's prerequisite. Its
/// numbers only (slice M6-SK, plan decision T84, by T69's rule): no Tunnel builds one yet
/// ([`AEROSPACE_ASSEMBLY_EXPANSION`], kept out of its def, so its ExpandMenu entry stays `Planned`), and it trains
/// nothing: its production comes with slice M6-SD, which makes the Phantom's recipe live, the Specter's with slice
/// M6-SA and the Prism's with slice M6-SP. Its numbers are the design's "Placeholder stats".
pub(crate) const AEROSPACE_ASSEMBLY: ObjectDef = ObjectDef {
    kind: ObjectKind::AerospaceAssembly,
    name: "Aerospace Assembly",
    faction: Some(Faction::Syndicate),          // `factions/syndicate/`
    category: Category::Structure,              // Entity Type - Structure Type (Underground)
    footprint: Footprint::Tiles { w: 3, h: 3 }, // Size 3x3
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    // Not listed; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`).
    destructible: true,
    // Not listed: every expansion's, `factions/syndicate/tunnel_menus.md > Expansions under construction` (design change
    // 0254): "Sight: none (underground). **Groupable**: true."
    groupable: true,
    sight_range: 0.0,
    max_hp: Some(600.0),                     // MaxHP 600
    armor: Armor { point: 6.0, full: 54.0 }, // PointArmor 6, FullArmor 54 (underground expansion, n = 6)
    // `core/map.md > Elevation` (design change 0005): "Syndicate underground expansions (inside a Tunnel Area) may be
    // placed on sloped tiles".
    needs_flat_ground: false,
    structure: Some(StructureDef {
        // Not given (plan decision T29): AAAA, since none of its sides has a function.
        symmetry: SymmetryType::AAAA,
        build_radius_extension: 0,
        power: 0,
        // With slice M6-SD: its production, its recipes the consts below, each live with its slice.
        production: None,
        underground: true, // Entity Type - Structure Type (Underground)
        expansion: None,   // with slice M6-SD: [`AEROSPACE_ASSEMBLY_EXPANSION`]
        // TechPrerequisite - player owns at least one Forge (anywhere in the Tunnel Network)
        requires: &[ObjectKind::Forge],
        ..StructureDef::NONE
    }),
    ..ObjectDef::NONE
};

/// The Aerospace Assembly's construction by a Tunnel (`StructureDef::expansion`), for slice M6-SD to make live.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "slice M6-SD: a Tunnel builds the Aerospace Assembly")
)]
pub(crate) const AEROSPACE_ASSEMBLY_EXPANSION: ExpansionDef = ExpansionDef {
    tier: 3,     // Tier Requirement - 3
    cost: 300,   // Cost 300 Space Crystals
    frames: 640, // Build Time 640 frames (40 seconds)
    // `factions/syndicate/tunnel_menus.md > Expansions under construction` (design change 0254): "the ConstructionHP
    // Rule applies to every expansion (starts at 10%, grows with progress)".
    start_hp: 0.1,
};

// `> Placeholder stats`: "Produces (cost, time, TunnelSpaceCost): **Phantom** 175 SC, 320 f, 2; **Specter** 200 SC, 400
// f, 3; **Prism** 250 SC, 480 f, 2", each behind its unit's TechPrerequisite: the Phantom an Optics Lab
// (`factions/syndicate/units/phantom.md`), the Specter a Vault and a Chemical Plant
// (`factions/syndicate/units/specter.md`), the Prism an Orbital Command (`factions/syndicate/units/prism.md`); the
// TunnelSpaceCosts are the units' own (`UnitDef::tunnel_space_cost`). "Panel: **Q** Phantom, **W** Specter, **E**
// Prism".

/// Q: the Phantom, behind an Optics Lab, for slice M6-SD.
#[cfg_attr(not(test), expect(dead_code, reason = "slice M6-SD: the Aerospace Assembly's Q"))]
pub(crate) const PHANTOM_RECIPE: Recipe = Recipe {
    kind: ObjectKind::Phantom,
    cost: 175,
    frames: 320,
    requires: &[ObjectKind::OpticsLab],
    supplies: 0,
};

/// W: the Specter, behind a Vault and a Chemical Plant, for slice M6-SA.
#[cfg_attr(not(test), expect(dead_code, reason = "slice M6-SA: the Aerospace Assembly's W"))]
pub(crate) const SPECTER_RECIPE: Recipe = Recipe {
    kind: ObjectKind::Specter,
    cost: 200,
    frames: 400,
    requires: &[ObjectKind::Vault, ObjectKind::ChemicalPlant],
    supplies: 0,
};

/// E: the Prism, behind an Orbital Command, for slice M6-SP.
#[cfg_attr(not(test), expect(dead_code, reason = "slice M6-SP: the Aerospace Assembly's E"))]
pub(crate) const PRISM_RECIPE: Recipe = Recipe {
    kind: ObjectKind::Prism,
    cost: 250,
    frames: 480,
    requires: &[ObjectKind::OrbitalCommand],
    supplies: 0,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::syndicate::tests::assert_expansion;

    /// The stat block (design change 0367): an underground structure, 3×3, MaxHP 600, the expansions' 6 / 54, behind a
    /// Forge; a Tier 3 Tunnel's for 300 Space Crystals in 640 frames. Its three recipes, in its panel's order, each
    /// with its unit's TechPrerequisite; none live yet.
    #[test]
    fn the_aerospace_assembly_follows_the_design() {
        let expansion = ExpansionDef {
            tier: 3,
            cost: 300,
            frames: 640,
            start_hp: 0.1,
        };
        let kind = ObjectKind::AerospaceAssembly;
        assert_expansion(kind, "Aerospace Assembly", 3, 600.0, &[ObjectKind::Forge]);
        assert_eq!(AEROSPACE_ASSEMBLY_EXPANSION, expansion);
        let recipes = [PHANTOM_RECIPE, SPECTER_RECIPE, PRISM_RECIPE];
        let rows: Vec<(ObjectKind, u32, u32, &[ObjectKind])> =
            recipes.iter().map(|r| (r.kind, r.cost, r.frames, r.requires)).collect();
        let specter = [ObjectKind::Vault, ObjectKind::ChemicalPlant];
        assert_eq!(
            rows,
            [
                (ObjectKind::Phantom, 175, 320, &[ObjectKind::OpticsLab][..]),
                (ObjectKind::Specter, 200, 400, &specter[..]),
                (ObjectKind::Prism, 250, 480, &[ObjectKind::OrbitalCommand][..]),
            ]
        );
        let tunnel_space = recipes.map(|r| r.kind.def().unit.as_ref().and_then(|unit| unit.tunnel_space_cost));
        assert_eq!(tunnel_space, [Some(2), Some(3), Some(2)]);
    }
}
