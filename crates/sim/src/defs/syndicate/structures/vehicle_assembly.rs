//! The Vehicle Assembly — `factions/syndicate/structures/vehicle_assembly.md` (design change 0367's placeholders; slice
//! M6-SK: its numbers; slice M6-SV: a Tier 2 Tunnel builds it, and it produces the Assault Bike; slice M6-SB: the
//! Shadebringer).

use crate::defs::structures::{ExitRule, ExpansionDef, ProductionDef, Recipe, StructureDef, SymmetryType};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;

/// `factions/syndicate/structures/vehicle_assembly.md`: the "Syndicate production building that assembles Syndicate
/// ground vehicles", an underground expansion behind a Chemical Plant, and the Forge's prerequisite. Its numbers came
/// with slice M6-SK (plan decision T84, by T69's rule); slice M6-SV makes it live: a Tier 2 Tunnel builds it
/// ([`VEHICLE_ASSEMBLY_EXPANSION`], its ExpandMenu entry its kind), and its production is the Hideout's
/// (`ExitRule::Network`, plan decision T100: into the Tunnel Network, out of a Tier 2+ Tunnel's Side A), the Assault
/// Bike its first recipe and (slice M6-SB) the Shadebringer its second; each other recipe goes live with its unit's
/// slice (M6-SL, M6-ST1), in its panel slot.
/// Its numbers are the design's "Placeholder stats".
pub(crate) const VEHICLE_ASSEMBLY: ObjectDef = ObjectDef {
    kind: ObjectKind::VehicleAssembly,
    name: "Vehicle Assembly",
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
        // `> Placeholder stats`: "Panel as the Hideout": the Hideout's production, into the Tunnel Network
        // (`factions/syndicate/structures/README.md > Tunnel Expansions > Rally Point Behavior`), its recipes the
        // consts below, in the panel's order, each live with its unit's slice: the Assault Bike (slice M6-SV) and the
        // Shadebringer (slice M6-SB).
        production: Some(ProductionDef {
            exits: ExitRule::Network,
            recipes: &[ASSAULT_BIKE_RECIPE, SHADEBRINGER_RECIPE],
            recruitment: None,
            augment: &[],
            default_rally: None,
        }),
        underground: true, // Entity Type - Structure Type (Underground)
        expansion: Some(VEHICLE_ASSEMBLY_EXPANSION),
        // TechPrerequisite - player owns at least one Chemical Plant (anywhere in the Tunnel Network)
        requires: &[ObjectKind::ChemicalPlant],
        ..StructureDef::NONE
    }),
    ..ObjectDef::NONE
};

/// The Vehicle Assembly's construction by a Tunnel (`StructureDef::expansion`, slice M6-SV): "Tier Requirement - 2".
pub(crate) const VEHICLE_ASSEMBLY_EXPANSION: ExpansionDef = ExpansionDef {
    tier: 2,     // Tier Requirement - 2
    cost: 250,   // Cost 250 Space Crystals
    frames: 480, // Build Time 480 frames (30 seconds)
    // `factions/syndicate/tunnel_menus.md > Expansions under construction` (design change 0254): "the ConstructionHP
    // Rule applies to every expansion (starts at 10%, grows with progress)".
    start_hp: 0.1,
};

// `> Placeholder stats`: "Produces (cost, time, TunnelSpaceCost): **Assault Bike** 125 SC, 240 f, 2; **Shadebringer** 150
// SC, 320 f, 2; **Laser Battery** 200 SC, 400 f, 3; **Subterranean Transport** 200 SC, 400 f, 3", each behind its unit's
// TechPrerequisite ("**Shadebringer** (requires a Vault), **Laser Battery** (requires an Optics Lab), **Subterranean
// Transport** (requires a Forge)"); the TunnelSpaceCosts are the units' own (`UnitDef::tunnel_space_cost`). Its panel
// is "as the Hideout: **Q** Assault Bike, **W** Shadebringer, **E** Laser Battery, **R** Subterranean Transport".

/// Q: the Assault Bike (slice M6-SV).
pub(crate) const ASSAULT_BIKE_RECIPE: Recipe = Recipe {
    kind: ObjectKind::AssaultBike,
    cost: 125,
    frames: 240,
    requires: &[],
    supplies: 0,
};

/// W: the Shadebringer, behind a Vault (slice M6-SB).
pub(crate) const SHADEBRINGER_RECIPE: Recipe = Recipe {
    kind: ObjectKind::Shadebringer,
    cost: 150,
    frames: 320,
    requires: &[ObjectKind::Vault],
    supplies: 0,
};

/// E: the Laser Battery, behind an Optics Lab, for slice M6-SL.
#[cfg_attr(not(test), expect(dead_code, reason = "slice M6-SL: the Vehicle Assembly's E"))]
pub(crate) const LASER_BATTERY_RECIPE: Recipe = Recipe {
    kind: ObjectKind::LaserBattery,
    cost: 200,
    frames: 400,
    requires: &[ObjectKind::OpticsLab],
    supplies: 0,
};

/// R: the Subterranean Transport, behind a Forge, for slice M6-ST1.
#[cfg_attr(not(test), expect(dead_code, reason = "slice M6-ST1: the Vehicle Assembly's R"))]
pub(crate) const SUBTERRANEAN_TRANSPORT_RECIPE: Recipe = Recipe {
    kind: ObjectKind::SubterraneanTransport,
    cost: 200,
    frames: 400,
    requires: &[ObjectKind::Forge],
    supplies: 0,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::syndicate::tests::assert_expansion;

    /// The stat block (design change 0367): an underground structure, 3×3, MaxHP 600, the expansions' 6 / 54, behind a
    /// Chemical Plant; a Tier 2 Tunnel's for 250 Space Crystals in 480 frames, from 10% of its MaxHP. Its four recipes,
    /// in its panel's order, each with its unit's TechPrerequisite; live (slice M6-SV), its production the Hideout's
    /// into the Tunnel Network, with the Assault Bike's and (slice M6-SB) the Shadebringer's, the others each its unit's
    /// slice's.
    #[test]
    fn the_vehicle_assembly_follows_the_design() {
        let expansion = ExpansionDef {
            tier: 2,
            cost: 250,
            frames: 480,
            start_hp: 0.1,
        };
        let kind = ObjectKind::VehicleAssembly;
        assert_expansion(kind, "Vehicle Assembly", 3, 600.0, &[ObjectKind::ChemicalPlant]);
        assert_eq!(VEHICLE_ASSEMBLY_EXPANSION, expansion);
        let structure = kind.def().structure.as_ref().unwrap();
        assert_eq!(structure.expansion, Some(expansion), "a Tier 2 Tunnel builds it");
        let production = ProductionDef {
            exits: ExitRule::Network,
            recipes: &[ASSAULT_BIKE_RECIPE, SHADEBRINGER_RECIPE],
            recruitment: None,
            augment: &[],
            default_rally: None,
        };
        assert_eq!(
            structure.production,
            Some(production),
            "the Hideout's production, into the network; the Assault Bike's and the Shadebringer's recipes so far"
        );
        let recipes = [
            ASSAULT_BIKE_RECIPE,
            SHADEBRINGER_RECIPE,
            LASER_BATTERY_RECIPE,
            SUBTERRANEAN_TRANSPORT_RECIPE,
        ];
        let rows: Vec<(ObjectKind, u32, u32, &[ObjectKind])> =
            recipes.iter().map(|r| (r.kind, r.cost, r.frames, r.requires)).collect();
        assert_eq!(
            rows,
            [
                (ObjectKind::AssaultBike, 125, 240, &[][..]),
                (ObjectKind::Shadebringer, 150, 320, &[ObjectKind::Vault][..]),
                (ObjectKind::LaserBattery, 200, 400, &[ObjectKind::OpticsLab][..]),
                (ObjectKind::SubterraneanTransport, 200, 400, &[ObjectKind::Forge][..]),
            ]
        );
        let tunnel_space = recipes.map(|r| r.kind.def().unit.as_ref().and_then(|unit| unit.tunnel_space_cost));
        assert_eq!(tunnel_space, [Some(2), Some(2), Some(3), Some(3)]);
    }
}
