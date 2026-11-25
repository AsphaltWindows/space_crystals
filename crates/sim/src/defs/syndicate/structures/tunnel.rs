//! The Tunnel — `factions/syndicate/tunnels.md > Tunnel` — with its tier table, the rules of the Tunnel Network
//! that its Tunnels make up, and its construction by an Agent.

use crate::defs::structures::{
    BuilderGoes, DropOff, SiteDef, SitePrice, StructureDef, SymmetryType, TierDef, Transit, TunnelDef,
};
use crate::defs::{Armor, Category, Footprint, ObjectDef, ObjectKind};
use crate::player::Faction;
use crate::sides::SideLetter;
use crate::units::Tick;

/// The Tunnel Network's transit delay, ticks, as the Tunnel's def gives it (`TunnelDef::transit_delay`): its name for
/// the tests that wait it out.
pub const TRANSIT_DELAY_TICKS: Tick = TUNNEL_DEF.transit_delay;

/// `factions/syndicate/tunnels.md > Tunnel > Tiers`, its table column by column, and `> Tunnel Network`'s numbers.
pub(crate) const TUNNEL_DEF: TunnelDef = TunnelDef {
    tiers: &[
        // Tier 1 (base): HP 600, Tunnel Area Radius 2 (8x8), Transit Infantry.
        TierDef {
            max_hp: 600.0,
            area_radius: 2,
            transit: &[Transit::Infantry],
        },
        // Tier 2: HP 800, Tunnel Area Radius 3 (10x10), Transit Infantry + Vehicles.
        TierDef {
            max_hp: 800.0,
            area_radius: 3,
            transit: &[Transit::Infantry, Transit::Vehicles],
        },
        // Tier 3: HP 1000, Tunnel Area Radius 4 (12x12), Transit Infantry + Vehicles + Air.
        TierDef {
            max_hp: 1000.0,
            area_radius: 4,
            transit: &[Transit::Infantry, Transit::Vehicles, Transit::Air],
        },
    ],
    // `> Tunnel Network > Transit delay` (design change 0245): "after a unit **enters** the network, there is a
    // **small delay before it can exit**: **2 seconds (32 frames), placeholder**". Flat, "not by distance".
    transit_delay: 32,
    // `> Tunnel Network > Regeneration inside` (design change 0249): "every unit inside the Tunnel Network slowly
    // regenerates HP: **1% of its MaxHP per second, placeholder**".
    regen_per_second: 0.01,
};

/// The Tunnel's construction on its site by an Agent (`StructureDef::site`, slice M4-E; `> Constructed by - Agent`).
pub(crate) const TUNNEL_SITE: SiteDef = SiteDef {
    // `factions/syndicate/tunnels.md > Cost Scaling > Construction Cost`: "Cost = current Tunnels owned (all tiers
    // count), in Supplies"; "Lingering Tunnel ghosts count as Tunnels owned [...] (likewise a Tunnel under construction
    // counts)" (design change 0233).
    price: SitePrice::SuppliesPerOwned,
    // `factions/syndicate/tunnels.md > Cost Scaling > Construction Time`: "480 frames (30 seconds)".
    frames: 480,
    // `factions/syndicate/units/agent.md > Building > Tunnel Construction Flow`: "The Tunnel starts at 10% HP
    // (ConstructionHP Rule)".
    start_hp: 0.1,
    // `factions/syndicate/units/agent.md > Tunnel Construction Flow`: "If construction completes: The Tunnel becomes
    // operational and the Agent is inside the Tunnel Network".
    builder_goes: BuilderGoes::Network,
};

/// `factions/syndicate/tunnels.md > Tunnel`: one type with a tier (plan decision T39; `crate::tunnel`). It blocks,
/// sees and can be destroyed (slice M4-A), and takes the Agents' Space Crystals at its Side B hatch (slice M4-D). An
/// Agent builds it (`> Constructed by - Agent`, slice M4-E: [`TUNNEL_SITE`]). The network, the Supplies at Side D and
/// its upgrades come with slices M4-C to M4-I.
pub(crate) static TUNNEL: ObjectDef = ObjectDef {
    kind: ObjectKind::Tunnel,
    name: "Tunnel",
    faction: Some(Faction::Syndicate),          // `factions/syndicate/`
    category: Category::Structure,              // Entity Type - Structure Type
    footprint: Footprint::Tiles { w: 4, h: 4 }, // Size - 4x4
    selectable: true,                           // `core/entities.md > Object Type > Selectable - true`
    destructible: true,                         // Destructible - true
    groupable: false,                           // Groupable - false
    sight_range: 5.0,                           // SightRange - 5
    // Tiers: HP 600 at Tier 1, the tier a Tunnel starts at; `Object::max_hp` reads its tier's (`crate::tunnel`).
    max_hp: Some(600.0),
    max_shields: None,
    armor: Armor {
        point: 1.0, // Tiers: PointArmor 1 (every tier)
        full: 16.0, // Tiers: FullArmor 16 (every tier)
    },
    // A surface structure (`core/map.md > Elevation > Structures only on flat terrain`): only the expansions may stand
    // on slopes (design change 0005).
    needs_flat_ground: true,
    unit: None,
    structure: Some(StructureDef {
        // SymmetryType - ABCD, clockwise from the top: A the entrance and exit, B the crystal hatch, C the back wall
        // (opposite A, no function), D the Supplies hatch (design change 0248, correcting 0224). Of Side A's two
        // paths, the exit is the half nearer B and the entry the half nearer D (0248, for slice M4-C).
        symmetry: SymmetryType::ABCD,
        build_radius_extension: 0,
        power: 0,
        seeds_build_area: false,
        production: None,
        mining: None,
        onto: None,
        constructs: &[],
        warp_in: &[],
        // Agents deliver to its hatches, crystals at Side B and Supplies at Side D (design change 0248), each "1 grid
        // square, near the middle of the side" (0255 point 6; `economy::drop_off`), one Agent at a time (plan
        // decision T34, with the side rule in its def). Supplies from slice M4-I on.
        drop_off: Some(DropOff::Hatches {
            crystals: SideLetter::B,
            supplies: SideLetter::D,
        }),
        underground: false,
        tunnel: Some(TUNNEL_DEF),
        site: Some(TUNNEL_SITE),
        expansion: None,
        requires: &[], // TechPrerequisite - None
    }),
    default_settings: None,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::UnitBase;

    /// A 4×4 ABCD surface structure, Destructible, Groupable false, SightRange 5, whose tier table is `> Tiers`: HP
    /// 600 / 800 / 1000, PointArmor 1 and FullArmor 16 at every tier, Tunnel Area Radius 2 / 3 / 4, Transit Infantry,
    /// then Vehicles, then Air; with its two hatches.
    #[test]
    fn the_tunnel_follows_the_design() {
        let def = ObjectKind::Tunnel.def();
        let structure = def.structure.as_ref().unwrap();
        assert_eq!(
            (def.category, def.faction, def.tile_size(), structure.symmetry),
            (
                Category::Structure,
                Some(Faction::Syndicate),
                Some((4, 4)),
                SymmetryType::ABCD
            )
        );
        let armor = Armor { point: 1.0, full: 16.0 };
        assert_eq!(
            (def.max_hp, def.max_shields, def.armor, def.sight_range),
            (Some(600.0), None, armor, 5.0)
        );
        assert_eq!(
            (def.selectable, def.destructible, def.groupable, def.needs_flat_ground),
            (true, true, false, true)
        );
        let tiers: Vec<(f32, u32, &[Transit])> = (structure.tunnel.unwrap().tiers.iter())
            .map(|tier| (tier.max_hp, tier.area_radius, tier.transit))
            .collect();
        use Transit::{Air, Infantry, Vehicles};
        assert_eq!(
            tiers,
            [
                (600.0, 2, &[Infantry][..]),
                (800.0, 3, &[Infantry, Vehicles][..]),
                (1000.0, 4, &[Infantry, Vehicles, Air][..]),
            ]
        );
        // `> Tunnel Network`: the transit delay, "2 seconds (32 frames)", and the regeneration, "1% of its MaxHP per
        // second" (design changes 0245 and 0249).
        let network = structure.tunnel.unwrap();
        assert_eq!((network.transit_delay, network.regen_per_second), (32, 0.01));
        // Design change 0248: crystals at the hatch on Side B, Supplies at the one on Side D.
        let expected = StructureDef {
            symmetry: SymmetryType::ABCD,
            build_radius_extension: 0,
            seeds_build_area: false,
            power: 0,
            production: None,
            mining: None,
            onto: None,
            constructs: &[],
            warp_in: &[],
            drop_off: Some(DropOff::Hatches {
                crystals: SideLetter::B,
                supplies: SideLetter::D,
            }),
            underground: false,
            tunnel: structure.tunnel,
            site: structure.site,
            expansion: None,
            requires: &[],
        };
        assert_eq!(structure, &expected);
    }

    /// `factions/syndicate/tunnels.md > Cost Scaling`: its price counts the Tunnels owned, in Supplies, and its
    /// construction takes "480 frames (30 seconds)"; `factions/syndicate/units/agent.md > Building > Tunnel
    /// Construction Flow`: it "starts at 10% HP (ConstructionHP Rule)", and the Agent ends up "inside the Tunnel
    /// Network".
    #[test]
    fn an_agent_builds_it_in_480_frames_for_a_supply_per_tunnel() {
        let site = ObjectKind::Tunnel.def().structure.as_ref().unwrap().site.unwrap();
        assert_eq!(
            site,
            SiteDef {
                price: SitePrice::SuppliesPerOwned,
                frames: 480,
                start_hp: 0.1,
                builder_goes: BuilderGoes::Network,
            }
        );
        let agent = ObjectKind::Agent.def().unit.as_ref().unwrap().worker.unwrap();
        assert_eq!(agent.builds, [ObjectKind::Tunnel]);
    }

    /// `factions/syndicate/tunnels.md > Tunnel Network > Transit Tier Requirements`: "Tier 1+: Infantry (Light
    /// Infantry, Heavy Infantry)", "Tier 2+: Vehicles (Wheeled, Tracked, Drill, Hover Vehicle, Mech)", "Tier 3+: Air
    /// units (Hover Craft, Glider)".
    #[test]
    fn each_tier_admits_its_unit_bases() {
        use Transit::{Air, Infantry, Vehicles};
        use UnitBase as B;
        let tiers = TUNNEL_DEF.tiers;
        let first_tier = [
            (B::LightInfantry, Infantry, 1),
            (B::HeavyInfantry, Infantry, 1),
            (B::WheeledVehicle, Vehicles, 2),
            (B::TrackedVehicle, Vehicles, 2),
            (B::DrillUnit, Vehicles, 2),
            (B::HoverVehicle, Vehicles, 2),
            (B::Mech, Vehicles, 2),
            (B::HoverCraft, Air, 3),
            (B::Glider, Air, 3),
        ];
        for (base, transit, first) in first_tier {
            assert_eq!(Transit::of(base), transit, "{base:?}");
            for (tier, row) in (1..).zip(tiers) {
                assert_eq!(row.admits(base), tier >= first, "{base:?} at Tier {tier}");
            }
        }
    }
}
