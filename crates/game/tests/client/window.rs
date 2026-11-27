//! (Slice M5-S7, design change 0413) The game window (`core/camera.md > Window Mode`) and the cursor in it
//! (`core/camera.md > Panning > Arrow keys and screen edge`), on a window of the test's own: a headless app has none,
//! and `bevy_winit`, which would apply the window's mode and the cursor's grab, isn't there either, so the tests read
//! what the client asks of them. The startup mode is the command line's (`cli::tests`), the 8 px edge margin
//! `input::tests`'.

use bevy::window::{
    CursorGrabMode, CursorOptions, MonitorSelection, PrimaryWindow, WindowFocused, WindowMode, WindowPosition,
    WindowResolution,
};
use space_crystals::platform::window::CONFINE_TRIES;
use space_crystals::qa::QaConfig;

use super::*;

/// A game of [`small_scenario`] with a primary window of 1000 × 700 physical pixels, in `mode`.
fn windowed_game(config: AppConfig, mode: WindowMode) -> (App, Entity) {
    let mut app = headless(config);
    let window = Window {
        mode,
        resolution: WindowResolution::new(1000, 700),
        ..default()
    };
    let entity = app.world_mut().spawn((window, PrimaryWindow)).id();
    app.update();
    (app, entity)
}

fn window(app: &App, entity: Entity) -> &Window {
    app.world().get::<Window>(entity).unwrap()
}

fn grab(app: &App, entity: Entity) -> CursorGrabMode {
    app.world().get::<CursorOptions>(entity).unwrap().grab_mode
}

/// The system tells the client the window gained or lost the focus, as `bevy_winit` does.
fn focus(app: &mut App, entity: Entity, focused: bool) {
    app.world_mut().write_message(WindowFocused {
        window: entity,
        focused,
    });
    app.update();
}

/// Taps `key`, with `modifiers` held.
fn chord(app: &mut App, modifiers: &[KeyCode], key: KeyCode) {
    for &m in modifiers {
        input(app).keys.press(m);
    }
    input(app).tap_key(key);
    app.update();
    for &m in modifiers {
        input(app).keys.release(m);
    }
    app.update();
}

/// Design change 0413's Verification: "F11 (or Alt+Enter): it becomes a window; again: full screen." Both keys switch
/// the window there and back; it leaves full screen centred, at the size it had as a window before (here 1000 × 700,
/// whatever full screen made of it), and Enter alone does nothing.
#[test]
fn f11_and_alt_enter_switch_between_full_screen_and_a_window() {
    let full_screen = WindowMode::BorderlessFullscreen(MonitorSelection::Current);
    let (mut app, entity) = windowed_game(AppConfig::new(small_scenario()), WindowMode::Windowed);
    for (modifiers, key) in [(&[][..], KeyCode::F11), (&[KeyCode::AltLeft][..], KeyCode::Enter)] {
        chord(&mut app, modifiers, key);
        assert_eq!(window(&app, entity).mode, full_screen, "{key:?}");
        // Full screen takes the monitor's size, as winit reports it.
        let mut w = app.world_mut().get_mut::<Window>(entity).unwrap();
        w.resolution.set_physical_resolution(1920, 1080);
        chord(&mut app, modifiers, key);
        let w = window(&app, entity);
        assert_eq!(w.mode, WindowMode::Windowed, "{key:?} again");
        assert_eq!(w.physical_size(), UVec2::new(1000, 700), "the size it had as a window");
        assert_eq!(w.position, WindowPosition::Centered(MonitorSelection::Current));
    }
    chord(&mut app, &[], KeyCode::Enter);
    assert_eq!(window(&app, entity).mode, WindowMode::Windowed, "Enter alone");
}

/// A window that starts in full screen, as a game does, leaves it at the launch's size (`--window-size`).
#[test]
fn a_window_that_starts_in_full_screen_leaves_it_at_the_launch_size() {
    let mut config = AppConfig::new(small_scenario());
    config.window_size = UVec2::new(1024, 600);
    let full_screen = WindowMode::BorderlessFullscreen(MonitorSelection::Current);
    let (mut app, entity) = windowed_game(config, full_screen);
    chord(&mut app, &[], KeyCode::F11);
    assert_eq!(window(&app, entity).mode, WindowMode::Windowed);
    assert_eq!(window(&app, entity).physical_size(), UVec2::new(1024, 600));
}

/// Design change 0413: "The cursor stays in the game window while the game is the active window, in full screen and
/// in a window alike ... It is freed when the game stops being the active window (Alt-Tab ...); clicking back into the
/// game confines it again." Confined only once the system said the window has the focus (a new window's
/// `Window::focused` is true before it ever had it), freed when it loses it, confined again when it gets it back.
#[test]
fn the_cursor_is_confined_to_the_window_while_it_has_the_focus() {
    for mode in [
        WindowMode::Windowed,
        WindowMode::BorderlessFullscreen(MonitorSelection::Current),
    ] {
        let (mut app, entity) = windowed_game(AppConfig::new(small_scenario()), mode);
        assert_eq!(grab(&app, entity), CursorGrabMode::None, "{mode:?}: not focused yet");
        focus(&mut app, entity, true);
        assert_eq!(grab(&app, entity), CursorGrabMode::Confined, "{mode:?}: focused");
        focus(&mut app, entity, false);
        assert_eq!(grab(&app, entity), CursorGrabMode::None, "{mode:?}: Alt-Tab frees it");
        focus(&mut app, entity, true);
        assert_eq!(
            grab(&app, entity),
            CursorGrabMode::Confined,
            "{mode:?}: back, confined again"
        );
        // The toggle keeps it confined.
        chord(&mut app, &[], KeyCode::F11);
        assert_eq!(grab(&app, entity), CursorGrabMode::Confined, "{mode:?}: toggled");
    }
}

/// A grab the system refused (`bevy_winit` then puts the cursor's grab back as it was) is tried again in the next
/// frames, [`CONFINE_TRIES`] times at most until the focus changes again; and a QA run, which ignores the real mouse,
/// never confines it.
#[test]
fn a_refused_grab_is_tried_again_and_a_qa_run_never_confines_the_cursor() {
    let (mut app, entity) = windowed_game(AppConfig::new(small_scenario()), WindowMode::Windowed);
    focus(&mut app, entity, true);
    let refuse = |app: &mut App| {
        let mut options = app.world_mut().get_mut::<CursorOptions>(entity).unwrap();
        options.bypass_change_detection().grab_mode = CursorGrabMode::None;
        app.update();
        app.world().get::<CursorOptions>(entity).unwrap().grab_mode
    };
    for _ in 1..CONFINE_TRIES {
        assert_eq!(refuse(&mut app), CursorGrabMode::Confined, "tried again");
    }
    assert_eq!(refuse(&mut app), CursorGrabMode::None, "given up");
    focus(&mut app, entity, false);
    focus(&mut app, entity, true);
    assert_eq!(grab(&app, entity), CursorGrabMode::Confined, "a new focus, new tries");

    let mut qa = AppConfig::new(small_scenario());
    qa.qa = QaConfig {
        exit_at_tick: Some(1000),
        ..default()
    };
    let (mut app, entity) = windowed_game(qa, WindowMode::Windowed);
    focus(&mut app, entity, true);
    assert_eq!(grab(&app, entity), CursorGrabMode::None, "a QA run");
}
