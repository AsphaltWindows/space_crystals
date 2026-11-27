//! Headless client tests: no window and no GPU. Every `App::update` runs `FixedUpdate` exactly once
//! (`TimeUpdateStrategy::FixedTimesteps(1)`), and real time advances by one tick (1/16 s) per update; the first
//! update only starts the clock, so the tests assert on `Sim::tick()`, never on the number of updates
//! (`ARCHITECTURE.md > Client architecture > Client tests`). Input arrives through `InjectedInput`, a virtual
//! keyboard and mouse (`ARCHITECTURE.md > Client architecture > Input`).

mod camera;
mod input;
mod qa;
mod recording;
mod visuals;
mod window;

use std::collections::{BTreeMap, BTreeSet};

use bevy::ecs::message::{Message, Messages};
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use sc_sim::{
    Command, CornerPos, DebugCommand, ObjectId, ObjectKind, PlayerCommand, PlayerId, Scenario, SimEvent, Spawn,
    SpawnAt, Tick, TilePos, scenarios,
};
use space_crystals::camera::{CameraView, ViewportInsets, cursor_to_view, viewport_rect};
use space_crystals::input::{ActionState, InjectedInput, InputAction};
use space_crystals::object_render::ObjectVisual;
use space_crystals::platform::{DisplayMode, SurfaceSize};
use space_crystals::sim_driver::{PendingCommands, RecordedInputs, SimControl, SimEventMessage, SimRes};
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
        .map
        .spawns
        .extend([patch_at(2, 2), patch_at(3, 2), patch_at(10, 9)]);
    scenario.map.spawns.push(Spawn::new(
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
    scenario.map.terrain.set_corner_heights(&targets);
    scenario.map.spawns.push(patch_at(16, 12));
    assert_eq!(scenario.validate(), vec![]);
    scenario
}

/// A 64 × 48 map whose camera starts at its center, far from the edges where the view would be clamped.
fn centered_scenario() -> Scenario {
    let mut scenario = scenarios::empty(64, 48);
    scenario.map.slots[0].camera_start = Some(Vec2::new(32.0, 24.0) * 64.0);
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
        orientation: Default::default(),
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
