//! Headless selection tests (`systems/controls/selection_and_hud.md`, `ARCHITECTURE.md > Client architecture >
//! Selection`). Every input arrives through `InjectedInput`, the virtual keyboard and mouse, and goes through the
//! input layer's bindings like a player's would; each `App::update` is one tick and 1/16 s of real time
//! (`ARCHITECTURE.md > Client architecture > Client tests`).

mod groups;
mod map;
mod panels;
mod state;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use sc_sim::{ObjectId, ObjectKind, PlayerId, Scenario, SimConfig, SlotId, Spawn, SpawnAt, TilePos, scenarios};
use space_crystals::camera::{CameraView, ViewportInsets, viewport_rect, visible_extent_su};
use space_crystals::input::{ControlGroup, InjectedInput};
use space_crystals::platform::{DisplayMode, SurfaceSize};
use space_crystals::selection::ControlStates;
use space_crystals::sim_driver::{LocalPlayer, SimRes};
use space_crystals::{AppConfig, build_app};

/// Where the camera looks in these tests (view space): the world view then spans x 704 to 2496 and y 536 to 1264.
const VIEW_CENTER: Vec2 = Vec2::new(1600.0, 900.0);

// The objects of [`scenario`], by id. Player 1's are on the left, player 2's on the right, far enough apart.
/// Player 1's Peacekeepers on screen.
const OWN_PKS: [u32; 3] = [1, 2, 5];
/// Player 1's Guards.
const OWN_GUARDS: [u32; 2] = [3, 4];
/// Player 2's units.
const ENEMY_PK: u32 = 6;
const ENEMY_GUARD: u32 = 7;
const ENEMY_PK_2: u32 = 10;
/// A neutral Space Crystals patch and a Supply Delivery Station.
const PATCH: u32 = 8;
const STATION: u32 = 11;
/// Player 1's Peacekeeper off screen, to the east.
const FAR_PK: u32 = 9;

/// Where object `id` of [`scenario`] is, and is drawn (the map is flat).
fn at(id: u32) -> Vec2 {
    match id {
        1 => Vec2::new(900.0, 800.0),
        2 => Vec2::new(1050.0, 800.0),
        3 => Vec2::new(1200.0, 800.0),
        4 => Vec2::new(1350.0, 800.0),
        5 => Vec2::new(900.0, 1100.0),
        6 => Vec2::new(2200.0, 800.0),
        7 => Vec2::new(2350.0, 800.0),
        8 => Vec2::new(25.5, 16.5) * 64.0,
        9 => Vec2::new(3000.0, 800.0),
        10 => Vec2::new(2200.0, 1100.0),
        11 => Vec2::new(28.0, 11.0) * 64.0,
        _ => panic!("no object {id}"),
    }
}

/// A flat 64 × 48 map with two players' units, a patch and a station (ids as above).
fn scenario() -> Scenario {
    use ObjectKind::{CrystalPatch, Guard, Peacekeeper, SupplyDeliveryStation};
    let mut scenario = scenarios::empty(64, 48);
    let unit = |kind, slot, id| Spawn::new(kind, Some(SlotId(slot)), SpawnAt::Point(at(id)));
    scenario.map.spawns.extend([
        unit(Peacekeeper, 1, 1),
        unit(Peacekeeper, 1, 2),
        unit(Guard, 1, 3),
        unit(Guard, 1, 4),
        unit(Peacekeeper, 1, 5),
        unit(Peacekeeper, 2, 6),
        unit(Guard, 2, 7),
        Spawn::new(CrystalPatch, None, SpawnAt::Tile(TilePos::new(25, 16))),
        unit(Peacekeeper, 1, 9),
        unit(Peacekeeper, 2, 10),
        Spawn::new(SupplyDeliveryStation, None, SpawnAt::Tile(TilePos::new(27, 10))),
    ]);
    assert_eq!(scenario.validate(), vec![]);
    scenario
}

/// A headless game of `scenario` as player 1, debug commands on or off, the camera at [`VIEW_CENTER`].
fn game_of(scenario: Scenario, debug: bool) -> App {
    let mut config = AppConfig::new(scenario);
    config.display = DisplayMode::Headless;
    if !debug {
        config.scenario.setup.config = SimConfig::default();
    }
    let mut app = build_app(config);
    app.insert_resource(TimeUpdateStrategy::FixedTimesteps(1));
    app.finish();
    app.cleanup();
    app.update();
    app.world_mut().resource_mut::<CameraView>().center = VIEW_CENTER;
    updates(&mut app, 2);
    app
}

/// [`game_of`] [`scenario`], debug commands on.
fn game() -> App {
    game_of(scenario(), true)
}

fn updates(app: &mut App, n: usize) {
    for _ in 0..n {
        app.update();
    }
}

/// Long enough (half a second) that the next click or key press doesn't make a double-click or a double tap.
fn pause(app: &mut App) {
    updates(app, 8);
}

/// The virtual keyboard and mouse.
fn input(app: &mut App) -> Mut<'_, InjectedInput> {
    app.world_mut().resource_mut::<InjectedInput>()
}

fn press_keys(app: &mut App, keys: &[KeyCode]) {
    for &key in keys {
        input(app).keys.press(key);
    }
}

fn release_keys(app: &mut App, keys: &[KeyCode]) {
    for &key in keys {
        input(app).keys.release(key);
    }
}

/// The cursor position (logical pixels) that points at the view point `view`, with the camera where it is now.
fn screen_at(app: &App, view: Vec2) -> Vec2 {
    let size = *app.world().resource::<SurfaceSize>();
    let insets = *app.world().resource::<ViewportInsets>();
    let (position, viewport) = viewport_rect(size, insets);
    let su_per_pixel = visible_extent_su(viewport, size.physical.x, 1.0).x / viewport.x as f32;
    let center = camera_center(app);
    ((view - center) / su_per_pixel + position.as_vec2() + viewport.as_vec2() / 2.0) / size.scale_factor
}

/// `clicks` left clicks at the cursor position `screen` in a row, with the modifier keys `modifiers` held: the
/// button goes down in one frame and up in the next. Then a [`pause`].
fn click_screen(app: &mut App, screen: Vec2, modifiers: &[KeyCode], clicks: usize) {
    press_keys(app, modifiers);
    input(app).cursor = Some(screen);
    for _ in 0..clicks {
        input(app).mouse.press(MouseButton::Left);
        app.update();
        input(app).mouse.release(MouseButton::Left);
        app.update();
    }
    release_keys(app, modifiers);
    pause(app);
}

/// A left click on the map at the view point `view`, with the modifier keys `modifiers` held.
fn click(app: &mut App, view: Vec2, modifiers: &[KeyCode]) {
    let screen = screen_at(app, view);
    click_screen(app, screen, modifiers, 1);
}

/// A double-click on the map at the view point `view`.
fn double_click(app: &mut App, view: Vec2) {
    let screen = screen_at(app, view);
    click_screen(app, screen, &[], 2);
}

/// A box dragged with the left button from the view point `from` to `to`: down, moved, up, a frame each.
fn drag(app: &mut App, from: Vec2, to: Vec2) {
    let (from, to) = (screen_at(app, from), screen_at(app, to));
    input(app).cursor = Some(from);
    input(app).mouse.press(MouseButton::Left);
    app.update();
    input(app).cursor = Some(to);
    app.update();
    input(app).mouse.release(MouseButton::Left);
    app.update();
    pause(app);
}

/// Taps `key` while the modifier keys `modifiers` are down; two frames.
fn tap(app: &mut App, modifiers: &[KeyCode], key: KeyCode) {
    press_keys(app, modifiers);
    input(app).tap_key(key);
    app.update();
    release_keys(app, modifiers);
    app.update();
}

fn local_player(app: &App) -> PlayerId {
    app.world().resource::<LocalPlayer>().0
}

/// The local player's selection, in id order.
fn selected(app: &App) -> Vec<u32> {
    let states = app.world().resource::<ControlStates>();
    states.selection(local_player(app)).ids().map(|id| id.0).collect()
}

/// The ids of the local player's ActiveGroup.
fn active_group(app: &App) -> Vec<u32> {
    let states = app.world().resource::<ControlStates>();
    let group = states.selection(local_player(app)).active_group();
    group.map_or_else(Vec::new, |g| g.instances.iter().map(|id| id.0).collect())
}

/// The local player's control group `n`.
fn control_group(app: &App, n: u8) -> Vec<u32> {
    let states = app.world().resource::<ControlStates>();
    let group = ControlGroup::new(n).unwrap();
    states
        .of(local_player(app))
        .groups
        .get(group)
        .iter()
        .map(|id| id.0)
        .collect()
}

fn camera_center(app: &App) -> Vec2 {
    app.world().resource::<CameraView>().center
}

fn object_exists(app: &App, id: u32) -> bool {
    app.world().resource::<SimRes>().object(ObjectId(id)).is_some()
}

/// Sorted ids, for comparing with [`selected`].
fn ids<const N: usize>(mut ids: [u32; N]) -> Vec<u32> {
    ids.sort_unstable();
    ids.to_vec()
}
