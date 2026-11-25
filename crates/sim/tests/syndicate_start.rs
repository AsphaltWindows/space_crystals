//! A Syndicate player's start (slice M4-B; `factions/syndicate/tunnels.md > Tunnel > Starting Conditions`): the banks,
//! the objects of the Starting Layout on `dev_skirmish` in either slot, and Tunnel Space
//! (`factions/syndicate/overview.md > SyndicateResources > Tunnel Space`, `tunnels.md > Tunnel > Tunnel Space`),
//! derived from the Tunnel and the Hideout; `dev_skirmish` with every faction in either slot; and the layout's
//! Tunnel Area rules (design change 0242) as the map's validation applies them. The Starting Conditions table is
//! unit-tested in `scenario/starting.rs`, the derivation of Tunnel Space in `economy/army/tunnel_space.rs`, the
//! placement rules in `placement/tests/tunnel_area.rs`, the bar and the camera in the client.

use sc_sim::nav::TileRect;
use sc_sim::scenario::DEFAULT_SEED;
use sc_sim::sides::{SideLetter, sides};
use sc_sim::testing::TestSim;
use sc_sim::tunnel::{self, Resource};
use sc_sim::{
    ArmyCap, ArmyResource, Banks, DebugCommand, Faction, GameSetup, MapDef, Object, ObjectKind, PlacementCheck,
    PlayerId, SU_PER_GU, Scenario, ScenarioProblem, Sim, SlotId, SpawnAt, TilePos, Vec2, WholeProblem, scenarios,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const TUNNEL: ObjectKind = ObjectKind::Tunnel;
const HIDEOUT: ObjectKind = ObjectKind::Hideout;
const AGENT: ObjectKind = ObjectKind::Agent;
const SYNDICATE: Faction = Faction::Syndicate;

/// A point in grid units.
fn point(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y) * SU_PER_GU
}

/// `dev_skirmish` with player 1 in slot 1 and player 2 in slot 2, playing `factions`.
fn dev_skirmish_as(factions: [Faction; 2]) -> Scenario {
    let map = scenarios::dev_skirmish_map();
    let setup = GameSetup::per_slot(&map, &factions, DEFAULT_SEED);
    Scenario { map, setup }
}

/// `player`'s objects: kind and position, in id order.
fn objects_of(sim: &Sim, player: PlayerId) -> Vec<(ObjectKind, Vec2)> {
    sim.objects()
        .filter(|o| o.owner == Some(player))
        .map(|o| (o.kind, o.pos))
        .collect()
}

/// The first object of `kind` that `player` owns.
fn find(sim: &Sim, player: PlayerId, kind: ObjectKind) -> &Object {
    let found = sim.objects().find(|o| o.owner == Some(player) && o.kind == kind);
    found.expect("the player owns one")
}

fn footprint(o: &Object) -> TileRect {
    let p = o.placement.expect("a structure");
    TileRect::new(p.origin, p.w, p.h)
}

fn tunnel_space(used: u32, available: u32) -> Option<ArmyCap> {
    Some(ArmyCap {
        resource: ArmyResource::TunnelSpace,
        used,
        available,
    })
}

/// The acceptance of slice M4-B, start (`tunnels.md > Tunnel > Starting Conditions`: "One pre-built **Tier 1
/// Tunnel** containing one pre-built **Hideout**", "One **Agent**", "**1 Supply**", "Starting Space Crystals:
/// **100**"): in either slot of `dev_skirmish`, a Syndicate player starts with the Tier 1 Tunnel, the Hideout inside its
/// Tunnel Area and the Agent where the layout puts them (slot 2's being slot 1's 180° image); 100 Space Crystals and
/// 1 Supply; Tunnel Space 1/11 (48 tunnel-region cells less the Hideout's 4 is 44, over 4 is 11; the Agent's
/// TunnelSpaceCost is 1); and the camera starts on the Tunnel (`core/camera.md > Starting Position`: "Syndicate:
/// starting Tunnel"). The GDO beside them starts as ever.
#[test]
fn a_syndicate_player_starts_with_their_starting_conditions() {
    let banks = Banks {
        crystals: 100,
        supplies: 1,
        ..Banks::default()
    };
    let slot_1 = [
        (TUNNEL, point(6.0, 6.0)),
        (HIDEOUT, point(9.0, 9.0)),
        (AGENT, point(6.0, 9.5)),
    ];
    for (factions, player, image) in [
        ([SYNDICATE, Faction::Gdo], P1, false),
        ([Faction::Gdo, SYNDICATE], P2, true),
    ] {
        let scenario = dev_skirmish_as(factions);
        let sim = Sim::new(&scenario);
        let at = |(kind, pos): (ObjectKind, Vec2)| (kind, if image { point(64.0, 48.0) - pos } else { pos });
        assert_eq!(objects_of(&sim, player), slot_1.map(at), "{player:?}");
        let tunnel = find(&sim, player, TUNNEL);
        assert_eq!(tunnel.tunnel.as_ref().map(|t| t.tier), Some(1));
        let area = tunnel::area::of(tunnel).expect("a Tunnel");
        assert_eq!((area.w, area.h), (8, 8));
        assert!(tunnel::area::encloses(area, footprint(find(&sim, player, HIDEOUT))));
        assert_eq!(sim.player(player).unwrap().banks(), banks, "{player:?}");
        assert_eq!(sim.army_cap(player), tunnel_space(1, 11), "{player:?}");
        assert_eq!(scenario.primary_structure(player), Some(at(slot_1[0]).1), "the Tunnel");
        let gdo = if image { P1 } else { P2 };
        assert_eq!(
            sim.army_cap(gdo).map(|c| (c.resource, c.used, c.available)),
            Some((ArmyResource::UnitControl, 0, 200))
        );
    }
}

/// The layout's geometry (slice M4-B; design changes 0248, 0255, 0259): in either slot, the Tunnel's Side A (the
/// entrance) faces the open map, south in slot 1, and its Side D (the Supplies hatch) the middle of the map, east in
/// slot 1; its crystal hatch, on Side B (`tunnel::hatch_square`, slice M4-D: the middle square farther from Side A),
/// lies 1 grid unit edge to edge from a patch of the field (0259: the patch "1 GU from the hatch"), across a free
/// column. Q0076: with patches that close, 3 or more Agents jam at the hatch; the layout keeps 0259's 1 GU until the
/// design decides.
#[test]
fn the_crystal_hatch_faces_the_field_one_grid_unit_away() {
    let sim = Sim::new(&dev_skirmish_as([SYNDICATE, SYNDICATE]));
    let patches: Vec<TilePos> = sim
        .objects()
        .filter(|o| o.kind == ObjectKind::CrystalPatch)
        .map(|o| o.placement.unwrap().origin)
        .collect();
    for (player, south, hatch) in [(P1, (0, 1), TilePos::new(4, 5)), (P2, (0, -1), TilePos::new(59, 42))] {
        let tunnel = find(&sim, player, TUNNEL);
        let letters = sides(TUNNEL, tunnel.placement.unwrap().orientation()).unwrap();
        // North, east, south and west: C, D, A, B in slot 1; turned 180° in slot 2.
        let expected = if south == (0, 1) {
            [SideLetter::C, SideLetter::D, SideLetter::A, SideLetter::B]
        } else {
            [SideLetter::A, SideLetter::B, SideLetter::C, SideLetter::D]
        };
        assert_eq!(letters, expected, "{player:?}");
        assert_eq!(
            tunnel::hatch_square(tunnel, Resource::Crystals),
            Some(hatch),
            "{player:?}"
        );
        let gap = |p: &TilePos| ((p.x - hatch.x).abs() - 1, (p.y - hatch.y).abs());
        assert!(
            patches.iter().any(|p| gap(p) == (1, 0)),
            "{player:?}: a patch 1 GU west or east of the hatch"
        );
    }
}

/// The acceptance of slice M4-B, validation: `dev_skirmish` validates with the GDO, the Colonists and the Syndicate
/// in each of the nine slot combinations, and a game starts on it; no layout is outdated, and its file is the
/// built-in map. An incomplete Syndicate layout lacks its kinds (`ScenarioProblem::LayoutMissing`) in both slots, and
/// no game starts on the map, a GDO one included (the M3 Colonists test, `colonists_start.rs`, for the Syndicate).
#[test]
fn dev_skirmish_validates_with_every_faction_in_either_slot() {
    let factions = [Faction::Gdo, Faction::Colonists, SYNDICATE];
    for a in factions {
        for b in factions {
            let scenario = dev_skirmish_as([a, b]);
            assert_eq!(scenario.validate(), [], "{a:?}, {b:?}");
            assert!(Sim::try_new(&scenario).is_ok(), "{a:?}, {b:?}");
        }
    }
    assert_eq!(scenarios::dev_skirmish_map().outdated_layouts(), []);
    let file: MapDef = serde_json::from_str(include_str!("../../../maps/dev_skirmish.json")).unwrap();
    assert_eq!(
        file,
        scenarios::dev_skirmish_map(),
        "maps/dev_skirmish.json is the built-in map"
    );

    let mut map = scenarios::dev_skirmish_map();
    map.slots[0].layouts.get_mut(SYNDICATE).remove(1); // the Hideout
    map.regenerate();
    let lacks = |slot| ScenarioProblem::LayoutMissing {
        slot: SlotId(slot),
        faction: SYNDICATE,
        kind: HIDEOUT,
        missing: 1,
    };
    let expected = [lacks(1), lacks(2)];
    assert_eq!(map.validate(), expected);
    assert_eq!(map.outdated_layouts(), [(SlotId(1), SYNDICATE), (SlotId(2), SYNDICATE)]);
    assert_eq!(Sim::try_new(&Scenario::new(map)).err(), Some(expected.to_vec()));
}

/// Design change 0242 ("Starting Layouts must respect this"): `dev_skirmish`'s starting Tunnel moved to the lake's
/// shore, (18, 6) to (21, 9), its area taking in the Water from x = 22, or to (12, 1), 1 tile from the north edge,
/// is refused as a whole by the map's validation, in slot 1 and in its image, and no game starts. 2 tiles from the
/// edge it is valid (Placeholder: Q0078), the Hideout following it into the area.
#[test]
fn a_layout_tunnel_by_the_lake_or_the_edge_is_refused() {
    let moved = |tunnel: (i32, i32), hideout: (i32, i32)| {
        let mut map = scenarios::dev_skirmish_map();
        let layout = map.slots[0].layouts.get_mut(SYNDICATE);
        layout[0].at = SpawnAt::Tile(TilePos::new(tunnel.0, tunnel.1));
        layout[1].at = SpawnAt::Tile(TilePos::new(hideout.0, hideout.1));
        map.regenerate();
        map
    };
    let refused = |slot, problem| ScenarioProblem::LayoutPlacement {
        slot: SlotId(slot),
        faction: SYNDICATE,
        index: 0,
        kind: TUNNEL,
        check: PlacementCheck {
            tiles: Vec::new(),
            whole: vec![problem],
        },
    };
    let lake = moved((18, 6), (16, 8));
    let expected = [
        refused(1, WholeProblem::TunnelAreaNotDrillable),
        refused(2, WholeProblem::TunnelAreaNotDrillable),
    ];
    assert_eq!(lake.validate(), expected);
    assert!(Sim::try_new(&Scenario::new(lake)).is_err());
    let edge = moved((12, 1), (14, 5));
    let expected = [
        refused(1, WholeProblem::TunnelAreaOffMap),
        refused(2, WholeProblem::TunnelAreaOffMap),
    ];
    assert_eq!(edge.validate(), expected);
    assert_eq!(moved((10, 2), (14, 6)).validate(), []);
}

/// `tunnels.md > Tunnel Space`: "If placing a new expansion ... drops total Tunnel Space below the Syndicate player's
/// currently-committed Tunnel Space usage, existing units are unaffected — usage is simply displayed over capacity
/// (e.g. 12/10)". At 11/11 (the starting Agent and 10 more), a second Hideout spawned into the Tunnel Area takes 4
/// cells: 11/10, and every Agent stays. Used counts the units on the map, whatever they do.
#[test]
fn an_expansion_over_full_tunnel_space_removes_nothing() {
    let mut t = TestSim::new(dev_skirmish_as([SYNDICATE, Faction::Gdo]));
    for i in 0..10 {
        t.spawn_unit_at(1, AGENT, P1, point(12.0 + i as f32, 12.0));
    }
    t.run(1);
    assert_eq!(t.sim().army_cap(P1), tunnel_space(11, 11));
    t.debug_at(
        2,
        DebugCommand::Spawn {
            kind: HIDEOUT,
            owner: Some(P1),
            at: SpawnAt::Tile(TilePos::new(8, 4)),
            orientation: Default::default(),
        },
    );
    t.run(40);
    assert_eq!(t.sim().army_cap(P1), tunnel_space(11, 10));
    let agents = t
        .sim()
        .objects()
        .filter(|o| o.owner == Some(P1) && o.kind == AGENT)
        .count();
    assert_eq!(agents, 11, "nothing is removed");
    let hideouts = t
        .sim()
        .objects()
        .filter(|o| o.owner == Some(P1) && o.kind == HIDEOUT)
        .count();
    assert_eq!(hideouts, 2);
}
