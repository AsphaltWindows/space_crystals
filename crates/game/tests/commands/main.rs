//! Headless tests of the command flow (`systems/controls/selection_and_hud.md > CommandPanel`,
//! `systems/controls/selection_and_hud.md > DefaultState`, `systems/controls/selection_and_hud.md >
//! RightClickResolution`, `systems/controls/selection_and_hud.md > AwaitingTarget`,
//! `systems/controls/selection_and_hud.md > PointerDisplayType`, `systems/controls/selection_and_hud.md >
//! CommandIndicators`; `systems/controls/commands.md > Unit Command`). Every input arrives through `InjectedInput`, the
//! virtual keyboard and mouse, and goes through the input layer's bindings like a player's would; each `App::update` is
//! one tick and 1/16 s of real time (`ARCHITECTURE.md > Client architecture > Client tests`). The orders given are read
//! back from the recorded inputs, as the sim received them. The fog of war is off (`--no-fog`), so that everything is
//! drawn and can be clicked; what the fog hides from the command flow is tested in `fog.rs`. The sim takes an order on
//! an object only while the player sees it, so player 1 has scouts off screen, in sight of every target; and
//! auto-targeting is off (a debug command), so that nobody fights while the orders are given.
//!
//! The production UI of slice M2-H1 (`factions/gdo/structures/barracks.md > ObjectInterfaceState[Barracks]`,
//! `systems/controls/selection_and_hud.md > Production Building Default Right-Click`) plays on the same field with
//! player 1's Barracks added ([`production_scenario`]): `production` for Train and Cancel Production, `rally` for the
//! rally points and their indicators.
//!
//! The Colonists' portals (slice M3-D) show the same production panel, built from their recipes: `portals`, on a
//! Colonists field of their own. The Legionnaires' Weapon Conversion (slice M3-B), R: `convert`, on another.
//!
//! Remembered targets with the fog of war on, rally points by the units' own default commands and a right-click on a
//! selected unit (slice M3-R): `remembered`.
//!
//! The construction UI of slice M2-H2 (the Deployment Center's build menu, the Extraction Facility's plate, and
//! AwaitingPlacement with its ghost and the build area overlay) plays on a base of its own: `construction`.
//!
//! The Colonists' Deploy and Warp In (slice M3-J1: the Drone's and the Beacon's panels, their ghosts, the lingering
//! ghosts, whatever transforms showing only Cancel, and what a greyed-out button says): `colonists`, on a Colonists
//! field.
//!
//! The Agent's UI (slice M4-J1: its panel, Build Tunnel's price and ghost, Shift placing, sites, lingering Tunnel
//! ghosts, the Enter and Drop Off pointers, a carried Supply): `agent_ui`, on a Syndicate field.

mod agent_ui;
mod click_allowance;
mod colonists;
mod construction;
mod convert;
mod indicators;
mod panel;
mod portals;
mod production;
mod production_field;
mod rally;
mod remembered;
mod right_click;
mod riot_shield;
mod shield_orders;
mod shift_targeting;
mod supplies;
mod syndicate_panels;
mod targeting;
mod tunnel_network;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use sc_sim::{
    Command, DebugCommand, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, ProductionCommand, Scenario,
    SimEvent, SlotId, Spawn, SpawnAt, Target, TilePos, scenarios,
};
use space_crystals::camera::{CameraView, ViewportInsets, viewport_rect, visible_extent_su};
use space_crystals::command_ui::cursor::PointerLook;
use space_crystals::command_ui::{InterfaceState, PointerDisplay};
use space_crystals::hud::layout::CommandGrid;
use space_crystals::input::{CommandSlot, InjectedInput};
use space_crystals::platform::{DisplayMode, SurfaceSize};
use space_crystals::selection::ControlStates;
use space_crystals::selection::markers::DragBoxEdge;
use space_crystals::sim_driver::{LocalPlayer, PendingCommands, RecordedInputs, SimRes};
use space_crystals::{AppConfig, build_app};

use production_field::*;

/// Where the camera looks in these tests (view space): the world view then spans x 704 to 2496 and y 536 to 1264.
const VIEW_CENTER: Vec2 = Vec2::new(1600.0, 900.0);

// The objects of [`scenario`], by id. Player 1's are on the left, player 2's on the right, far enough apart.
/// Player 1's Peacekeepers on screen, and its Guards.
const OWN_PKS: [u32; 3] = [1, 2, 5];
const OWN_GUARDS: [u32; 2] = [3, 4];
/// Player 2's units.
const ENEMY_PK: u32 = 6;
const ENEMY_GUARD: u32 = 7;
/// A neutral Space Crystals patch and a Supply Delivery Station.
const PATCH: u32 = 8;
const STATION: u32 = 11;

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
        // Player 1's scouts, off screen: in sight of the enemy units, of the station and of the patch.
        12 => Vec2::new(2275.0, 500.0),
        13 => Vec2::new(1790.0, 500.0),
        14 => Vec2::new(1632.0, 1300.0),
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
        unit(Peacekeeper, 1, 12),
        unit(Peacekeeper, 1, 13),
        unit(Peacekeeper, 1, 14),
    ]);
    assert_eq!(scenario.validate(), vec![]);
    scenario
}

/// A headless game of [`scenario`] as player 1, debug commands on, the camera at [`VIEW_CENTER`], without the fog
/// of war and without auto-targeting.
fn game() -> App {
    game_of(scenario())
}

/// A headless game of `scenario`, as [`game`] sets it up.
fn game_of(scenario: Scenario) -> App {
    let mut config = AppConfig::new(scenario);
    config.display = DisplayMode::Headless;
    config.qa.no_fog = true;
    let mut app = build_app(config);
    app.insert_resource(TimeUpdateStrategy::FixedTimesteps(1));
    app.finish();
    app.cleanup();
    app.world_mut().resource_mut::<PendingCommands>().0.push(PlayerCommand {
        player: PlayerId(1),
        command: Command::Debug(DebugCommand::SetAutoTargeting { on: false }),
    });
    app.update();
    app.world_mut().resource_mut::<CameraView>().center = VIEW_CENTER;
    updates(&mut app, 2);
    app
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
    let center = app.world().resource::<CameraView>().center;
    ((view - center) / su_per_pixel + position.as_vec2() + viewport.as_vec2() / 2.0) / size.scale_factor
}

/// A click of `button` at the cursor position `screen`, with the modifier keys `modifiers` held: the button goes
/// down in one frame and up in the next. Then a [`pause`].
fn click_with(app: &mut App, button: MouseButton, screen: Vec2, modifiers: &[KeyCode]) {
    press_keys(app, modifiers);
    input(app).cursor = Some(screen);
    input(app).mouse.press(button);
    app.update();
    input(app).mouse.release(button);
    app.update();
    release_keys(app, modifiers);
    pause(app);
}

/// The pointer's PointerDisplayType once the cursor is over the view point `view`, a frame later.
fn pointer_over(app: &mut App, view: Vec2) -> Option<PointerDisplay> {
    let screen = screen_at(app, view);
    input(app).cursor = Some(screen);
    app.update();
    app.world().resource::<PointerLook>().display
}

/// Submits `command` in `player`'s name, for the next tick.
fn submit(app: &mut App, player: PlayerId, command: Command) {
    app.world_mut()
        .resource_mut::<PendingCommands>()
        .0
        .push(PlayerCommand { player, command });
}

/// A left click on the map at the view point `view`.
fn click(app: &mut App, view: Vec2, modifiers: &[KeyCode]) {
    let screen = screen_at(app, view);
    click_with(app, MouseButton::Left, screen, modifiers);
}

/// A right click on the map at the view point `view`.
fn right_click(app: &mut App, view: Vec2, modifiers: &[KeyCode]) {
    let screen = screen_at(app, view);
    click_with(app, MouseButton::Right, screen, modifiers);
}

/// A left click on the command panel's button of `slot`, found by the panel's geometry.
fn click_button(app: &mut App, slot: CommandSlot, modifiers: &[KeyCode]) {
    let surface = app.world().resource::<SurfaceSize>().logical();
    let screen = CommandGrid::on(surface).slot_rect(slot).center();
    click_with(app, MouseButton::Left, screen, modifiers);
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

/// Whether the selection box is drawn.
fn box_drawn(app: &mut App) -> bool {
    let mut query = app.world_mut().query::<(&DragBoxEdge, &Visibility)>();
    query.iter(app.world()).any(|(_, v)| *v != Visibility::Hidden)
}

/// Taps `key` while the modifier keys `modifiers` are down; two frames.
fn tap(app: &mut App, modifiers: &[KeyCode], key: KeyCode) {
    press_keys(app, modifiers);
    input(app).tap_key(key);
    app.update();
    release_keys(app, modifiers);
    app.update();
}

/// Selects player 1's Peacekeepers 1 and 2 and Guards 3 and 4 with a box.
fn select_mixed(app: &mut App) {
    drag(app, Vec2::new(850.0, 760.0), Vec2::new(1400.0, 840.0));
    assert_eq!(selected(app), [1, 2, 3, 4]);
}

/// Selects the objects of `ids` with a box around them, which must select exactly them.
fn select_box(app: &mut App, ids: &[u32]) {
    let points: Vec<Vec2> = ids.iter().map(|&id| at(id)).collect();
    select_around(app, &points, ids);
}

/// Selects the objects of `ids` with a box around where they are now, which must select exactly them.
fn select_units(app: &mut App, ids: &[u32]) {
    let points: Vec<Vec2> = ids.iter().map(|&id| pos(app, id)).collect();
    select_around(app, &points, ids);
}

/// Drags a box around `points`, which must select exactly `ids`.
fn select_around(app: &mut App, points: &[Vec2], ids: &[u32]) {
    let min = points.iter().copied().reduce(Vec2::min).unwrap() - 30.0;
    let max = points.iter().copied().reduce(Vec2::max).unwrap() + 30.0;
    drag(app, min, max);
    assert_eq!(selected(app), ids);
}

/// Where object `id` is now, as the sim has it (the maps of these tests are flat: where it is drawn).
fn pos(app: &App, id: u32) -> Vec2 {
    app.world()
        .resource::<SimRes>()
        .object(ObjectId(id))
        .expect("in the game")
        .pos
}

/// Spawns a `kind` of `owner` at the point `at` (a debug command) and returns its id, two frames later.
fn spawn(app: &mut App, kind: ObjectKind, owner: Option<PlayerId>, at: Vec2) -> u32 {
    let at = SpawnAt::Point(at);
    submit(
        app,
        PlayerId(1),
        Command::Debug(DebugCommand::Spawn {
            kind,
            owner,
            at,
            orientation: Default::default(),
        }),
    );
    updates(app, 2);
    let sim = app.world().resource::<SimRes>();
    let spawned = sim.objects().last().expect("spawned");
    assert_eq!((spawned.kind, spawned.owner), (kind, owner));
    spawned.id.0
}

/// Runs `n` ticks, one per frame, and returns the sim's events of each, in order.
fn run_collecting(app: &mut App, n: usize) -> Vec<SimEvent> {
    let mut events = Vec::new();
    for _ in 0..n {
        let before = app.world().resource::<SimRes>().tick();
        app.update();
        let sim = app.world().resource::<SimRes>();
        assert_eq!(sim.tick(), before + 1, "one tick per frame");
        events.extend(sim.events().iter().cloned());
    }
    events
}

fn local_player(app: &App) -> PlayerId {
    app.world().resource::<LocalPlayer>().0
}

/// The local player's selection, in id order.
fn selected(app: &App) -> Vec<u32> {
    let states = app.world().resource::<ControlStates>();
    states.selection(local_player(app)).ids().map(|id| id.0).collect()
}

/// The local player's ObjectInterfaceState.
fn interface(app: &App) -> InterfaceState {
    app.world().resource::<ControlStates>().of(local_player(app)).interface
}

/// An order as the sim received it.
#[derive(Clone, Debug, PartialEq)]
struct Given {
    player: PlayerId,
    units: Vec<u32>,
    kind: OrderKind,
    queue: bool,
}

/// Player 1's order of `kind` to `units`, queued or not.
fn given(units: &[u32], kind: OrderKind, queue: bool) -> Given {
    Given {
        player: PlayerId(1),
        units: units.to_vec(),
        kind,
        queue,
    }
}

/// Every order given so far: the recorded ticks' and those waiting for the next tick.
fn orders(app: &App) -> Vec<Given> {
    let recorded = app.world().resource::<RecordedInputs>();
    let pending = app.world().resource::<PendingCommands>();
    let commands = recorded.ticks.iter().flat_map(|t| t.commands.iter()).chain(&pending.0);
    commands
        .filter_map(|c| match &c.command {
            Command::Order { units, kind, queue } => Some(Given {
                player: c.player,
                units: units.iter().map(|id| id.0).collect(),
                kind: *kind,
                queue: *queue,
            }),
            Command::Ability { .. }
            | Command::Production { .. }
            | Command::Construction { .. }
            | Command::Tunnel { .. }
            | Command::Debug(_) => None,
        })
        .collect()
}

/// Every ability used so far, as the sim received them (like [`orders`]): player, units, ability.
fn abilities(app: &App) -> Vec<(PlayerId, Vec<u32>, sc_sim::Ability)> {
    let recorded = app.world().resource::<RecordedInputs>();
    let pending = app.world().resource::<PendingCommands>();
    let commands = recorded.ticks.iter().flat_map(|t| t.commands.iter()).chain(&pending.0);
    commands
        .filter_map(|c| match &c.command {
            Command::Ability { units, ability } => Some((c.player, units.iter().map(|id| id.0).collect(), *ability)),
            Command::Order { .. }
            | Command::Production { .. }
            | Command::Construction { .. }
            | Command::Tunnel { .. }
            | Command::Debug(_) => None,
        })
        .collect()
}

/// The point of a point order (Move to a point, AttackMove, Patrol).
fn point_of(kind: OrderKind) -> Vec2 {
    match kind {
        OrderKind::Move(Target::Point(p)) | OrderKind::AttackMove(p) | OrderKind::Patrol(p) => p,
        other => panic!("{other:?} has no point"),
    }
}

/// Whether `a` and `b` are the same point, give or take the cursor's rounding.
fn near(a: Vec2, b: Vec2) -> bool {
    a.distance(b) < 0.5
}

fn object_orders(app: &App, id: u32) -> sc_sim::Orders {
    let sim = app.world().resource::<SimRes>();
    sim.object(ObjectId(id)).unwrap().orders.clone().unwrap()
}

/// A slot by its row and column.
fn slot(row: u8, column: u8) -> CommandSlot {
    CommandSlot::new(row, column).unwrap()
}
