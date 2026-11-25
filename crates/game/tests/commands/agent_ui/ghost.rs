//! An Agent's Tunnel ghost (slice M4-J1): judged as the sim's placement check judges a Build, by what the player knows
//! of the map (`sc_sim::Sim::build_check`, slice M4-E; `factions/syndicate/tunnels.md > Tunnel Area Non-Overlap
//! Rule`, design changes 0237 and 0242), its Tier 1 Tunnel Area drawn around it, and the Tunnel Areas the player knows
//! of shown while it is placed. Its problems are the sim's, and a click on a red ghost says the first of them. An own
//! Tunnel's upgrade reserves its larger area against it (slice M4-G).

use sc_sim::tuning::vision::FOG_DELAY_TICKS;
use sc_sim::{PlacementCheck, PlacementProblem, TileProblem, TunnelCommand, WholeProblem};
use space_crystals::command_ui::feedback::placement_text;
use space_crystals::placement::ghost_area::ShownArea;
use space_crystals::placement::{GhostArea, TunnelAreaOverlay};

use super::*;

/// The objects of [`fog_game`], by id.
const OWN_TUNNEL: u32 = 1;
const AGENT_ID: u32 = 2;
const ENEMY_TUNNEL: u32 = 3;
const SCOUT: u32 = 4;
/// Player 1's Agent, in the north of the world view, and the scout by player 2's Tunnel, removed after the first
/// ticks.
const AGENT_AT: Vec2 = Vec2::new(1600.0, 700.0);
const SCOUT_AT: Vec2 = Vec2::new(2688.0, 1376.0);
/// A Water tile in the Agent's sight, and one that nothing of player 1's ever saw.
const KNOWN_WATER: TilePos = TilePos::new(27, 8);
const UNEXPLORED_WATER: TilePos = TilePos::new(34, 4);

/// A game of player 1, the Syndicate, with the fog of war on, on a flat 64 × 48 map with two Water tiles
/// ([`KNOWN_WATER`], [`UNEXPLORED_WATER`]): their Tunnel at tiles (16, 15) to (19, 18) (its Tier 1 area (14, 13) to
/// (21, 20)), their Agent, and player 2's Tunnel at (40, 24) to (43, 27), which player 1's scout saw at Tier 2 (its
/// area (37, 21) to (46, 30)) before it was removed; once out of player 1's sight, the Tunnel went back to Tier 1
/// (area (38, 22) to (45, 29)), unseen. Player 1 has 10 Supplies.
fn fog_game() -> App {
    let mut rows = vec![".".repeat(64); 48];
    for water in [KNOWN_WATER, UNEXPLORED_WATER] {
        let x = water.x as usize;
        rows[water.y as usize].replace_range(x..=x, "~");
    }
    let scenario = ScenarioBuilder::new(&rows.join("\n"))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Syndicate)
        .spawn(TUNNEL_KIND, Some(P1), SpawnAt::Tile(TilePos::new(16, 15)))
        .spawn(AGENT, Some(P1), SpawnAt::Point(AGENT_AT))
        .spawn(TUNNEL_KIND, Some(P2), SpawnAt::Tile(TilePos::new(40, 24)))
        .spawn(ObjectKind::Guard, Some(P1), SpawnAt::Point(SCOUT_AT))
        .build();
    let mut config = AppConfig::new(scenario);
    config.display = DisplayMode::Headless;
    let mut app = build_app(config);
    app.insert_resource(TimeUpdateStrategy::FixedTimesteps(1));
    app.finish();
    app.cleanup();
    let set_tier = |tier| {
        Command::Debug(DebugCommand::SetTier {
            id: ObjectId(ENEMY_TUNNEL),
            tier,
        })
    };
    submit(
        &mut app,
        P1,
        Command::Debug(DebugCommand::SetAutoTargeting { on: false }),
    );
    submit(&mut app, P1, set_tier(2));
    app.update();
    app.world_mut().resource_mut::<CameraView>().center = VIEW_CENTER;
    updates(&mut app, 2);
    submit(
        &mut app,
        P1,
        Command::Debug(DebugCommand::Remove { id: ObjectId(SCOUT) }),
    );
    updates(&mut app, 2 + FOG_DELAY_TICKS as usize);
    submit(&mut app, P1, set_tier(1));
    updates(&mut app, 2);
    set_supplies(&mut app, 10);
    app
}

/// The ghost with the camera on it and the cursor over its centre, at the Tunnel's site at `origin`: its check, and
/// the area drawn around it.
fn ghost_at(app: &mut App, origin: TilePos) -> (PlacementCheck, Option<ShownArea>) {
    look_at(app, centre(origin));
    pointer_over(app, centre(origin));
    let ghost = app.world().resource::<PlacementGhost>().shown.clone().expect("a ghost");
    assert_eq!(ghost.origin, origin);
    (ghost.check, app.world().resource::<GhostArea>().shown)
}

/// The Tier 1 area of a Tunnel at `origin`, with whether it is clear.
fn area_of(origin: TilePos, clear: bool) -> Option<ShownArea> {
    let area = TileRect::new(TilePos::new(origin.x - 2, origin.y - 2), 8, 8);
    Some(ShownArea { area, clear })
}

/// The placement check of a ghost that breaks only `whole` as a whole.
fn only(whole: WholeProblem) -> PlacementCheck {
    PlacementCheck {
        tiles: Vec::new(),
        whole: vec![whole],
    }
}

/// The Tunnel Areas the overlay shows: Tunnel, tier and area.
fn overlay(app: &App) -> Option<Vec<(u32, u8, TileRect)>> {
    let overlay = app.world().resource::<TunnelAreaOverlay>();
    let shown = overlay
        .shown
        .iter()
        .map(|known| (known.tunnel.0, known.tier, known.area));
    overlay.visible.then(|| shown.collect())
}

/// Design change 0242 and `factions/syndicate/tunnels.md > Tunnel Area Non-Overlap Rule`: the ghost is red, its area
/// with it, over the Tier 1 area of player 1's own Tunnel; over the area of player 2's Tunnel "by what the player
/// knows: their remembered position and their tier as last seen" (red within the Tier 2 area remembered, where the
/// Tunnel's real Tier 1 area doesn't reach, green past it); with known Water in its area or under it; and with its
/// area past the map's edge ("no Tunnel near the map edge (closer than its area reaches)"), green a tile further in.
/// Green over unexplored Water: "never-explored tiles pass on the ghost". While it is placed, the overlay shows the
/// areas player 1 knows of, the remembered one at its tier as last seen. A click on a red ghost says why and sends
/// nothing; one on a green ghost sends the Build.
#[test]
fn the_tunnel_ghost_is_judged_by_what_the_player_knows() {
    use WholeProblem::{TunnelAreaNotDrillable, TunnelAreaOffMap, TunnelAreaOverlap};
    let mut app = fog_game();
    click_on(&mut app, AGENT_ID);
    assert_eq!(overlay(&app), None);
    tap(&mut app, &[], KeyA);
    assert!(matches!(interface(&app), InterfaceState::AwaitingPlacement(_)));
    let own_area = TileRect::new(TilePos::new(14, 13), 8, 8);
    let remembered = TileRect::new(TilePos::new(37, 21), 10, 10);
    let known = vec![(OWN_TUNNEL, 1, own_area), (ENEMY_TUNNEL, 2, remembered)];
    assert_eq!(overlay(&app), Some(known));

    let own = TilePos::new(20, 17);
    assert_eq!(ghost_at(&mut app, own), (only(TunnelAreaOverlap), area_of(own, false)));
    let clear_of_own = TilePos::new(24, 17);
    assert_eq!(
        ghost_at(&mut app, clear_of_own),
        (PlacementCheck::default(), area_of(clear_of_own, true))
    );

    let at_last_seen_tier = TilePos::new(48, 24);
    let red = (only(TunnelAreaOverlap), area_of(at_last_seen_tier, false));
    assert_eq!(ghost_at(&mut app, at_last_seen_tier), red);
    let further = TilePos::new(49, 24);
    assert_eq!(
        ghost_at(&mut app, further),
        (PlacementCheck::default(), area_of(further, true))
    );

    let water_in_area = TilePos::new(28, 9);
    let red = (only(TunnelAreaNotDrillable), area_of(water_in_area, false));
    assert_eq!(ghost_at(&mut app, water_in_area), red);
    let (check, _) = ghost_at(&mut app, TilePos::new(26, 6));
    assert_eq!(check.whole, [TunnelAreaNotDrillable]);
    assert!(
        check.problems_at(KNOWN_WATER).eq([TileProblem::NotBuildable]),
        "{check:?}"
    );

    let by_the_edge = TilePos::new(50, 1);
    assert_eq!(
        ghost_at(&mut app, by_the_edge),
        (only(TunnelAreaOffMap), area_of(by_the_edge, false))
    );
    let further_in = TilePos::new(50, 2);
    assert_eq!(
        ghost_at(&mut app, further_in),
        (PlacementCheck::default(), area_of(further_in, true))
    );

    let on_unexplored_water = TilePos::new(33, 3);
    let green = (PlacementCheck::default(), area_of(on_unexplored_water, true));
    assert_eq!(ghost_at(&mut app, on_unexplored_water), green);

    look_at(&mut app, centre(at_last_seen_tier));
    click(&mut app, centre(at_last_seen_tier), &[]);
    let overlap = PlacementProblem::Whole(TunnelAreaOverlap);
    assert_eq!(said(&app), Some(placement_text(Some(overlap))));
    assert_eq!(builds(&app), []);
    assert!(matches!(interface(&app), InterfaceState::AwaitingPlacement(_)));
    look_at(&mut app, centre(on_unexplored_water));
    click(&mut app, centre(on_unexplored_water), &[]);
    let build = given(&[AGENT_ID], build_at(on_unexplored_water), false);
    assert_eq!(builds(&app), [build]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
}

/// Slice M4-G, `factions/syndicate/tunnels.md > Cost Scaling > Upgrade Costs`: "from the start of the upgrade, the
/// larger Tunnel Area of the target tier counts against other placements". While player 1's Tunnel upgrades to Tier
/// 2, their ghost is red over the Tier 2 area it reserves, a column past its Tier 1 area, and green a tile further. The
/// overlay shows the reserved area while the Agent places a Tunnel (Placeholder: Q0086 point 4), the Tunnel's current
/// one while the Tunnel itself is selected.
#[test]
fn an_upgrading_tunnel_reserves_its_larger_area_against_the_ghost() {
    use WholeProblem::TunnelAreaOverlap;
    let mut app = fog_game();
    let upgrade = Command::Tunnel {
        tunnel: ObjectId(OWN_TUNNEL),
        command: TunnelCommand::Upgrade,
    };
    submit(&mut app, P1, upgrade);
    updates(&mut app, 2);
    click_on(&mut app, AGENT_ID);
    tap(&mut app, &[], KeyA);
    let reserved = TileRect::new(TilePos::new(13, 12), 10, 10);
    let remembered = TileRect::new(TilePos::new(37, 21), 10, 10);
    let known = vec![(OWN_TUNNEL, 2, reserved), (ENEMY_TUNNEL, 2, remembered)];
    assert_eq!(overlay(&app), Some(known));
    let beside = TilePos::new(24, 17);
    assert_eq!(
        ghost_at(&mut app, beside),
        (only(TunnelAreaOverlap), area_of(beside, false))
    );
    let further = TilePos::new(25, 17);
    assert_eq!(
        ghost_at(&mut app, further),
        (PlacementCheck::default(), area_of(further, true))
    );

    tap(&mut app, &[], KeyCode::Escape);
    click_on(&mut app, OWN_TUNNEL);
    let current = TileRect::new(TilePos::new(14, 13), 8, 8);
    let shown = overlay(&app).expect("the overlay shows");
    assert_eq!(shown[0], (OWN_TUNNEL, 1, current));
}
