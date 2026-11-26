//! The Tunnel Area display (slices M4-J2 and M4-J3; `factions/syndicate/tunnel_area_and_space.md > Tunnel Area
//! display`, design changes 0299 and 0300): the selected Tunnels' areas, a Tunnel ghost's rings and their red cells,
//! and the mesh.

use bevy::math::Vec2;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{BuildingOrientation, Command, DebugCommand, Faction, ObjectKind, OrderKind, SpawnAt, TunnelCommand};

use super::rings::{Side, edge_marks, own_areas};
use super::*;

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const TUNNEL: ObjectKind = ObjectKind::Tunnel;

fn rect(x: i32, y: i32, w: u32, h: u32) -> TileRect {
    TileRect::new(TilePos::new(x, y), w, h)
}

fn t(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y)
}

/// A Tunnel's 4 × 4 footprint at `(x, y)`.
fn footprint(x: i32, y: i32) -> TileRect {
    rect(x, y, 4, 4)
}

fn area(tunnel: ObjectId, tier: u8, area: TileRect) -> TunnelArea {
    TunnelArea { tunnel, tier, area }
}

/// The cells of column `x`, from row `y0` to row `y1`, both included.
fn column(x: i32, y0: i32, y1: i32) -> Vec<TilePos> {
    (y0..=y1).map(|y| t(x, y)).collect()
}

/// A flat 48 × 20 map: player 1's Tunnels 1 on tiles (4, 6) to (7, 9) and 2 on (14, 6) to (17, 9), their Guard 3,
/// and player 2's Tunnel 4 on (24, 6) to (27, 9), which player 1's Guard sees. No auto-targeting; 10 Supplies for
/// player 1.
fn field() -> TestSim {
    let map = vec![".".repeat(48); 20].join("\n");
    let tile = |x, y| SpawnAt::Tile(TilePos::new(x, y));
    let mut t = ScenarioBuilder::new(&map)
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Syndicate)
        .spawn(TUNNEL, Some(P1), tile(4, 6))
        .spawn(TUNNEL, Some(P1), tile(14, 6))
        .spawn(ObjectKind::Guard, Some(P1), SpawnAt::Point(Vec2::new(1480.0, 512.0)))
        .spawn(TUNNEL, Some(P2), tile(24, 6))
        .start();
    t.without_auto_targeting();
    let supplies = DebugCommand::SetResources {
        player: P1,
        crystals: 0,
        supplies: 10,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    t.debug_at(1, supplies);
    t
}

/// `factions/syndicate/tunnel_area_and_space.md > Tunnel Area display` (design change 0299): "Selecting one of your
/// Tunnels shows its Tunnel Area", each own Tunnel in the selection at its tier now; "Enemy Tunnel Areas are never
/// shown", seen or not; a Guard has none. An own Tunnel that upgrades shows its current area, not the larger one it
/// reserves (slice M4-G).
#[test]
fn the_selection_shows_its_own_tunnels_areas_only() {
    let mut t = field();
    let (own, other, guard, enemy) = (ObjectId(1), ObjectId(2), ObjectId(3), ObjectId(4));
    let upgrade = Command::Tunnel {
        tunnel: other,
        command: TunnelCommand::Upgrade,
    };
    t.debug_at(2, DebugCommand::SetTier { id: own, tier: 2 })
        .at(2, P1, upgrade)
        .run_until(3);
    let upgrading = t.sim().object(other).and_then(|o| o.tunnel.as_ref()).expect("a Tunnel");
    assert_eq!((upgrading.tier, upgrading.reserved_tier()), (1, 2));
    let selection = Selection::new([
        (own, TUNNEL),
        (other, TUNNEL),
        (guard, ObjectKind::Guard),
        (enemy, TUNNEL),
    ]);
    assert_eq!(
        selected_areas(t.sim(), P1, &selection),
        [area(own, 2, rect(1, 3, 10, 10)), area(other, 1, rect(12, 4, 8, 8))]
    );
    assert_eq!(selected_areas(t.sim(), P1, &Selection::new([(enemy, TUNNEL)])), []);
    assert_eq!(selected_areas(t.sim(), P2, &Selection::new([(enemy, TUNNEL)])).len(), 1);
}

/// The rings of a Tunnel ghost on open ground (design change 0299: "the would-be Tier 2 and Tier 3 areas"): Tier 2's
/// 36 cells around the Tier 1 area's 8 × 8, Tier 3's 44 around Tier 2's 10 × 10, none red. A kind without a tier table
/// has none.
#[test]
fn a_tunnel_ghost_has_a_ring_for_each_later_tier() {
    let map = Map::new(32, 32);
    let all = |_| true;
    let rings = rings::rings(&map, TUNNEL, footprint(12, 12), all, &[]);
    let shape = |r: &Ring| (r.tier, r.area, r.inner, r.cells().count());
    let shape: Vec<(u8, TileRect, TileRect, usize)> = rings.iter().map(shape).collect();
    assert_eq!(
        shape,
        [
            (2, rect(9, 9, 10, 10), rect(10, 10, 8, 8), 36),
            (3, rect(8, 8, 12, 12), rect(9, 9, 10, 10), 44),
        ]
    );
    assert!(rings.iter().all(|r| r.blocked.is_empty()));
    assert!(rings[0].holds(t(9, 9)) && !rings[0].holds(t(10, 10)) && !rings[0].holds(t(8, 8)));
    assert_eq!(rings::rings(&map, ObjectKind::Hideout, footprint(12, 12), all, &[]), []);
}

/// Design change 0299's verification, "Place a Tunnel near Water: the T3 ring shows red over the Water": Water in the
/// Tier 3 ring is red there only, Mountain in the Tier 2 ring is red in it, and Water in the Tier 1 area is no ring's
/// (the ghost's own area is red over it: `super::super::ghost_area`). Never-explored ground passes, as on the ghost
/// ("never-explored tiles pass on the ghost").
#[test]
fn undrillable_ground_the_player_knows_is_red_in_its_ring() {
    let mut rows = vec![".".repeat(32); 32];
    for (x, y, c) in [(8, 14, "~"), (9, 15, "M"), (10, 16, "~")] {
        rows[y].replace_range(x..=x, c);
    }
    let map = Map::from_ascii(&rows.join("\n")).expect("a map");
    let rings = rings::rings(&map, TUNNEL, footprint(12, 12), |_| true, &[]);
    let blocked: Vec<&[TilePos]> = rings.iter().map(|r| &r.blocked[..]).collect();
    assert_eq!(blocked, [&[t(9, 15)][..], &[t(8, 14)][..]]);
    let unexplored = rings::rings(&map, TUNNEL, footprint(12, 12), |c| c != t(8, 14), &[]);
    assert_eq!(unexplored[1].blocked, []);
}

/// Design change 0300: "off-map cells (confirmed)" are red. A Tunnel 2 tiles from the west edge has its Tier 1 area
/// on the map (design change 0283) and both rings off it on that side: Tier 2's column of 10, Tier 3's column of 12
/// and its two corner cells beyond the edge. The map's edge shows them, a mark on the west side of each of the 12
/// tiles of column 0 they lie across from.
#[test]
fn cells_off_the_map_are_red_and_marked_on_its_edge() {
    let map = Map::new(32, 32);
    let rings = rings::rings(&map, TUNNEL, footprint(2, 12), |_| true, &[]);
    assert_eq!(rings[0].blocked, column(-1, 9, 18));
    let mut tier_3 = column(-2, 8, 19);
    tier_3.extend([t(-1, 8), t(-1, 19)]);
    tier_3.sort_by_key(|c| (c.y, c.x));
    assert_eq!(rings[1].blocked, tier_3);
    let marks: Vec<(TilePos, Side)> = edge_marks(&map, &rings).into_iter().collect();
    let west: Vec<(TilePos, Side)> = column(0, 8, 19).into_iter().map(|c| (c, Side::West)).collect();
    assert_eq!(marks, west);
    // In the north-west corner, a cell beyond it marks both sides of the corner tile.
    let corner = rings::rings(&map, TUNNEL, footprint(2, 2), |_| true, &[]);
    assert!(edge_marks(&map, &corner).is_superset(&[(t(0, 0), Side::West), (t(0, 0), Side::North)].into()));
}

/// Design change 0300's verification, "Place a Tunnel next to your own: the future T2/T3 rings show red where they
/// would overlap it": with an own Tunnel's Tier 1 area touching the ghost's on the east, the Tier 2 ring's east
/// column and the Tier 3 ring's are red along that area, and nowhere else.
#[test]
fn an_own_tunnels_area_is_red_where_the_rings_would_overlap_it() {
    let map = Map::new(40, 32);
    let own = tunnel::area::area(footprint(20, 12), 2);
    assert_eq!(own, rect(18, 10, 8, 8));
    let rings = rings::rings(&map, TUNNEL, footprint(12, 12), |_| true, &[own]);
    assert_eq!(rings[0].blocked, column(18, 10, 17));
    assert_eq!(rings[1].blocked, column(19, 10, 17));
}

/// What the ghost's check counts against the player's own Tunnels (`sc_sim::Sim::build_check`), each of which would
/// block an upgrade once built (module docs of `rings`): player 1's Tunnel 1 at its Tier 1 area, Tunnel 2, upgrading,
/// at the Tier 2 area it reserves (slice M4-G), the site Agent 5 started (design change 0289 point 3) and the Tier 1
/// area of Agent 6's pending ghost (0289 point 2); never player 2's Tunnel. The ghosts of the Agent placing don't
/// count while a click without Shift would replace its orders.
#[test]
fn the_own_areas_are_those_the_ghosts_check_counts() {
    let mut t = field();
    let (builder, pending) = (ObjectId(5), ObjectId(6));
    let tile_centre = |x: i32, y: i32| Vec2::new(x as f32 + 0.5, y as f32 + 0.5) * 64.0;
    let build = |agent, x, y| Command::Order {
        units: vec![agent],
        kind: OrderKind::Build {
            kind: TUNNEL,
            origin: TilePos::new(x, y),
            orientation: BuildingOrientation::default(),
        },
        queue: false,
    };
    let upgrade = Command::Tunnel {
        tunnel: ObjectId(2),
        command: TunnelCommand::Upgrade,
    };
    t.spawn_unit_at(1, ObjectKind::Agent, P1, tile_centre(34, 3))
        .spawn_unit_at(1, ObjectKind::Agent, P1, tile_centre(4, 16))
        .at(2, P1, build(builder, 34, 2))
        .at(2, P1, build(pending, 40, 14))
        .at(2, P1, upgrade);
    for _ in 0..200 {
        if t.sim().objects().any(|o| o.is_site()) {
            break;
        }
        t.step();
    }
    let site = t.sim().objects().find(|o| o.is_site()).expect("Agent 5's site");
    assert_eq!(site.placement.map(|p| p.origin), Some(TilePos::new(34, 2)));
    let standing = vec![rect(2, 4, 8, 8), rect(11, 3, 10, 10), rect(32, 0, 8, 8)];
    let mut all = standing.clone();
    all.push(rect(38, 12, 8, 8));
    assert_eq!(own_areas(t.sim(), P1, None), all);
    assert_eq!(own_areas(t.sim(), P1, Some(pending)), standing);
    assert_eq!(own_areas(t.sim(), P2, None), [rect(22, 4, 8, 8)]);
}

/// Each area and ring is drawn with its edge on the map only, on the tiles drawn: an 8 × 8 area is 64 tiles and 32
/// tile edges, 4 vertices each; of an area from (-2, -2), the 36 tiles on the map, and 12 edges, to the east and the
/// south. The two rings on open ground: 36 and 44 cells, and their outer edges, 40 and 48 (the inner one is the area
/// within's); red cells take their ring's place. A Tunnel 2 tiles from the west edge: of its rings, 26 and 30 cells on
/// the map, 28 and 32 edges, and the 12 marks of the map's edge.
#[test]
fn each_area_and_ring_is_drawn_with_its_edge_on_the_map_only() {
    let map = Map::new(32, 32);
    let all = |_: TilePos| true;
    let own = area(ObjectId(1), 1, rect(0, 0, 8, 8));
    let one = 64 * 4 + 32 * 4;
    assert_eq!(overlay_mesh(&map, &[own], &[], all).vertex_count(), one);
    let other = area(ObjectId(2), 1, rect(8, 0, 8, 8));
    assert_eq!(overlay_mesh(&map, &[own, other], &[], all).vertex_count(), 2 * one);
    let corner = area(ObjectId(1), 1, rect(-2, -2, 8, 8));
    assert_eq!(overlay_mesh(&map, &[corner], &[], all).vertex_count(), (36 + 12) * 4);
    assert_eq!(
        overlay_mesh(&map, &[own], &[], |t| t.x < 4).vertex_count(),
        (32 + 16) * 4
    );
    assert_eq!(overlay_mesh(&map, &[], &[], all).vertex_count(), 0);

    let open = rings::rings(&map, TUNNEL, footprint(12, 12), all, &[]);
    let rings_only = (36 + 40 + 44 + 48) * 4;
    assert_eq!(overlay_mesh(&map, &[], &open, all).vertex_count(), rings_only);
    let beside = rings::rings(&map, TUNNEL, footprint(12, 12), all, &[rect(18, 10, 8, 8)]);
    assert!(!beside[0].blocked.is_empty());
    assert_eq!(overlay_mesh(&map, &[], &beside, all).vertex_count(), rings_only);
    let by_the_edge = rings::rings(&map, TUNNEL, footprint(2, 12), all, &[]);
    assert_eq!(
        overlay_mesh(&map, &[], &by_the_edge, all).vertex_count(),
        (26 + 28 + 30 + 32 + 12) * 4
    );
    assert_eq!(overlay_mesh(&map, &[], &by_the_edge, |_| false).vertex_count(), 0);
}
