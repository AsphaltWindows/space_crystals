//! The playtest menu (slice M5-S7; design change 0414, `core/game_rules.md > Playtest menu (temporary)`), headless,
//! one tick per update (`docs/architecture/client-architecture-placement-and-tests.md > Client architecture > Client tests`), its buttons clicked through the
//! virtual mouse where the menu's layout puts them (`MenuLayout`): F10 opens it and pauses the game, F10 again or
//! Resume continues, and Quit exits (here); Restart starts the game over as it was launched, as a fresh launch of it
//! would, with every piece of the client's state, and the new game records to a file of its own (`restart`). The menu
//! at the game's end is `game_end::menu`'s, and the input it holds `selection::menu`'s.

mod restart;

use bevy::ecs::message::Messages;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use sc_sim::testing::ScenarioBuilder;
use sc_sim::{Command, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, SpawnAt, Target, Tick, TilePos};
use space_crystals::input::{InjectedInput, Modal};
use space_crystals::object_render::ObjectVisuals;
use space_crystals::platform::{DisplayMode, SurfaceSize};
use space_crystals::playtest_menu::{MenuButton, MenuLayout, MenuRoot, PlaytestMenu};
use space_crystals::sim_driver::{PendingCommands, SimControl, SimRes};
use space_crystals::{AppConfig, build_app};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
/// Player 1's Peacekeeper in [`duel`].
const PK: u32 = 3;

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

fn centre(x: i32, y: i32) -> Vec2 {
    TilePos::new(x, y).center()
}

/// A flat 24 × 12 map: player 1's Generator 1 and Peacekeeper 3 in the west, player 2's Generator 2 and Peacekeeper 4
/// in the east.
fn duel() -> AppConfig {
    let scenario = ScenarioBuilder::new(&vec![".".repeat(24); 12].join("\n"))
        .spawn(ObjectKind::Generator, Some(P1), tile(1, 1))
        .spawn(ObjectKind::Generator, Some(P2), tile(20, 1))
        .spawn(ObjectKind::Peacekeeper, Some(P1), SpawnAt::Point(centre(4, 7)))
        .spawn(ObjectKind::Peacekeeper, Some(P2), SpawnAt::Point(centre(19, 7)))
        .build();
    AppConfig::new(scenario)
}

/// A headless app of `config`, after its first update (tick 0).
fn headless(mut config: AppConfig) -> App {
    config.display = DisplayMode::Headless;
    let mut app = build_app(config);
    app.insert_resource(TimeUpdateStrategy::FixedTimesteps(1));
    app.finish();
    app.cleanup();
    app.update();
    app
}

fn tick(app: &App) -> Tick {
    app.world().resource::<SimRes>().tick()
}

fn run_until(app: &mut App, until: Tick) {
    while tick(app) < until {
        app.update();
    }
}

fn updates(app: &mut App, n: usize) {
    for _ in 0..n {
        app.update();
    }
}

fn input(app: &mut App) -> Mut<'_, InjectedInput> {
    app.world_mut().resource_mut::<InjectedInput>()
}

fn menu(app: &App) -> PlaytestMenu {
    *app.world().resource::<PlaytestMenu>()
}

/// Taps F10, the playtest menu's key, for a frame.
fn f10(app: &mut App) {
    input(app).tap_key(KeyCode::F10);
    app.update();
}

/// Clicks `button` of the menu that is open, at its middle, for a frame.
fn click(app: &mut App, button: MenuButton) {
    let size = app.world().resource::<SurfaceSize>().logical();
    let layout = MenuLayout::of(menu(app), size).expect("the menu is open");
    let at = layout.button(button).expect("the menu has the button").center();
    let mut input = input(app);
    input.cursor = Some(at);
    input.mouse.press(MouseButton::Left);
    input.mouse.release(MouseButton::Left);
    app.update();
}

/// Whether the menu's nodes show.
fn menu_shown(app: &mut App) -> bool {
    let world = app.world_mut();
    let mut root = world.query_filtered::<&Visibility, With<MenuRoot>>();
    *root.single(world).unwrap() != Visibility::Hidden
}

fn position(app: &App, id: u32) -> Vec2 {
    app.world().resource::<SimRes>().object(ObjectId(id)).unwrap().pos
}

/// Where object `id`'s visual is drawn.
fn drawn(app: &App, id: u32) -> Vec3 {
    let entity = app.world().resource::<ObjectVisuals>().by_id[&ObjectId(id)];
    app.world().get::<Transform>(entity).unwrap().translation
}

/// Orders player 1's Peacekeeper across the map, to tile (20, 7).
fn send_peacekeeper(app: &mut App) {
    app.world_mut().resource_mut::<PendingCommands>().0.push(PlayerCommand {
        player: P1,
        command: Command::Order {
            units: vec![ObjectId(PK)],
            kind: OrderKind::Move(Target::Point(centre(20, 7))),
            queue: false,
        },
    });
}

fn exits(app: &App) -> Vec<AppExit> {
    let messages = app.world().resource::<Messages<AppExit>>();
    messages.iter_current_update_messages().cloned().collect()
}

/// Design change 0414's Verification: "Press F10 mid-game: the menu appears and the game is paused (units frozen); F10
/// or Resume: play continues." While it is open the tick doesn't advance over 60 frames and the moving Peacekeeper
/// stays where it is, drawn there; the menu holds the input (`Modal`). F10 closes it, and Resume does too.
#[test]
fn f10_pauses_the_game_until_f10_again_or_resume() {
    let mut app = headless(duel());
    send_peacekeeper(&mut app);
    run_until(&mut app, 10);
    assert!(!menu_shown(&mut app));
    let moving = position(&app, PK);
    app.update();
    assert_ne!(position(&app, PK), moving, "the Peacekeeper walks");

    f10(&mut app);
    assert_eq!(menu(&app), PlaytestMenu::Paused);
    assert!(menu_shown(&mut app));
    assert!(app.world().resource::<SimControl>().paused);
    assert_eq!(*app.world().resource::<Modal>(), Modal(true));
    let (paused_at, frozen, drawn_at) = (tick(&app), position(&app, PK), drawn(&app, PK));
    updates(&mut app, 60);
    assert_eq!(tick(&app), paused_at, "nothing advances");
    assert_eq!(position(&app, PK), frozen, "units frozen");
    assert_eq!(drawn(&app, PK), drawn_at, "and drawn where they stand");
    let layout = MenuLayout::of(PlaytestMenu::Paused, app.world().resource::<SurfaceSize>().logical()).unwrap();
    let buttons: Vec<MenuButton> = layout.buttons.iter().map(|(b, _)| *b).collect();
    assert_eq!(buttons, [MenuButton::Resume, MenuButton::Restart, MenuButton::Quit]);

    f10(&mut app);
    assert_eq!(menu(&app), PlaytestMenu::Closed, "F10 again");
    assert!(!app.world().resource::<SimControl>().paused);
    updates(&mut app, 4);
    assert!(tick(&app) > paused_at, "play continues");
    assert_ne!(position(&app, PK), frozen);
    app.update();
    assert!(!menu_shown(&mut app));
    assert_eq!(*app.world().resource::<Modal>(), Modal(false));

    f10(&mut app);
    let paused_at = tick(&app);
    updates(&mut app, 5);
    assert_eq!(tick(&app), paused_at);
    click(&mut app, MenuButton::Resume);
    assert_eq!(menu(&app), PlaytestMenu::Closed, "Resume");
    updates(&mut app, 4);
    assert!(tick(&app) > paused_at, "play continues");
}

/// Design change 0414's Verification: "Quit: the game closes": the click sends `AppExit::Success`, as closing the
/// window does.
#[test]
fn quit_closes_the_game() {
    let mut app = headless(duel());
    run_until(&mut app, 3);
    f10(&mut app);
    assert_eq!(exits(&app), []);
    click(&mut app, MenuButton::Quit);
    assert_eq!(exits(&app), [AppExit::Success]);
}

/// A click beside the buttons, on the panel or off it, does nothing: the menu stays open, the game paused.
#[test]
fn a_click_beside_the_buttons_does_nothing() {
    let mut app = headless(duel());
    run_until(&mut app, 3);
    f10(&mut app);
    let size = app.world().resource::<SurfaceSize>().logical();
    let layout = MenuLayout::of(PlaytestMenu::Paused, size).unwrap();
    for at in [layout.panel.min + Vec2::splat(2.0), Vec2::new(5.0, size.y / 2.0)] {
        let mut input = input(&mut app);
        input.cursor = Some(at);
        input.mouse.press(MouseButton::Left);
        input.mouse.release(MouseButton::Left);
        app.update();
        assert_eq!(menu(&app), PlaytestMenu::Paused, "{at}");
    }
    assert_eq!(tick(&app), 3);
    assert_eq!(exits(&app), []);
}
