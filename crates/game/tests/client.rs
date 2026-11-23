//! Headless client tests: no window and no GPU. Every `App::update` runs `FixedUpdate` exactly once
//! (`TimeUpdateStrategy::FixedTimesteps(1)`), and real time advances by one tick (1/16 s) per update; the first
//! update only starts the clock, so the tests assert on `Sim::tick()`, never on the number of updates
//! (`ARCHITECTURE.md > Client architecture > Client tests`). Input arrives through `InjectedInput`, a virtual
//! keyboard and mouse (`ARCHITECTURE.md > Client architecture > Input`).

use std::collections::{BTreeMap, BTreeSet};

use bevy::ecs::message::{Message, Messages};
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use sc_sim::{
    Command, CornerPos, DebugCommand, ObjectId, ObjectKind, PlayerCommand, PlayerId, Replay, Scenario, Sim, SimConfig,
    SimEvent, Spawn, SpawnAt, SpawnSettings, Tick, TilePos, scenarios,
};
use space_crystals::camera::{
    CameraView, GrabPan, ViewportInsets, cursor_to_ground, cursor_to_view, view_rect, viewport_rect, visible_extent_su,
};
use space_crystals::coords::{drawn_position, ground_under, lift};
use space_crystals::hud::{HUD_BAR_HEIGHT_VH, TOP_BAR_HEIGHT_VH};
use space_crystals::input::{
    ActionState, Binding, Bindings, Chord, CommandSlot, ControlGroup, DRAG_THRESHOLD_PX, InjectedInput, InputAction,
    MapClick, Modifiers, SelectionBox,
};
use space_crystals::map_render::{GridOverlay, MapRenderAssets, MapView};
use space_crystals::object_render::{ObjectVisual, ObjectVisuals};
use space_crystals::platform::{DisplayMode, SurfaceSize};
use space_crystals::qa::QaConfig;
use space_crystals::sim_driver::{
    LoadedScenario, PendingCommands, RecordedInputs, RenderAlpha, SimControl, SimEventMessage, SimRes, load_scenario,
};
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
    Spawn::new(ObjectKind::CrystalPatch, None, SpawnAt::Tile(TilePos::new(x, y)))
}

/// A small map with three patches and a station.
fn small_scenario() -> Scenario {
    let mut scenario = scenarios::empty(16, 12);
    scenario
        .spawns
        .extend([patch_at(2, 2), patch_at(3, 2), patch_at(10, 9)]);
    scenario.spawns.push(Spawn::new(
        ObjectKind::SupplyDeliveryStation,
        None,
        SpawnAt::Tile(TilePos::new(6, 5)),
    ));
    scenario
}

/// A 32 × 24 map with a plateau at level 3 on corners (12, 9) to (20, 15), its slopes falling away around it, and
/// a patch in the middle of the plateau at tile (16, 12), where the camera can center on it.
fn plateau_scenario() -> Scenario {
    let mut scenario = scenarios::empty(32, 24);
    let targets: BTreeMap<CornerPos, u8> = (12..=20)
        .flat_map(|x| (9..=15).map(move |y| (CornerPos::new(x, y), 3)))
        .collect();
    scenario.map.set_corner_heights(&targets);
    scenario.spawns.push(patch_at(16, 12));
    assert_eq!(scenario.validate(), vec![]);
    scenario
}

/// The virtual keyboard and mouse.
fn input(app: &mut App) -> Mut<'_, InjectedInput> {
    app.world_mut().resource_mut::<InjectedInput>()
}

fn set_cursor(app: &mut App, cursor: Vec2) {
    input(app).cursor = Some(cursor);
}

/// This frame's game input.
fn actions(app: &App) -> &ActionState {
    app.world().resource::<ActionState>()
}

/// The messages of type `M` sent since the last call.
fn drain<M: Message>(app: &mut App) -> Vec<M> {
    app.world_mut().resource_mut::<Messages<M>>().drain().collect()
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

/// Holds exactly these actions, whatever their bindings.
fn hold(app: &mut App, actions: &[InputAction]) {
    input(app).held = actions.iter().copied().collect();
}

/// Fires an action once, in the next frame, whatever its bindings.
fn press(app: &mut App, action: InputAction) {
    input(app).pressed.push(action);
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

/// `core/camera.md > Panning > Middle-mouse grab`: the grabbed point stays exactly under the cursor while the button
/// is held, and the view stops the moment it is released.
#[test]
fn the_middle_mouse_grab_keeps_the_grabbed_point_under_the_cursor() {
    // The view starts at the map's center, far from the edges where it would be clamped.
    let mut scenario = scenarios::empty(64, 48);
    scenario.players[0].start_location = None;
    let mut app = headless_app(scenario);
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

/// Objects stand on the terrain (drawn lifted by its height), and every pick resolves to the ground drawn under
/// the cursor.
#[test]
fn objects_and_picking_follow_the_terrain_height() {
    let mut app = headless_app(plateau_scenario());
    updates(&mut app, 2);
    let patch = app.world().resource::<SimRes>().object(ObjectId(1)).unwrap().pos;
    let entity = app.world().resource::<ObjectVisuals>().by_id[&ObjectId(1)];
    let translation = app.world().get::<Transform>(entity).unwrap().translation;
    assert_eq!(translation.truncate(), Vec2::new(patch.x, -(patch.y - lift(3.0))));

    // Point the camera at the patch: the cursor at the viewport's center picks the patch's ground point.
    let map = app.world().resource::<SimRes>().map().clone();
    app.world_mut().resource_mut::<CameraView>().center = drawn_position(&map, patch);
    app.update();
    assert_eq!(camera_center(&app), drawn_position(&map, patch), "not clamped");
    let size = *app.world().resource::<SurfaceSize>();
    let insets = *app.world().resource::<ViewportInsets>();
    let (position, viewport) = viewport_rect(size, insets);
    let middle = (position.as_vec2() + viewport.as_vec2() / 2.0) / size.scale_factor;
    let view = *app.world().resource::<CameraView>();
    let ground = cursor_to_ground(middle, size, insets, view, 1.0, &map).unwrap();
    assert!((ground - patch).length() < 0.01, "{ground} should be {patch}");
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

// The input layer (`ARCHITECTURE.md > Client architecture > Input`), driven through the virtual keyboard and mouse.

fn group(n: u8) -> ControlGroup {
    ControlGroup::new(n).unwrap()
}

fn slot(row: u8, column: u8) -> InputAction {
    InputAction::CommandSlot(CommandSlot::new(row, column).unwrap())
}

/// Taps `key` while the modifier keys `modifiers` are down, for one frame, and returns the actions the tap fired
/// (without the modifier actions, which the modifiers hold).
fn tap(app: &mut App, modifiers: &[KeyCode], key: KeyCode) -> Vec<InputAction> {
    for &m in modifiers {
        input(app).keys.press(m);
    }
    input(app).tap_key(key);
    app.update();
    for &m in modifiers {
        input(app).keys.release(m);
    }
    let roles = [
        InputAction::Queue,
        InputAction::ToggleSelection,
        InputAction::SelectSameType,
    ];
    actions(app).pressed_actions().filter(|a| !roles.contains(a)).collect()
}

/// The middle of the world view on the surface, in logical pixels.
fn view_middle(app: &App) -> Vec2 {
    let size = *app.world().resource::<SurfaceSize>();
    let insets = *app.world().resource::<ViewportInsets>();
    let (position, viewport) = viewport_rect(size, insets);
    (position.as_vec2() + viewport.as_vec2() / 2.0) / size.scale_factor
}

/// The view point under `screen`, with the camera where it is now.
fn view_at(app: &App, screen: Vec2) -> Vec2 {
    let size = *app.world().resource::<SurfaceSize>();
    let insets = *app.world().resource::<ViewportInsets>();
    cursor_to_view(screen, size, insets, camera_center(app), 1.0).unwrap()
}

/// `systems/controls/selection_and_hud.md > ControlGroups > Default keys`: a number recalls its group, Ctrl assigns,
/// Shift adds, and pressing the number twice quickly recalls and centers.
#[test]
fn control_group_keys_resolve_by_their_modifiers_and_double_taps() {
    use KeyCode::{ControlRight, Digit1, Digit2, ShiftLeft};
    let mut app = headless_app(small_scenario());
    updates(&mut app, 2);
    assert_eq!(tap(&mut app, &[], Digit1), [InputAction::RecallGroup(group(1))]);
    // The next press of 1, a frame (1/16 s) later, completes a double tap; a third one starts over.
    assert_eq!(
        tap(&mut app, &[], Digit1),
        [InputAction::RecallAndCenterGroup(group(1))]
    );
    assert_eq!(tap(&mut app, &[], Digit1), [InputAction::RecallGroup(group(1))]);
    // Too slow (over half a second later), or another key in between: single presses.
    updates(&mut app, 8);
    assert_eq!(tap(&mut app, &[], Digit1), [InputAction::RecallGroup(group(1))]);
    assert_eq!(tap(&mut app, &[], Digit2), [InputAction::RecallGroup(group(2))]);
    assert_eq!(tap(&mut app, &[], Digit1), [InputAction::RecallGroup(group(1))]);
    // Either Ctrl assigns and either Shift adds, also twice in a row: a chord with more modifiers beats a double tap.
    assert_eq!(
        tap(&mut app, &[ControlRight], Digit2),
        [InputAction::AssignGroup(group(2))]
    );
    assert_eq!(tap(&mut app, &[ShiftLeft], Digit2), [InputAction::AddToGroup(group(2))]);
    assert_eq!(tap(&mut app, &[ShiftLeft], Digit2), [InputAction::AddToGroup(group(2))]);
}

/// The command panel's hotkeys (`> CommandPanel > Grid Layout`, `> Standard Slot Assignments`), group cycling and
/// cancel. Shift + a hotkey still presses the slot, with the queue modifier held
/// (`systems/controls/commands.md > Unit Command`).
#[test]
fn hotkeys_fire_their_actions_and_modifiers_hold_their_roles() {
    use KeyCode::*;
    let mut app = headless_app(small_scenario());
    updates(&mut app, 2);
    assert_eq!(tap(&mut app, &[], KeyQ), [slot(0, 0)]);
    assert_eq!(tap(&mut app, &[], KeyF), [slot(1, 3)]);
    assert_eq!(tap(&mut app, &[], KeyZ), [InputAction::CommandSlot(CommandSlot::BACK)]);
    assert_eq!(
        tap(&mut app, &[], KeyX),
        [InputAction::CommandSlot(CommandSlot::CANCEL_PRODUCTION)]
    );
    assert_eq!(
        tap(&mut app, &[], KeyC),
        [InputAction::CommandSlot(CommandSlot::RALLY_POINT)]
    );
    assert_eq!(tap(&mut app, &[], Tab), [InputAction::NextGroup]);
    assert_eq!(tap(&mut app, &[ShiftRight], Tab), [InputAction::PreviousGroup]);
    assert_eq!(tap(&mut app, &[], Escape), [InputAction::Cancel]);
    assert!(tap(&mut app, &[], KeyP).is_empty(), "P is not bound");

    // Shift + W: slot W, queued. The slot is held while its key is down, the roles while their modifier is.
    input(&mut app).keys.press(ShiftLeft);
    input(&mut app).keys.press(KeyW);
    app.update();
    let state = actions(&app);
    assert!(state.just_pressed(slot(0, 1)));
    assert!(state.held(InputAction::Queue) && state.held(InputAction::ToggleSelection));
    assert!(!state.held(InputAction::SelectSameType));
    assert_eq!(state.modifiers(), Modifiers::SHIFT);
    input(&mut app).keys.release(ShiftLeft);
    app.update();
    let state = actions(&app);
    assert!(state.held(slot(0, 1)) && !state.just_pressed(slot(0, 1)));
    assert!(state.just_released(InputAction::Queue) && !state.held(InputAction::Queue));
    input(&mut app).keys.release(KeyW);
    app.update();
    assert!(actions(&app).just_released(slot(0, 1)));
}

/// Gameplay asks for actions, never for keys: rebinding one changes the table and nothing else.
#[test]
fn a_rebound_action_follows_its_new_binding() {
    let mut app = headless_app(small_scenario());
    updates(&mut app, 2);
    {
        let mut bindings = app.world_mut().resource_mut::<Bindings>();
        bindings.unbind(InputAction::ToggleGrid);
        bindings.bind(
            InputAction::ToggleGrid,
            Binding::Press(Chord::key(KeyCode::KeyH).with(Modifiers::ALT)),
        );
    }
    let grid_shown = |app: &mut App| {
        let mut query = app.world_mut().query_filtered::<&Visibility, With<GridOverlay>>();
        *query.single(app.world()).unwrap() != Visibility::Hidden
    };
    assert!(tap(&mut app, &[], KeyCode::KeyG).is_empty());
    assert!(tap(&mut app, &[], KeyCode::KeyH).is_empty(), "H needs Alt now");
    assert!(!grid_shown(&mut app));
    assert_eq!(
        tap(&mut app, &[KeyCode::AltLeft], KeyCode::KeyH),
        [InputAction::ToggleGrid]
    );
    assert!(grid_shown(&mut app));
}

/// `core/camera.md > Panning`: the arrow keys and the middle button pan through their bindings; a modifier pressed
/// meanwhile doesn't stop them.
#[test]
fn arrow_keys_and_the_middle_button_pan_through_their_bindings() {
    let mut scenario = scenarios::empty(64, 48);
    scenario.players[0].start_location = None;
    let mut app = headless_app(scenario);
    updates(&mut app, 2);
    let start = camera_center(&app);
    input(&mut app).keys.press(KeyCode::ArrowLeft);
    updates(&mut app, 2);
    let moved = camera_center(&app);
    assert!(
        moved.x < start.x && moved.y == start.y,
        "{moved} should be west of {start}"
    );
    input(&mut app).keys.press(KeyCode::ControlLeft);
    app.update();
    assert!(camera_center(&app).x < moved.x);
    input(&mut app).keys.release(KeyCode::ArrowLeft);
    input(&mut app).keys.release(KeyCode::ControlLeft);
    app.update();
    let stopped = camera_center(&app);
    updates(&mut app, 2);
    assert_eq!(camera_center(&app), stopped);

    let grab_at = view_middle(&app);
    set_cursor(&mut app, grab_at);
    input(&mut app).mouse.press(MouseButton::Middle);
    app.update();
    let grabbed = view_at(&app, grab_at);
    assert_eq!(app.world().resource::<GrabPan>().grabbed, Some(grabbed));
    let moved_to = grab_at + Vec2::new(-90.0, 40.0);
    set_cursor(&mut app, moved_to);
    app.update();
    assert_ne!(camera_center(&app), stopped);
    assert!((view_at(&app, moved_to) - grabbed).length() < 1e-3);
    input(&mut app).mouse.release(MouseButton::Middle);
    app.update();
    assert_eq!(app.world().resource::<GrabPan>().grabbed, None);
}

/// `> MapClickSelection`: a left click on the map is sent when released, with the view point and the ground drawn
/// there (they differ on raised ground), the modifiers, and whether it completes a double-click. A right click (the
/// context command) is sent when pressed.
#[test]
fn clicks_on_the_map_carry_the_ground_point_the_modifiers_and_double_clicks() {
    let mut app = headless_app(plateau_scenario());
    updates(&mut app, 2);
    // Look at the middle of the plateau.
    let map = app.world().resource::<SimRes>().map().clone();
    let patch = TilePos::new(16, 12).center();
    app.world_mut().resource_mut::<CameraView>().center = drawn_position(&map, patch);
    let at = view_middle(&app);
    set_cursor(&mut app, at);
    app.update();
    let view = view_at(&app, at);
    let pointer = *actions(&app).pointer();
    assert_eq!(pointer.screen, Some(at));
    assert!(!pointer.over_ui);
    assert_eq!(pointer.view, Some(view));
    assert_eq!(pointer.ground, Some(ground_under(&map, view)));
    let ground = pointer.ground.unwrap();
    assert!(
        (ground - patch).length() < 0.01,
        "{ground} should be the plateau's {patch}"
    );
    assert_ne!(ground, view);

    input(&mut app).mouse.press(MouseButton::Left);
    app.update();
    assert!(
        drain::<MapClick>(&mut app).is_empty(),
        "a left click is sent when released"
    );
    input(&mut app).mouse.release(MouseButton::Left);
    app.update();
    let click = MapClick {
        action: InputAction::Select,
        screen: at,
        view,
        ground,
        modifiers: Modifiers::NONE,
        double: false,
    };
    assert_eq!(drain::<MapClick>(&mut app), [click]);

    // Another click a little to the side, quickly, with Shift held: a double-click, whose Shift means "toggle".
    let near = at + Vec2::new(3.0, -2.0);
    set_cursor(&mut app, near);
    input(&mut app).keys.press(KeyCode::ShiftLeft);
    input(&mut app).mouse.press(MouseButton::Left);
    input(&mut app).mouse.release(MouseButton::Left);
    updates(&mut app, 2);
    let clicks = drain::<MapClick>(&mut app);
    assert_eq!(clicks.len(), 1);
    assert_eq!((clicks[0].screen, clicks[0].modifiers), (near, Modifiers::SHIFT));
    assert!(clicks[0].double);
    let bindings = app.world().resource::<Bindings>();
    assert!(bindings.holds(InputAction::ToggleSelection, clicks[0].modifiers));
    assert!(!bindings.holds(InputAction::SelectSameType, clicks[0].modifiers));
    input(&mut app).keys.release(KeyCode::ShiftLeft);

    input(&mut app).mouse.press(MouseButton::Right);
    app.update();
    let clicks = drain::<MapClick>(&mut app);
    assert_eq!(clicks.len(), 1);
    assert_eq!(clicks[0].action, InputAction::ContextCommand);
    assert_eq!(clicks[0].ground, ground_under(&map, view_at(&app, near)));
    assert!(!clicks[0].double, "the previous click was another action's");
    input(&mut app).mouse.release(MouseButton::Right);
    app.update();
    assert!(drain::<MapClick>(&mut app).is_empty());
}

/// `> BoxSelection`: a left press that moves further than the drag threshold is a box in view space, shown while
/// dragging and sent when released, kept inside the world view.
#[test]
fn dragging_the_select_button_makes_a_box() {
    let mut app = headless_app(scenarios::empty(64, 48));
    updates(&mut app, 2);
    let start = view_middle(&app);
    set_cursor(&mut app, start);
    input(&mut app).mouse.press(MouseButton::Left);
    app.update();
    set_cursor(&mut app, start + Vec2::new(DRAG_THRESHOLD_PX, 0.0));
    app.update();
    assert_eq!(
        actions(&app).pointer().drag_box,
        None,
        "not further than the threshold yet"
    );
    let end = start + Vec2::new(-120.0, 80.0);
    set_cursor(&mut app, end);
    app.update();
    let rect = Rect::from_corners(view_at(&app, start), view_at(&app, end));
    assert_eq!(actions(&app).pointer().drag_box, Some(rect));
    input(&mut app).mouse.release(MouseButton::Left);
    app.update();
    let dragged = SelectionBox {
        rect,
        modifiers: Modifiers::NONE,
    };
    assert_eq!(drain::<SelectionBox>(&mut app), [dragged]);
    assert!(drain::<MapClick>(&mut app).is_empty(), "a drag is no click");
    assert_eq!(actions(&app).pointer().drag_box, None);

    // A drag into the HUD's bottom bar ends at the world view's bottom edge.
    set_cursor(&mut app, start);
    input(&mut app).mouse.press(MouseButton::Left);
    app.update();
    let size = *app.world().resource::<SurfaceSize>();
    let insets = *app.world().resource::<ViewportInsets>();
    set_cursor(&mut app, Vec2::new(start.x + 50.0, size.logical().y - 2.0));
    input(&mut app).keys.press(KeyCode::ControlLeft);
    app.update();
    assert!(actions(&app).pointer().over_ui);
    input(&mut app).mouse.release(MouseButton::Left);
    app.update();
    let boxes = drain::<SelectionBox>(&mut app);
    let visible = view_rect(size, insets, camera_center(&app), 1.0);
    assert_eq!(boxes.len(), 1);
    assert_eq!(boxes[0].rect.min, view_at(&app, start));
    assert_eq!(boxes[0].rect.max.y, visible.max.y);
    assert_eq!(boxes[0].modifiers, Modifiers::CTRL);
}

/// No map click or box starts over the UI: over a UI node (injected: headless apps have no UI picking), or over the
/// HUD's bars, outside the world view.
#[test]
fn clicks_over_the_ui_never_reach_the_map() {
    let mut app = headless_app(small_scenario());
    updates(&mut app, 2);
    let middle = view_middle(&app);
    let click_both = |app: &mut App| {
        for button in [MouseButton::Left, MouseButton::Right] {
            input(app).mouse.press(button);
            input(app).mouse.release(button);
        }
        updates(app, 2);
        drain::<MapClick>(app).len()
    };
    set_cursor(&mut app, middle);
    input(&mut app).over_ui = true;
    assert_eq!(click_both(&mut app), 0);
    let pointer = *actions(&app).pointer();
    assert_eq!(pointer.screen, Some(middle));
    assert!(pointer.over_ui && pointer.view.is_none() && pointer.ground.is_none());

    input(&mut app).over_ui = false;
    set_cursor(&mut app, Vec2::new(middle.x, 2.0)); // the top bar
    assert_eq!(click_both(&mut app), 0);
    assert!(actions(&app).pointer().over_ui);

    set_cursor(&mut app, middle);
    assert_eq!(click_both(&mut app), 2);
    assert!(!actions(&app).pointer().over_ui);
}

// Replays and interpolation.

/// The commands the replay test's client submits at each frame: accepted ones and rejected ones.
fn scripted_commands(frame: u32) -> Vec<PlayerCommand> {
    let spawn = |kind, x, y| {
        debug(DebugCommand::Spawn {
            kind,
            owner: None,
            at: SpawnAt::Tile(TilePos::new(x, y)),
        })
    };
    let stranger = PlayerCommand {
        player: PlayerId(9),
        command: Command::Debug(DebugCommand::Remove { id: ObjectId(1) }),
    };
    match frame {
        3 => vec![spawn(ObjectKind::CrystalPatch, 12, 8)],
        7 => vec![
            debug(DebugCommand::Remove { id: ObjectId(2) }),
            spawn(ObjectKind::SupplyDeliveryStation, 1, 8),
        ],
        9 => vec![debug(DebugCommand::Remove { id: ObjectId(99) }), stranger],
        15 => vec![debug(DebugCommand::Remove { id: ObjectId(1) })],
        _ => Vec::new(),
    }
}

/// `ARCHITECTURE.md > Commands, events and replays`: what the client records (`LoadedScenario` and
/// `RecordedInputs`) plays back in a bare `Sim` to the same state hash at every tick, whether debug commands are on
/// (the default) or off (`--no-debug`). The player's input that is not a command never reaches the sim.
#[test]
fn the_clients_recording_replays_to_the_same_state_hashes() {
    for sim_config in [SimConfig::DEBUG, SimConfig::default()] {
        let mut scenario = small_scenario();
        // A station waiting for its first delivery, so that the state changes by itself too.
        scenario.spawns.push(Spawn {
            settings: Some(SpawnSettings::SupplyDeliveryStation {
                starting_supplies: 0,
                delivery_size: 30,
                delivery_interval: 5,
            }),
            ..Spawn::new(
                ObjectKind::SupplyDeliveryStation,
                None,
                SpawnAt::Tile(TilePos::new(12, 2)),
            )
        });
        assert_eq!(scenario.validate(), vec![]);
        let mut config = AppConfig::new(scenario);
        config.sim_config = sim_config;
        let mut app = headless(config);
        let initial_ids = sim_ids(&app);
        let middle = view_middle(&app);
        let mut hashes = BTreeMap::new();
        for frame in 0..40 {
            app.world_mut()
                .resource_mut::<PendingCommands>()
                .0
                .extend(scripted_commands(frame));
            let mut input = input(&mut app);
            match frame {
                2 => input.keys.press(KeyCode::ArrowRight),
                4 => input.tap_key(KeyCode::KeyG),
                5 => input.keys.release(KeyCode::ArrowRight),
                6 => {
                    input.cursor = Some(middle);
                    input.keys.press(KeyCode::ControlLeft);
                    input.tap_key(KeyCode::Digit1);
                    input.mouse.press(MouseButton::Left);
                }
                8 => input.mouse.release(MouseButton::Left),
                10 => input.mouse.press(MouseButton::Right),
                _ => {}
            }
            app.update();
            let sim = app.world().resource::<SimRes>();
            hashes.insert(sim.tick(), sim.state_hash());
        }
        let sim = app.world().resource::<SimRes>().sim().clone();
        assert_eq!(sim.config(), sim_config);
        assert_eq!(sim.tick(), 39);
        if sim_config.debug {
            assert_eq!(sim_ids(&app).len(), initial_ids.len());
            assert!(!sim_ids(&app).contains(&ObjectId(1)) && sim_ids(&app).contains(&ObjectId(7)));
        } else {
            assert_eq!(sim_ids(&app), initial_ids, "every debug command was rejected");
        }

        let replay = Replay {
            scenario: app.world().resource::<LoadedScenario>().0.clone(),
            config: sim.config(),
            inputs: app.world().resource::<RecordedInputs>().ticks.clone(),
        };
        let mut bare = replay.start();
        loop {
            assert_eq!(
                Some(&bare.state_hash()),
                hashes.get(&bare.tick()),
                "tick {}",
                bare.tick()
            );
            if !replay.step(&mut bare) {
                break;
            }
        }
        assert_eq!(bare.tick(), sim.tick());
        assert_eq!(bare, sim);
    }
}

/// Advances real time by `fraction` of a tick in one frame. Returns the frame's interpolation factor and where
/// object 1 is drawn: on flat ground, the ground position it is drawn at.
fn advance(app: &mut App, fraction: f64) -> (f32, Vec2) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.insert_resource(TimeUpdateStrategy::ManualDuration(timestep.mul_f64(fraction)));
    app.update();
    let alpha = app.world().resource::<RenderAlpha>().0;
    let entity = app.world().resource::<ObjectVisuals>().by_id[&ObjectId(1)];
    let drawn = app.world().get::<Transform>(entity).unwrap().translation;
    (alpha, Vec2::new(drawn.x, -drawn.y))
}

/// `ARCHITECTURE.md > Client architecture > Visuals > Interpolation`: between ticks, an object is drawn between its
/// `prev_pos` and `pos`, by the part of the next tick that has passed (`Time<Fixed>::overstep_fraction`); a teleport
/// (`DebugCommand::SetPosition`) is drawn at its target at once, never on the way there.
#[test]
fn objects_are_drawn_between_their_tick_positions_except_after_a_teleport() {
    let mut scenario = scenarios::empty(16, 12);
    scenario.spawns.push(patch_at(2, 2));
    let mut app = headless_app(scenario);
    updates(&mut app, 2);
    // Nothing in the sim moves yet: make object 1 a free (not tile-based) object that moved in the last tick.
    let (from, to) = (Vec2::new(200.0, 300.0), Vec2::new(520.0, 140.0));
    let mut state = serde_json::to_value(app.world().resource::<SimRes>().sim()).unwrap();
    let object = &mut state["objects"][0];
    object["prev_pos"] = serde_json::to_value(from).unwrap();
    object["pos"] = serde_json::to_value(to).unwrap();
    object["placement"] = serde_json::Value::Null;
    let moved: Sim = serde_json::from_value(state).unwrap();
    app.world_mut().resource_mut::<SimRes>().replace(moved);
    let start = tick(&app);

    let close = |a: Vec2, b: Vec2| (a - b).length() < 1e-3;
    let (alpha, drawn) = advance(&mut app, 0.0);
    assert_eq!(alpha, 0.0);
    assert!(close(drawn, from), "{drawn} should be {from}");
    for (fraction, expected_alpha) in [(0.25, 0.25), (0.5, 0.75)] {
        let (alpha, drawn) = advance(&mut app, fraction);
        assert!((alpha - expected_alpha).abs() < 1e-6, "{alpha}");
        let expected = from.lerp(to, expected_alpha);
        assert!(close(drawn, expected), "{drawn} should be {expected}");
    }
    assert_eq!(tick(&app), start, "no tick ran yet");

    // The next tick teleports the object: drawn at the target in the frame of the tick, wherever the clock is.
    let target = Vec2::new(700.0, 600.0);
    app.world_mut()
        .resource_mut::<PendingCommands>()
        .0
        .push(debug(DebugCommand::SetPosition {
            id: ObjectId(1),
            pos: target,
        }));
    let (alpha, drawn) = advance(&mut app, 0.5);
    assert_eq!(tick(&app), start + 1);
    assert!((alpha - 0.25).abs() < 1e-6, "{alpha}");
    assert_eq!(drawn, target);
    let object = app.world().resource::<SimRes>().object(ObjectId(1)).unwrap().clone();
    assert_eq!((object.prev_pos, object.pos), (target, target));
    let (_, drawn) = advance(&mut app, 0.5);
    assert_eq!(drawn, target);
}
