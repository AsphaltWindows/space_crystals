//! The Tunnel Areas the player knows of, and their mesh (slice M4-J2).

use bevy::math::Vec2;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{Command, DebugCommand, Faction, ObjectKind, SpawnAt, TunnelCommand};

use super::*;

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
/// Player 1's Tunnel, player 2's seen one and its hidden one, and player 1's Guard of [`field`].
const OWN: ObjectId = ObjectId(1);
const SEEN: ObjectId = ObjectId(2);
const HIDDEN: ObjectId = ObjectId(3);
const GUARD: ObjectId = ObjectId(4);
/// Where the Guard sees [`SEEN`] from, and where it sees neither of player 2's Tunnels.
const BESIDE: Vec2 = Vec2::new(1480.0, 512.0);
const AWAY: Vec2 = Vec2::new(300.0, 1000.0);

fn rect(x: i32, y: i32, w: u32, h: u32) -> TileRect {
    TileRect::new(TilePos::new(x, y), w, h)
}

/// A flat 48 × 20 map: player 1's Tunnel ([`OWN`]) on tiles (4, 6) to (7, 9); player 2's [`SEEN`] on (24, 6) to
/// (27, 9), which player 1's Guard ([`GUARD`]) sees from [`BESIDE`], and its [`HIDDEN`] on (40, 6) to (43, 9), far
/// from anything of player 1's. Their Tier 1 areas reach 2 tiles past their footprints. No auto-targeting.
fn field() -> TestSim {
    let map = vec![".".repeat(48); 20].join("\n");
    let tile = |x, y| SpawnAt::Tile(TilePos::new(x, y));
    let mut t = ScenarioBuilder::new(&map)
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Syndicate)
        .spawn(ObjectKind::Tunnel, Some(P1), tile(4, 6))
        .spawn(ObjectKind::Tunnel, Some(P2), tile(24, 6))
        .spawn(ObjectKind::Tunnel, Some(P2), tile(40, 6))
        .spawn(ObjectKind::Guard, Some(P1), SpawnAt::Point(BESIDE))
        .start();
    t.without_auto_targeting();
    t
}

fn known(t: &TestSim) -> Vec<KnownArea> {
    known_areas(t.sim(), &Sight::new(t.sim(), P1, true))
}

fn area(tunnel: ObjectId, own: bool, tier: u8, area: TileRect) -> KnownArea {
    KnownArea {
        tunnel,
        own,
        tier,
        area,
    }
}

/// The player knows their own Tunnel's area and that of the enemy Tunnel they see, never that of one they have never
/// seen. Without the fog they know every Tunnel's.
#[test]
fn the_player_knows_their_own_areas_and_the_enemy_ones_they_see() {
    let mut t = field();
    t.run(2);
    assert_eq!(
        known(&t),
        [
            area(OWN, true, 1, rect(2, 4, 8, 8)),
            area(SEEN, false, 1, rect(22, 4, 8, 8)),
        ]
    );
    let everything = known_areas(t.sim(), &Sight::new(t.sim(), P1, false));
    let tunnels: Vec<ObjectId> = everything.iter().map(|known| known.tunnel).collect();
    assert_eq!(tunnels, [OWN, SEEN, HIDDEN]);
    // Player 2 sees nothing of player 1's.
    let theirs: Vec<(ObjectId, bool)> = known_areas(t.sim(), &Sight::new(t.sim(), P2, true))
        .iter()
        .map(|known| (known.tunnel, known.own))
        .collect();
    assert_eq!(theirs, [(SEEN, true), (HIDDEN, true)]);
}

/// Design change 0237: an enemy Tunnel out of sight is known at its remembered position and its tier as last seen,
/// whatever it became since; seen again, at its tier now. The player's own Tunnel's area is always its current tier's.
/// The Guard leaves in tick 3, and the Tunnel's tiles stay Visible for the fog's delay (`core/vision.md > Visible >
/// Fog delay`, 8 ticks).
#[test]
fn an_enemy_tunnel_out_of_sight_keeps_its_last_seen_tier() {
    let mut t = field();
    let away = DebugCommand::SetPosition { id: GUARD, pos: AWAY };
    let back = DebugCommand::SetPosition { id: GUARD, pos: BESIDE };
    t.debug_at(3, away)
        .debug_at(15, DebugCommand::SetTier { id: SEEN, tier: 3 })
        .debug_at(15, DebugCommand::SetTier { id: OWN, tier: 2 })
        .run_until(17);
    assert_eq!(
        known(&t),
        [
            area(OWN, true, 2, rect(1, 3, 10, 10)),
            area(SEEN, false, 1, rect(22, 4, 8, 8)),
        ]
    );
    t.debug_at(18, back).run_until(20);
    assert_eq!(known(&t)[1], area(SEEN, false, 3, rect(20, 2, 12, 12)));
}

/// Slice M4-G (Placeholder: Q0086 point 4): while the player places a Tunnel, their own Tunnel that upgrades shows the
/// Tier 2 area it reserves (`factions/syndicate/tunnels.md > Cost Scaling > Upgrade Costs`: "from the start of the
/// upgrade, the larger Tunnel Area of the target tier counts against other placements"), and the enemy Tunnel they see
/// upgrading its tier now, as the sim lets them know it (`sc_sim::placement::Occupant::as_seen`). The rest of the time
/// both show at their tier now.
#[test]
fn while_placing_a_tunnel_an_own_upgrade_shows_its_reserved_area() {
    let mut t = field();
    let supplies = |player| DebugCommand::SetResources {
        player,
        crystals: 0,
        supplies: 10,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    let upgrade = |tunnel| Command::Tunnel {
        tunnel,
        command: TunnelCommand::Upgrade,
    };
    t.debug_at(1, supplies(P1)).debug_at(1, supplies(P2));
    t.at(2, P1, upgrade(OWN)).at(2, P2, upgrade(SEEN)).run_until(3);
    for id in [OWN, SEEN] {
        let tunnel = t.sim().object(id).and_then(|o| o.tunnel.as_ref()).expect("a Tunnel");
        assert_eq!((tunnel.tier, tunnel.reserved_tier()), (1, 2), "{id:?} upgrades");
    }
    let now = [
        area(OWN, true, 1, rect(2, 4, 8, 8)),
        area(SEEN, false, 1, rect(22, 4, 8, 8)),
    ];
    assert_eq!(known(&t), now);
    let placing = [
        area(OWN, true, 2, rect(1, 3, 10, 10)),
        area(SEEN, false, 1, rect(22, 4, 8, 8)),
    ];
    assert_eq!(reserving(t.sim(), known(&t)), placing);
}

/// Each area is drawn in its owner's colours with its own edge, also where two touch: an 8 × 8 area is 64 tiles and
/// 32 tile edges, 4 vertices each. Neither tint nor edge past the map's edge, nor where the area goes on past it, nor
/// on the tiles not drawn: of an area from (-2, -2), the 36 tiles on the map, and 12 edges, to the east and the south.
#[test]
fn each_area_is_drawn_with_its_edge_on_the_map_only() {
    let map = Map::new(20, 20);
    let own = area(OWN, true, 1, rect(0, 0, 8, 8));
    let enemy = area(SEEN, false, 1, rect(8, 0, 8, 8));
    let one = 64 * 4 + 32 * 4;
    assert_eq!(areas_mesh(&map, &[own], |_| true).vertex_count(), one);
    assert_eq!(areas_mesh(&map, &[own, enemy], |_| true).vertex_count(), 2 * one);
    let corner = area(OWN, true, 1, rect(-2, -2, 8, 8));
    assert_eq!(areas_mesh(&map, &[corner], |_| true).vertex_count(), 36 * 4 + 12 * 4);
    assert_eq!(areas_mesh(&map, &[own], |t| t.x < 4).vertex_count(), 32 * 4 + 16 * 4);
    assert_eq!(areas_mesh(&map, &[], |_| true).vertex_count(), 0);
}
