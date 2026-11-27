//! (Slice M5-S7) While the playtest menu is open, game input is blocked (design change 0414; the brief: "no
//! selection, no commands, no camera keys"): the menu holds the input (`space_crystals::input::Modal`). Clicks and
//! boxes select nothing, the control group keys and Tab do nothing, a right-click and a command key give no order, and
//! the arrow keys and the middle-button grab leave the camera where it is; the screen edge's pans, which only a
//! focused window gives, are held actions like the arrow keys' and stop alike. Closed again, the same input works.

use space_crystals::playtest_menu::PlaytestMenu;
use space_crystals::sim_driver::{PendingCommands, RecordedInputs};

use super::*;

/// Open ground in the world view, where a right-click is a Move.
const GROUND: Vec2 = Vec2::new(1500.0, 650.0);

fn menu(app: &App) -> PlaytestMenu {
    *app.world().resource::<PlaytestMenu>()
}

/// How many commands the client gave so far: those the sim applied, and those waiting for its next step.
fn commands(app: &App) -> usize {
    let applied = app
        .world()
        .resource::<RecordedInputs>()
        .ticks
        .iter()
        .map(|t| t.commands.len())
        .sum::<usize>();
    applied + app.world().resource::<PendingCommands>().0.len()
}

/// A right-click on the map at the view point `view`.
fn right_click(app: &mut App, view: Vec2) {
    input(app).cursor = Some(screen_at(app, view));
    input(app).mouse.press(MouseButton::Right);
    app.update();
    input(app).mouse.release(MouseButton::Right);
    app.update();
    pause(app);
}

/// The arrow key held for 8 frames, then the middle button dragged across the view: where the camera looks then.
fn pan(app: &mut App) -> Vec2 {
    press_keys(app, &[KeyCode::ArrowRight]);
    updates(app, 8);
    release_keys(app, &[KeyCode::ArrowRight]);
    app.update();
    let from = screen_at(app, VIEW_CENTER);
    input(app).cursor = Some(from);
    input(app).mouse.press(MouseButton::Middle);
    app.update();
    input(app).cursor = Some(from + Vec2::new(-200.0, 80.0));
    app.update();
    input(app).mouse.release(MouseButton::Middle);
    app.update();
    camera_center(app)
}

#[test]
fn while_the_playtest_menu_is_open_game_input_is_blocked() {
    let mut app = game();
    click(&mut app, at(1), &[]);
    tap(&mut app, &[KeyCode::ControlLeft], KeyCode::Digit2);
    click(&mut app, at(2), &[]);
    assert_eq!(selected(&app), [2]);
    let given = commands(&app);

    tap(&mut app, &[], KeyCode::F10);
    assert_eq!(menu(&app), PlaytestMenu::Paused);
    click(&mut app, at(5), &[]);
    drag(&mut app, at(1) - Vec2::splat(60.0), at(4) + Vec2::splat(60.0));
    tap(&mut app, &[], KeyCode::Digit2);
    tap(&mut app, &[KeyCode::ControlLeft], KeyCode::Digit5);
    tap(&mut app, &[], KeyCode::Tab);
    assert_eq!(selected(&app), [2], "no selection");
    assert!(control_group(&app, 5).is_empty(), "no control group");
    right_click(&mut app, GROUND);
    tap(&mut app, &[], KeyCode::KeyX);
    assert_eq!(commands(&app), given, "no command");
    let camera = camera_center(&app);
    assert_eq!(pan(&mut app), camera, "no camera keys, no grab");
    assert_eq!(menu(&app), PlaytestMenu::Paused, "and the menu stays open");

    // F10 again: the same input works.
    tap(&mut app, &[], KeyCode::F10);
    assert_eq!(menu(&app), PlaytestMenu::Closed);
    tap(&mut app, &[], KeyCode::Digit2);
    assert_eq!(selected(&app), [1], "group 2 recalled");
    right_click(&mut app, GROUND);
    assert!(commands(&app) > given, "an order");
    assert_ne!(pan(&mut app), camera, "the camera pans");
}
