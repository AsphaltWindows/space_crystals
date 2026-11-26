//! The Tunnel Area display in the client (slice M4-J3; `factions/syndicate/tunnel_area_and_space.md > Tunnel Area
//! display`, design changes 0299 and 0300, answering Q0086 point 4): selecting an own Tunnel shows its area, an enemy's
//! never; placing a Tunnel shows the rings of its ghost's later tiers, red where they would block an upgrade, and
//! rebuilds them only as the ghost moves to another tile.

use sc_sim::nav::TileRect;
use sc_sim::testing::ScenarioBuilder;
use sc_sim::{Faction, SU_PER_GU};
use space_crystals::placement::tunnel_areas::TunnelArea;
use space_crystals::placement::{PlacementGhost, TunnelAreaOverlay};

use super::*;
use KeyCode::{Escape, KeyA};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

/// The objects of [`areas_game`], by id, and where they are drawn (the map is flat).
const TUNNEL: u32 = 1;
const AGENT: u32 = 2;
const ENEMY_TUNNEL: u32 = 4;
const TUNNEL_AT: Vec2 = Vec2::new(1152.0, 1088.0); // tiles (16, 15) to (19, 18); Tier 1 area (14, 13) to (21, 20)
const ENEMY_AT: Vec2 = Vec2::new(2176.0, 1088.0); // tiles (32, 15) to (35, 18); Tier 1 area (30, 13) to (37, 20)
const AGENT_AT: Vec2 = Vec2::new(2464.0, 1952.0); // on tile (38, 30)
const GUARD_AT: Vec2 = Vec2::new(1888.0, 800.0); // on tile (29, 12), in sight of player 2's Tunnel
/// Water in the Agent's sight, and Water nothing of player 1's has seen.
const SEEN_WATER: TilePos = TilePos::new(40, 30);
const UNSEEN_WATER: TilePos = TilePos::new(51, 31);
/// A Tunnel's site whose Tier 1 area touches player 1's Tunnel's on the west and player 2's on the east, and one
/// whose Tier 3 ring holds both Waters, its Tier 1 area and Tier 2 ring clear of them.
const NEXT_TO_OWN: TilePos = TilePos::new(24, 15);
const NEAR_WATER: TilePos = TilePos::new(44, 28);

/// A game of player 1, the Syndicate, on a flat 64 × 48 map with two Water tiles ([`SEEN_WATER`], [`UNSEEN_WATER`]),
/// as [`game`] sets it up (the fog of war not drawn; the sim's sight as ever): their Tunnel, their Agent by the seen
/// Water, their Guard by player 2's Tunnel, which it sees, and 10 Supplies.
fn areas_game() -> App {
    let mut rows = vec![".".repeat(64); 48];
    for water in [SEEN_WATER, UNSEEN_WATER] {
        let x = water.x as usize;
        rows[water.y as usize].replace_range(x..=x, "~");
    }
    let tile = |x, y| SpawnAt::Tile(TilePos::new(x, y));
    let builder = ScenarioBuilder::new(&rows.join("\n"))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Syndicate)
        .spawn(ObjectKind::Tunnel, Some(P1), tile(16, 15))
        .spawn(ObjectKind::Agent, Some(P1), SpawnAt::Point(AGENT_AT))
        .spawn(ObjectKind::Guard, Some(P1), SpawnAt::Point(GUARD_AT))
        .spawn(ObjectKind::Tunnel, Some(P2), tile(32, 15));
    let mut app = game_of(builder.build());
    let supplies = DebugCommand::SetResources {
        player: P1,
        crystals: 0,
        supplies: 10,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    submit(&mut app, P1, Command::Debug(supplies));
    app.update();
    app
}

/// The middle of a Tunnel's footprint at `origin` (4 × 4 tiles), where the cursor puts its ghost there.
fn centre(origin: TilePos) -> Vec2 {
    (Vec2::new(origin.x as f32, origin.y as f32) + 2.0) * SU_PER_GU
}

/// The cells of column `x`, from row `y0` to row `y1`, both included.
fn column(x: i32, y0: i32, y1: i32) -> Vec<TilePos> {
    (y0..=y1).map(|y| TilePos::new(x, y)).collect()
}

/// The areas of the Tunnels selected that the display shows: Tunnel, tier and area.
type Areas = Vec<(u32, u8, TileRect)>;

/// What the display shows while it shows: the areas of the Tunnels selected, and the tiers of the ghost's rings.
fn display(app: &App) -> Option<(Areas, Vec<u8>)> {
    let overlay = app.world().resource::<TunnelAreaOverlay>();
    let area = |shown: &TunnelArea| (shown.tunnel.0, shown.tier, shown.area);
    let areas = overlay.selected.iter().map(area);
    let rings = overlay.rings.iter().map(|ring| ring.tier);
    overlay.visible.then(|| (areas.collect(), rings.collect()))
}

/// The ghost with the camera on it and the cursor over its middle, at a Tunnel's site at `origin`: the red cells of its
/// rings, Tier 2's then Tier 3's, as the display shows them.
fn red_at(app: &mut App, origin: TilePos) -> Vec<Vec<TilePos>> {
    app.world_mut().resource_mut::<CameraView>().center = centre(origin);
    app.update();
    pointer_over(app, centre(origin));
    let ghost = app.world().resource::<PlacementGhost>().shown.clone().expect("a ghost");
    assert_eq!(ghost.origin, origin);
    assert!(ghost.is_valid(), "{:?}", ghost.check);
    let overlay = app.world().resource::<TunnelAreaOverlay>();
    assert!(overlay.visible, "the rings show");
    overlay.rings.iter().map(|ring| ring.blocked.clone()).collect()
}

/// Selects player 1's Agent, the camera on it, and presses A, Build Tunnel.
fn place_a_tunnel(app: &mut App) {
    app.world_mut().resource_mut::<CameraView>().center = AGENT_AT;
    app.update();
    click(app, AGENT_AT, &[]);
    assert_eq!(selected(app), [AGENT]);
    tap(app, &[], KeyA);
    assert!(matches!(interface(app), InterfaceState::AwaitingPlacement(_)));
}

/// Design change 0299: "Selecting one of your Tunnels shows its Tunnel Area on the map", at its tier now, and "Select
/// an enemy Tunnel: no area is drawn" (its Verification), though player 1 sees it. The area hides as the selection
/// changes.
#[test]
fn selecting_an_own_tunnel_shows_its_area_and_an_enemys_none() {
    let mut app = areas_game();
    assert_eq!(display(&app), None);
    click(&mut app, TUNNEL_AT, &[]);
    assert_eq!(selected(&app), [TUNNEL]);
    let area = TileRect::new(TilePos::new(14, 13), 8, 8);
    assert_eq!(display(&app), Some((vec![(TUNNEL, 1, area)], vec![])));

    let sim = app.world().resource::<SimRes>();
    let enemy = sim.object(ObjectId(ENEMY_TUNNEL)).expect("player 2's Tunnel");
    assert!(sim.vision().sees(P1, enemy), "player 1 sees it");
    click(&mut app, ENEMY_AT, &[]);
    assert_eq!(selected(&app), [ENEMY_TUNNEL]);
    assert_eq!(display(&app), None, "no area is drawn");
    let overlay = app.world().resource::<TunnelAreaOverlay>();
    assert!(overlay.selected.is_empty() && overlay.rings.is_empty());
}

/// Design change 0299's Verification: "Place a Tunnel near Water: the T3 ring shows red over the Water". While the
/// Agent places a Tunnel, the display shows its ghost's Tier 2 and Tier 3 rings, and no Tunnel's area. With the Water
/// player 1 has seen in the Tier 3 ring, that ring is red over it, and only there: the Water no one of theirs has seen
/// passes, as on the ghost ("never-explored tiles pass on the ghost", `factions/syndicate/tunnel_area_and_space.md >
/// Tunnel Area Non-Overlap Rule > Drillable area`), and the Tier 2 ring is clear.
#[test]
fn placing_a_tunnel_near_water_shows_the_tier_3_ring_red_over_it() {
    let mut app = areas_game();
    place_a_tunnel(&mut app);
    let red = red_at(&mut app, NEAR_WATER);
    assert_eq!(red, [vec![], vec![SEEN_WATER]]);
    let sim = app.world().resource::<SimRes>();
    let explored = |t| sim.vision().state(P1, t) != sc_sim::VisibilityState::Unexplored;
    assert!(explored(SEEN_WATER) && !explored(UNSEEN_WATER));
    assert_eq!(display(&app), Some((vec![], vec![2, 3])));
    let overlay = app.world().resource::<TunnelAreaOverlay>();
    assert!(
        overlay.rings[1].holds(UNSEEN_WATER),
        "the unseen Water lies in the Tier 3 ring too"
    );
}

/// Design change 0300's Verification: "Place a Tunnel next to your own: the future T2/T3 rings show red where they
/// would overlap it". The ghost's Tier 1 area touches player 1's Tunnel's on the west: the Tier 2 ring's west column
/// is red along it, and the Tier 3 ring's. It touches player 2's on the east, which player 1 sees: "Enemy areas stay
/// hidden", so the rings are never red over it, though an enemy Tunnel blocks an upgrade too.
#[test]
fn placing_a_tunnel_next_to_your_own_shows_the_overlap_red() {
    let mut app = areas_game();
    place_a_tunnel(&mut app);
    let red = red_at(&mut app, NEXT_TO_OWN);
    assert_eq!(red, [column(21, 13, 20), column(20, 13, 20)]);
    let overlay = app.world().resource::<TunnelAreaOverlay>();
    assert!(
        overlay.rings[0].holds(TilePos::new(30, 15)),
        "the Tier 2 ring reaches player 2's area"
    );
    assert_eq!(display(&app), Some((vec![], vec![2, 3])));

    tap(&mut app, &[], Escape);
    assert_eq!(display(&app), None, "placement over, the rings go");
}

/// The rings follow the ghost tile by tile (slice M4-J3: rebuilt "when the ghost moves to another tile", never every
/// frame): the cursor moving within the ghost's tile and frames passing leave the mesh as it is; a tile further east,
/// it is rebuilt once.
#[test]
fn the_rings_are_rebuilt_only_as_the_ghost_moves_to_another_tile() {
    let mut app = areas_game();
    place_a_tunnel(&mut app);
    red_at(&mut app, NEAR_WATER);
    let builds = |app: &App| app.world().resource::<TunnelAreaOverlay>().builds;
    let built = builds(&app);
    pointer_over(&mut app, centre(NEAR_WATER) + Vec2::new(10.0, 12.0));
    updates(&mut app, 5);
    let ghost = app.world().resource::<PlacementGhost>().shown.clone().expect("a ghost");
    assert_eq!(ghost.origin, NEAR_WATER, "the same tile");
    assert_eq!(builds(&app), built);
    let east = TilePos::new(NEAR_WATER.x + 1, NEAR_WATER.y);
    pointer_over(&mut app, centre(east));
    assert_eq!(
        app.world()
            .resource::<PlacementGhost>()
            .shown
            .as_ref()
            .map(|g| g.origin),
        Some(east)
    );
    assert_eq!(builds(&app), built + 1);
    updates(&mut app, 5);
    assert_eq!(builds(&app), built + 1);
}
