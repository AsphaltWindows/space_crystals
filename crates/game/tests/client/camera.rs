//! The camera (`core/camera.md`): where it starts, panning by the pan actions and the middle-mouse grab, and
//! the clamp to the map.

use bevy::prelude::*;

use sc_sim::{Faction, LayoutObject, ObjectKind, PlayerId, SpawnAt, TilePos, scenarios};
use space_crystals::AppConfig;
use space_crystals::camera::{GrabPan, ViewportInsets, cursor_to_view, start_center, viewport_rect, visible_extent_su};
use space_crystals::hud::{HUD_BAR_HEIGHT_VH, TOP_BAR_HEIGHT_VH};
use space_crystals::input::InputAction;
use space_crystals::platform::SurfaceSize;
use space_crystals::sim_driver::SimRes;

use super::{camera_center, centered_scenario, headless, headless_app, hold, press, set_cursor, updates};

#[test]
fn the_camera_starts_at_the_players_camera_start() {
    let mut scenario = scenarios::empty(64, 48);
    scenario.map.slots[1].camera_start = Some(TilePos::new(40, 30).center());
    let mut config = AppConfig::new(scenario);
    config.player = PlayerId(2);
    let mut app = headless(config);
    app.update();
    assert_eq!(camera_center(&app), TilePos::new(40, 30).center());
    // Player 1's slot has no camera start, and its layout no Deployment Center: its start position stands in.
    // (The map center, for a player without a slot, is tested in `camera.rs`.)
    let mut app = headless_app(scenarios::empty(64, 48));
    app.update();
    assert_eq!(camera_center(&app), TilePos::new(16, 12).center());
}

/// `core/camera.md > Starting Position`: without a camera start, the camera starts on the player's primary starting
/// structure, the GDO's Deployment Center, where their Starting Layout puts it. A camera start comes first, and a
/// player of a faction whose Starting Conditions aren't implemented yet starts at their slot's start position.
#[test]
fn the_camera_starts_on_the_players_deployment_center() {
    let mut scenario = scenarios::empty(64, 48);
    let dc = LayoutObject {
        kind: ObjectKind::DeploymentCenter,
        at: SpawnAt::Tile(TilePos::new(20, 15)), // (20, 15) to (23, 18)
        orientation: Default::default(),
    };
    *scenario.map.slots[0].layouts.get_mut(Faction::Gdo) = vec![dc.clone()];
    let on_dc = Vec2::new(22.0, 17.0) * 64.0;
    let mut app = headless_app(scenario.clone());
    app.update();
    assert_eq!(app.world().resource::<SimRes>().objects().next().unwrap().pos, on_dc);
    assert_eq!(camera_center(&app), on_dc);

    let map = scenario.map.terrain.clone();
    *scenario.map.slots[1].layouts.get_mut(Faction::Gdo) = vec![dc];
    scenario.map.slots[0].camera_start = Some(TilePos::new(40, 30).center());
    assert_eq!(
        start_center(&scenario, PlayerId(1), &map),
        TilePos::new(40, 30).center()
    );
    assert_eq!(start_center(&scenario, PlayerId(2), &map), on_dc);
    scenario.setup.players[1].faction = Faction::Syndicate;
    assert_eq!(
        start_center(&scenario, PlayerId(2), &map),
        TilePos::new(47, 35).center()
    );
}

#[test]
fn injected_pan_actions_move_the_camera_and_it_stays_clamped() {
    let mut app = headless_app(scenarios::empty(64, 48));
    updates(&mut app, 2);
    let start = camera_center(&app);

    hold(&mut app, &[InputAction::PanRight]);
    app.update();
    let moved = camera_center(&app);
    assert!(moved.x > start.x, "{moved} should be east of {start}");
    assert_eq!(moved.y, start.y);

    // Keep panning far past the map edge: the view stops at the east edge.
    updates(&mut app, 100);
    let size = *app.world().resource::<SurfaceSize>();
    let insets = *app.world().resource::<ViewportInsets>();
    assert_eq!(
        insets,
        ViewportInsets {
            top_vh: TOP_BAR_HEIGHT_VH,
            bottom_vh: HUD_BAR_HEIGHT_VH,
            left_vh: 0.0,
            right_vh: 0.0,
        }
    );
    let visible = visible_extent_su(viewport_rect(size, insets).1, size.physical.x, 1.0);
    let map = Vec2::new(64.0, 48.0) * 64.0;
    assert_eq!(camera_center(&app), Vec2::new(map.x - visible.x / 2.0, start.y));

    // Up and left into the opposite corner.
    hold(&mut app, &[InputAction::PanLeft, InputAction::PanUp]);
    updates(&mut app, 200);
    assert_eq!(camera_center(&app), visible / 2.0);

    // Releasing the keys stops the camera.
    hold(&mut app, &[]);
    updates(&mut app, 3);
    assert_eq!(camera_center(&app), visible / 2.0);
}

/// `core/camera.md > Panning > Middle-mouse grab`: the grabbed point stays exactly under the cursor while the button
/// is held, and the view stops the moment it is released.
#[test]
fn the_middle_mouse_grab_keeps_the_grabbed_point_under_the_cursor() {
    // The view starts at the map's center, far from the edges where it would be clamped.
    let mut app = headless_app(centered_scenario());
    updates(&mut app, 2);
    let start = camera_center(&app);
    let size = *app.world().resource::<SurfaceSize>();
    let insets = *app.world().resource::<ViewportInsets>();
    let first = Vec2::new(400.0, 300.0);
    set_cursor(&mut app, first);
    press(&mut app, InputAction::GrabPan);
    app.update();
    let grabbed = cursor_to_view(first, size, insets, start, 1.0).unwrap();
    assert_eq!(app.world().resource::<GrabPan>().grabbed, Some(grabbed));
    assert_eq!(camera_center(&app), start, "grabbing alone doesn't move the view");

    hold(&mut app, &[InputAction::GrabPan]);
    for to in [
        Vec2::new(350.0, 280.0),
        Vec2::new(520.0, 430.0),
        Vec2::new(300.0, 200.0),
    ] {
        set_cursor(&mut app, to);
        app.update();
        let under = cursor_to_view(to, size, insets, camera_center(&app), 1.0).unwrap();
        assert!((under - grabbed).length() < 1e-3, "{under} should be {grabbed}");
    }
    assert_ne!(camera_center(&app), start);

    // Releasing stops the view at once, wherever the cursor goes.
    let released_at = camera_center(&app);
    hold(&mut app, &[]);
    set_cursor(&mut app, Vec2::new(640.0, 360.0));
    updates(&mut app, 3);
    assert_eq!(camera_center(&app), released_at);
    assert_eq!(app.world().resource::<GrabPan>().grabbed, None);
}
