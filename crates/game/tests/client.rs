//! Headless client tests: no window and no GPU. Every `App::update` runs `FixedUpdate` exactly once
//! (`TimeUpdateStrategy::FixedTimesteps(1)`); the first update only starts the clock, so the tests assert on
//! `Sim::tick()`, never on the number of updates (`ARCHITECTURE.md > Client architecture > Client tests`).

use std::collections::BTreeSet;

use bevy::ecs::message::Messages;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use sc_sim::{
    Command, DebugCommand, ObjectId, ObjectKind, PlayerCommand, PlayerId, Scenario, SimEvent, Spawn, SpawnAt, Tick,
    TilePos, scenarios,
};
use space_crystals::camera::{CameraView, ViewportInsets, viewport_rect, visible_extent_su};
use space_crystals::hud::{HUD_BAR_HEIGHT_VH, TOP_BAR_HEIGHT_VH};
use space_crystals::input::{InjectedActions, InputAction};
use space_crystals::map_render::{GridOverlay, MapRenderAssets, MapView};
use space_crystals::object_render::{ObjectVisual, ObjectVisuals};
use space_crystals::platform::{DisplayMode, SurfaceSize};
use space_crystals::qa::QaConfig;
use space_crystals::sim_driver::{PendingCommands, RecordedInputs, SimControl, SimEventMessage, SimRes, load_scenario};
use space_crystals::{AppConfig, build_app};

fn headless(mut config: AppConfig) -> App {
    config.display = DisplayMode::Headless;
    let mut app = build_app(config);
    app.insert_resource(TimeUpdateStrategy::FixedTimesteps(1));
    app.finish();
    app.cleanup();
    app
}

fn headless_app(scenario: Scenario) -> App {
    headless(AppConfig::new(scenario))
}

fn updates(app: &mut App, n: usize) {
    for _ in 0..n {
        app.update();
    }
}

fn tick(app: &App) -> Tick {
    app.world().resource::<SimRes>().tick()
}

fn patch_at(x: i32, y: i32) -> Spawn {
    Spawn {
        kind: ObjectKind::CrystalPatch,
        owner: None,
        at: SpawnAt::Tile(TilePos::new(x, y)),
    }
}

/// A small map with three patches and a station.
fn small_scenario() -> Scenario {
    let mut scenario = scenarios::empty(16, 12);
    scenario
        .spawns
        .extend([patch_at(2, 2), patch_at(3, 2), patch_at(10, 9)]);
    scenario.spawns.push(Spawn {
        kind: ObjectKind::SupplyDeliveryStation,
        owner: None,
        at: SpawnAt::Tile(TilePos::new(6, 5)),
    });
    scenario
}

fn debug(command: DebugCommand) -> PlayerCommand {
    PlayerCommand {
        player: PlayerId(1),
        command: Command::Debug(command),
    }
}

fn visual_ids(app: &mut App) -> BTreeSet<ObjectId> {
    let mut query = app.world_mut().query::<&ObjectVisual>();
    query.iter(app.world()).map(|v| v.0).collect()
}

fn sim_ids(app: &App) -> BTreeSet<ObjectId> {
    app.world().resource::<SimRes>().objects().map(|o| o.id).collect()
}

fn camera_center(app: &App) -> Vec2 {
    app.world().resource::<CameraView>().center
}

fn hold(app: &mut App, actions: &[InputAction]) {
    app.world_mut().resource_mut::<InjectedActions>().held = actions.iter().copied().collect();
}

fn press(app: &mut App, action: InputAction) {
    app.world_mut().resource_mut::<InjectedActions>().pressed.push(action);
}

#[test]
fn n_fixed_steps_are_n_sim_ticks() {
    let mut app = headless_app(small_scenario());
    app.update();
    let start = tick(&app);
    updates(&mut app, 10);
    assert_eq!(tick(&app), start + 10);
    // Every tick's inputs are recorded, empty ones included.
    assert_eq!(
        app.world().resource::<RecordedInputs>().ticks.len(),
        tick(&app) as usize
    );
}

#[test]
fn pending_commands_apply_on_the_next_tick() {
    let mut app = headless_app(scenarios::empty(16, 12));
    updates(&mut app, 3);
    let before = tick(&app);
    assert!(before >= 1);
    let spawn = debug(DebugCommand::Spawn {
        kind: ObjectKind::CrystalPatch,
        owner: None,
        at: SpawnAt::Tile(TilePos::new(3, 4)),
    });
    app.world_mut().resource_mut::<PendingCommands>().0.push(spawn.clone());
    app.world_mut().resource_mut::<Messages<SimEventMessage>>().clear();
    app.update();

    assert_eq!(tick(&app), before + 1);
    assert_eq!(sim_ids(&app).len(), 1);
    assert!(app.world().resource::<PendingCommands>().0.is_empty());
    let recorded = app.world().resource::<RecordedInputs>();
    assert_eq!(recorded.inputs_of(before + 1).unwrap().commands, vec![spawn]);
    assert!(recorded.inputs_of(before).unwrap().is_empty());
    // The tick's events are forwarded as messages.
    let events: Vec<SimEventMessage> = app
        .world_mut()
        .resource_mut::<Messages<SimEventMessage>>()
        .drain()
        .collect();
    let id = *sim_ids(&app).first().unwrap();
    assert_eq!(
        events,
        vec![SimEventMessage {
            tick: before + 1,
            event: SimEvent::Spawned(id)
        }]
    );
}

#[test]
fn a_paused_sim_keeps_its_tick_and_its_pending_commands() {
    let mut app = headless_app(scenarios::empty(16, 12));
    app.update();
    app.world_mut().resource_mut::<SimControl>().paused = true;
    let paused_at = tick(&app);
    app.world_mut()
        .resource_mut::<PendingCommands>()
        .0
        .push(debug(DebugCommand::Remove { id: ObjectId(99) }));
    updates(&mut app, 5);
    assert_eq!(tick(&app), paused_at);
    assert_eq!(app.world().resource::<PendingCommands>().0.len(), 1);
    app.world_mut().resource_mut::<SimControl>().paused = false;
    app.update();
    assert_eq!(tick(&app), paused_at + 1);
    assert!(app.world().resource::<PendingCommands>().0.is_empty());
}

#[test]
fn the_camera_starts_at_the_players_camera_start() {
    let mut scenario = scenarios::empty(64, 48);
    scenario.players[1].camera_start = Some(TilePos::new(40, 30).center());
    let mut config = AppConfig::new(scenario);
    config.player = PlayerId(2);
    let mut app = headless(config);
    app.update();
    assert_eq!(camera_center(&app), TilePos::new(40, 30).center());
    // Player 1 has no camera start and no starting structure yet: its start location stands in.
    let mut app = headless_app(scenarios::empty(64, 48));
    app.update();
    assert_eq!(camera_center(&app), TilePos::new(16, 12).center());
    // Without a start location, the map center.
    let mut scenario = scenarios::empty(64, 48);
    scenario.players[0].start_location = None;
    let mut app = headless_app(scenario);
    app.update();
    assert_eq!(camera_center(&app), Vec2::new(32.0, 24.0) * 64.0);
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

#[test]
fn every_object_has_a_visual_until_it_is_removed() {
    let mut app = headless_app(small_scenario());
    app.update();
    let ids = sim_ids(&app);
    assert_eq!(ids.len(), 4);
    assert_eq!(visual_ids(&mut app), ids);

    // A visual sits at its object's position, flipped into Bevy's +y-up space.
    let patch = app.world().resource::<SimRes>().object(ObjectId(1)).unwrap().pos;
    let entity = app.world().resource::<ObjectVisuals>().by_id[&ObjectId(1)];
    let translation = app.world().get::<Transform>(entity).unwrap().translation;
    assert_eq!(translation.truncate(), Vec2::new(patch.x, -patch.y));

    let removed = ObjectId(2);
    app.world_mut()
        .resource_mut::<PendingCommands>()
        .0
        .push(debug(DebugCommand::Remove { id: removed }));
    app.update();
    let remaining: BTreeSet<ObjectId> = ids.iter().copied().filter(|id| *id != removed).collect();
    assert_eq!(sim_ids(&app), remaining);
    assert_eq!(visual_ids(&mut app), remaining);
    assert!(!app.world().resource::<ObjectVisuals>().by_id.contains_key(&removed));

    // Spawning one later creates its visual too.
    app.world_mut()
        .resource_mut::<PendingCommands>()
        .0
        .push(debug(DebugCommand::Spawn {
            kind: ObjectKind::CrystalPatch,
            owner: None,
            at: SpawnAt::Tile(TilePos::new(12, 3)),
        }));
    app.update();
    assert_eq!(visual_ids(&mut app), sim_ids(&app));
    assert_eq!(visual_ids(&mut app).len(), 4);
}

#[test]
fn southern_objects_draw_in_front() {
    let mut app = headless_app(small_scenario());
    app.update();
    let visuals = app.world().resource::<ObjectVisuals>().by_id.clone();
    let z = |app: &App, id: u32| {
        app.world()
            .get::<Transform>(visuals[&ObjectId(id)])
            .unwrap()
            .translation
            .z
    };
    // Patch 3 (row 9) lies south of patch 1 (row 2); patches 1 and 2 share a row.
    assert!(z(&app, 3) > z(&app, 1));
    assert_eq!(z(&app, 1), z(&app, 2));
}

#[test]
fn the_grid_toggles_with_its_action() {
    let mut app = headless_app(small_scenario());
    app.update();
    let grid_visibility = |app: &mut App| {
        let mut query = app.world_mut().query_filtered::<&Visibility, With<GridOverlay>>();
        *query.single(app.world()).unwrap()
    };
    assert_eq!(grid_visibility(&mut app), Visibility::Hidden);
    press(&mut app, InputAction::ToggleGrid);
    app.update();
    assert_eq!(grid_visibility(&mut app), Visibility::Inherited);
    // A press fires once.
    updates(&mut app, 2);
    assert_eq!(grid_visibility(&mut app), Visibility::Inherited);
    press(&mut app, InputAction::ToggleGrid);
    app.update();
    assert_eq!(grid_visibility(&mut app), Visibility::Hidden);
}

#[test]
fn loading_another_scenario_rebuilds_the_map_and_the_visuals() {
    let mut app = headless_app(small_scenario());
    updates(&mut app, 3);
    assert_eq!(app.world().resource::<MapView>().builds, 1);

    let mut other = scenarios::empty(20, 10);
    other.spawns.push(patch_at(1, 1));
    load_scenario(app.world_mut(), other);
    app.update();

    assert_eq!(app.world().resource::<MapView>().builds, 2);
    let handle = app.world().resource::<MapRenderAssets>().terrain_mesh.clone();
    let vertices = app
        .world()
        .resource::<Assets<Mesh>>()
        .get(&handle)
        .unwrap()
        .count_vertices();
    assert_eq!(vertices, 20 * 10 * 4);
    // Object 1 of the new sim is a different object than object 1 of the old one.
    assert_eq!(visual_ids(&mut app), BTreeSet::from([ObjectId(1)]));
    assert_eq!(
        app.world().resource::<RecordedInputs>().ticks.len(),
        tick(&app) as usize
    );
}

#[test]
fn a_qa_run_holds_at_its_exit_tick_and_dumps_the_state() {
    let dump = std::env::temp_dir().join(format!("sc_client_dump_{}.json", std::process::id()));
    let mut config = AppConfig::new(small_scenario());
    config.qa = QaConfig {
        screenshots: Vec::new(),
        exit_at_tick: Some(5),
        dump_state: Some(dump.clone()),
    };
    let mut app = headless(config);
    let mut exit = None;
    for _ in 0..50 {
        app.update();
        exit = exit.or(app.should_exit());
    }
    assert_eq!(exit, Some(AppExit::Success));
    assert_eq!(tick(&app), 5, "the sim waits at the exit tick");

    let text = std::fs::read_to_string(&dump).unwrap();
    std::fs::remove_file(&dump).unwrap();
    let json: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(json["tick"], 5);
    assert_eq!(json["scenario"], "empty");
    assert_eq!(json["objects"].as_array().unwrap().len(), 4);
    assert_eq!(json["objects"][3]["kind"], "SupplyDeliveryStation");
    assert_eq!(json["client"]["local_player"], 1);
    assert_eq!(json["client"]["recorded_ticks"], 5);
    assert!(json["map"].is_object(), "the map is part of the dump");
}
