//! (Slice M5-S7) The playtest menu at the game's end (design change 0414, `core/game_rules.md > Playtest menu
//! (temporary)`: it "opens when the game ends (Victory, Defeat or Draw, shown with it)", and "the game-end menu appears
//! over the stopped game"): under the outcome banner, Restart and Quit, and no Resume; F10 doesn't close it.

use bevy::ecs::message::Messages;
use space_crystals::platform::SurfaceSize;
use space_crystals::playtest_menu::{MenuButton, MenuLayout, MenuRoot, PlaytestMenu};

use super::*;

fn menu(app: &App) -> PlaytestMenu {
    *app.world().resource::<PlaytestMenu>()
}

fn menu_shown(app: &mut App) -> bool {
    let world = app.world_mut();
    let mut root = world.query_filtered::<&Visibility, With<MenuRoot>>();
    *root.single(world).unwrap() != Visibility::Hidden
}

/// The menu's layout on the surface, while it is open.
fn layout(app: &App) -> MenuLayout {
    let size = app.world().resource::<SurfaceSize>().logical();
    MenuLayout::of(menu(app), size).expect("the menu is open")
}

/// Clicks `button` of the menu that is open, for a frame.
fn click(app: &mut App, button: MenuButton) {
    let at = layout(app).button(button).expect("the menu has the button").center();
    let mut input = app.world_mut().resource_mut::<InjectedInput>();
    input.cursor = Some(at);
    input.mouse.press(MouseButton::Left);
    input.mouse.release(MouseButton::Left);
    app.update();
}

/// Design change 0414's Verification: "Win or lose a game: the menu appears with the result, offering Restart and
/// Quit." Player 1 wins: the banner says VICTORY, the menu shows under it, Restart and Quit; F10 leaves it open; Quit
/// closes the game.
#[test]
fn at_the_game_end_the_menu_shows_with_the_result() {
    let mut app = game_as(AppConfig::new(duel()), P1);
    run_until(&mut app, 3);
    assert_eq!(menu(&app), PlaytestMenu::Closed);
    remove(&mut app, 2);
    app.update();
    assert_eq!(banner(&mut app), shown("VICTORY", "The game is over"));
    assert_eq!(menu(&app), PlaytestMenu::GameEnd);
    assert!(menu_shown(&mut app));
    let layout = layout(&app);
    let buttons: Vec<MenuButton> = layout.buttons.iter().map(|(b, _)| *b).collect();
    assert_eq!(buttons, [MenuButton::Restart, MenuButton::Quit], "no Resume");
    assert_eq!(layout.title, None, "the banner tells the result");
    let size = app.world().resource::<SurfaceSize>().logical();
    assert!(
        layout.panel.min.y > size.y * 0.5,
        "under the banner, in the middle of the world view"
    );

    app.world_mut().resource_mut::<InjectedInput>().tap_key(KeyCode::F10);
    app.update();
    assert_eq!(menu(&app), PlaytestMenu::GameEnd, "F10 leaves it open");
    click(&mut app, MenuButton::Quit);
    let exits: Vec<AppExit> = app
        .world()
        .resource::<Messages<AppExit>>()
        .iter_current_update_messages()
        .cloned()
        .collect();
    assert_eq!(exits, [AppExit::Success]);
}

/// Defeated, player 2 sees DEFEAT with the same menu; Restart starts the game over from 0:00: the banner and the menu
/// go, the players' Generators stand again, and the game runs on.
#[test]
fn restart_from_the_game_end_starts_the_game_over() {
    let mut app = game_as(AppConfig::new(duel()), P2);
    run_until(&mut app, 3);
    remove(&mut app, 2);
    app.update();
    assert_eq!(banner(&mut app), shown("DEFEAT", "The game is over"));
    assert_eq!(menu(&app), PlaytestMenu::GameEnd);

    click(&mut app, MenuButton::Restart);
    assert!(tick(&app) <= 1, "from 0:00");
    let sim = app.world().resource::<SimRes>();
    assert_eq!(sim.outcome(), None);
    assert!(sim.object(ObjectId(2)).is_some(), "player 2's Generator stands again");
    assert_eq!(menu(&app), PlaytestMenu::Closed);
    app.update();
    assert_eq!(banner(&mut app), None);
    assert!(!menu_shown(&mut app));
    run_until(&mut app, 10);
}
