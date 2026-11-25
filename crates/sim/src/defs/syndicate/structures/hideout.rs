//! The Hideout — `factions/syndicate/structures/hideout.md`.

use crate::defs::structures::{ExitRule, ExpansionDef, ProductionDef, Recipe, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/syndicate/structures/hideout.md`: an underground expansion (plan decision T32): hidden from the other
/// players while owned, it is walked over, and can be destroyed. It produces into its owner's Tunnel Network (slice
/// M4-C, `crate::network`). A Tunnel builds it in its Tunnel Area ([`HIDEOUT_EXPANSION`], slice M4-F).
pub(crate) static HIDEOUT: ObjectDef = ObjectDef {
    kind: ObjectKind::Hideout,
    name: "Hideout",
    faction: Some(Faction::Syndicate),          // `factions/syndicate/`
    category: Category::Structure,              // Entity Type - Structure Type (Underground)
    footprint: Footprint::Tiles { w: 2, h: 2 }, // Size - 2x2
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    // Not listed for the Hideout; a type with MaxHP is destructible (`core/entities.md > Object Instance > HP`).
    destructible: true,
    // `factions/syndicate/tunnels.md > Tunnel ObjectInterfaceState > Expansions under construction` (design change
    // 0254, answering Q0065 points 5 and 6): "Sight: none (underground). Groupable: true."
    groupable: true,
    sight_range: 0.0,
    max_hp: Some(400.0), // HP - 400
    max_shields: None,
    // `factions/syndicate/structures/README.md > Tunnel Expansions > High armor underground` (design changes 0262 and
    // 0264; the stat block's since design change 0284): "Placeholder for every expansion: n = 6, i.e. PointArmor 6,
    // FullArmor 6 x size in grid squares (a 2x2 expansion: 6 / 24)".
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
        // `> Produces`, into the Tunnel Network (`factions/syndicate/structures/README.md > Tunnel Expansions > Rally
        // Point Behavior`). The Guard joins the Agent with its prerequisite, the Optics Lab (slice M4-K); the
        // Grenadier with its numbers.
        production: Some(ProductionDef {
            exits: ExitRule::Network,
            recipes: &[Recipe {
                kind: ObjectKind::Agent,
                cost: 100,   // "Agent: 100 Space Crystals"
                frames: 240, // "240 frames (15 seconds)"
            }],
        }),
        mining: None,
        onto: None,
        constructs: &[],
        warp_in: &[],
        drop_off: None,
        underground: true, // Entity Type - Structure Type (Underground)
        tunnel: None,
        site: None,
        expansion: Some(HIDEOUT_EXPANSION),
    }),
    default_settings: None,
};

/// The Hideout's construction by a Tunnel (`StructureDef::expansion`, slice M4-F): "Additional Hideouts can be
/// constructed in any Tier 1+ Tunnel".
pub(crate) const HIDEOUT_EXPANSION: ExpansionDef = ExpansionDef {
    tier: 1,     // Tier Requirement - 1
    cost: 200,   // Cost - 200 Space Crystals
    frames: 400, // Build Time - 400 frames (25 seconds)
    // `factions/syndicate/tunnels.md > Expansions under construction` (design change 0254): "the ConstructionHP Rule
    // applies to every expansion (starts at 10%, grows with progress)".
    start_hp: 0.1,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// An underground structure ("Structure Type (Underground)"), 2×2, HP 400, which may stand on slopes (design
    /// change 0005) and has no tiers; an expansion's high armor, PointArmor 6 and FullArmor 24
    /// (`factions/syndicate/structures/README.md > Tunnel Expansions > High armor underground`, placeholders of design
    /// changes 0262 and 0264); no sight and Groupable (`factions/syndicate/tunnels.md > Expansions under
    /// construction`, design change 0254 answering Q0065 points 5 and 6). Not given, so a placeholder: SymmetryType
    /// AAAA (plan decision T29).
    #[test]
    fn the_hideout_follows_the_design() {
        let def = ObjectKind::Hideout.def();
        let structure = def.structure.as_ref().unwrap();
        assert_eq!(
            (def.category, def.faction, def.tile_size(), structure.symmetry),
            (
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
            (structure.underground, structure.tunnel, structure.drop_off.is_some()),
            (true, None, false)
        );
        // `> Produces`: "Agent: 100 Space Crystals, 240 frames (15 seconds)", into the Tunnel Network (slice M4-C);
        // the Guard comes with its prerequisite (slice M4-K).
        let production = structure.production.expect("the Hideout produces");
        assert_eq!(production.exits, ExitRule::Network);
        let recipes: Vec<(ObjectKind, u32, u32)> = (production.recipes.iter())
            .map(|recipe| (recipe.kind, recipe.cost, recipe.frames))
            .collect();
        assert_eq!(recipes, [(ObjectKind::Agent, 100, 240)]);
    }

    /// "Tier Requirement - 1", "Cost - 200 Space Crystals", "Build Time - 400 frames (25 seconds)", built by a Tunnel
    /// from 10% of its HP (`factions/syndicate/tunnels.md > Expansions under construction`, design change 0254).
    #[test]
    fn a_tunnel_builds_it_for_200_crystals_in_400_frames() {
        let structure = ObjectKind::Hideout.def().structure.as_ref().unwrap();
        let expected = ExpansionDef {
            tier: 1,
            cost: 200,
            frames: 400,
            start_hp: 0.1,
        };
        assert_eq!((structure.expansion, structure.site), (Some(expected), None));
    }
}
