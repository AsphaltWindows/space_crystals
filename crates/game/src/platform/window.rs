//! (Slice M5-S7, design change 0413) The game window and the cursor in it: `core/camera.md > Window Mode` and
//! `core/camera.md > Panning > Arrow keys and screen edge`.
//!
//! - **At launch** ([`super::PlatformPlugin::primary_window`]): "The game starts in **borderless full screen** by
//!   default", on the monitor it opens on (`MonitorSelection::Current`: winit's guess at creation, the monitor under
//!   the mouse on X11; Bevy warns at creation that it can't tell the current monitor yet, and leaves the choice to
//!   winit). `--windowed` opens a window instead, and so does every QA run, rendered at its fixed size and scale
//!   factor for reproducible screenshots (`PlatformPlugin::reproducible`); `--headless` and offscreen runs have no
//!   window at all.
//! - **The toggle** ([`toggle_fullscreen`]): "**F11** or **Alt+Enter** switches between full screen and a window
//!   (keys: placeholder)" ([`InputAction::ToggleFullscreen`]), in every mode of the client. The window comes back
//!   centred on its monitor, at the size it last had as a window ([`WindowedSize`]: the launch's at first).
//! - **The cursor** ([`confine_cursor`]): "The cursor stays in the game window while the game is the active window,
//!   in full screen and in a window alike ... It is freed when the game stops being the active window (Alt-Tab, or
//!   the system switching windows); clicking back into the game confines it again." Bevy 0.20's
//!   `CursorOptions::grab_mode` ([`CursorGrabMode::Confined`]), which `bevy_winit` applies when it changes; the focus
//!   comes from the window's [`WindowFocused`] messages, never from `Window::focused`'s default (true before the
//!   system ever focused the window). macOS has no confinement: Bevy falls back to none there. Never in QA runs,
//!   which ignore the real mouse ([`RawInput`]).

use bevy::prelude::*;
use bevy::window::{
    CursorGrabMode, CursorOptions, MonitorSelection, PrimaryWindow, WindowFocused, WindowMode, WindowPosition,
};

use crate::input::{ActionState, InputAction, RawInput};

/// The mode a window goes full screen in: borderless, on the monitor it is on.
pub const FULL_SCREEN: WindowMode = WindowMode::BorderlessFullscreen(MonitorSelection::Current);

/// How often the cursor is confined again, a frame apart, when confining it failed (`bevy_winit` then puts back the
/// mode it had): the X server refuses a grab while another holds the pointer, as a window manager may for the click
/// that focused the window. Not every frame for as long as the window has the focus, which would log two lines a frame
/// while the grab keeps failing. A QA setting, not a design number.
pub const CONFINE_TRIES: u32 = 30;

/// The physical size the window takes when it leaves full screen: the size it had as a window when it went full
/// screen, the launch's (`--window-size`) at first.
#[derive(Resource, Copy, Clone, Debug, PartialEq, Eq)]
pub struct WindowedSize(pub UVec2);

/// The mode the window switches to from `mode`: a window from full screen, borderless full screen from a window.
pub fn toggled(mode: WindowMode) -> WindowMode {
    match mode {
        WindowMode::Windowed => FULL_SCREEN,
        WindowMode::BorderlessFullscreen(_) | WindowMode::Fullscreen(..) => WindowMode::Windowed,
    }
}

/// [`InputAction::ToggleFullscreen`]: the primary window between borderless full screen and a window (module docs).
pub(super) fn toggle_fullscreen(
    actions: Option<Res<ActionState>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut windowed: ResMut<WindowedSize>,
) {
    if !actions.is_some_and(|a| a.just_pressed(InputAction::ToggleFullscreen)) {
        return;
    }
    for mut window in &mut windows {
        let mode = toggled(window.mode);
        if mode == WindowMode::Windowed {
            let size = windowed.0.max(UVec2::ONE);
            window.resolution.set_physical_resolution(size.x, size.y);
            window.position = WindowPosition::Centered(MonitorSelection::Current);
        } else {
            windowed.0 = window.physical_size();
        }
        window.mode = mode;
    }
}

/// The cursor's grab while the window has the focus or not, and the client reads the real mouse or not (module
/// docs): confined to the window only while both hold.
pub fn cursor_grab(focused: bool, live: bool) -> CursorGrabMode {
    if focused && live {
        CursorGrabMode::Confined
    } else {
        CursorGrabMode::None
    }
}

/// What [`confine_cursor`] knows of the primary window: whether it has the focus, as its last [`WindowFocused`]
/// message said, and how many times it tried to confine the cursor since.
#[derive(Default, Debug)]
pub(super) struct Focus {
    focused: bool,
    tries: u32,
}

/// Confines the cursor to the primary window while it has the focus, and frees it when it loses it (module docs).
pub(super) fn confine_cursor(
    raw: Option<Res<RawInput>>,
    mut focus_changes: MessageReader<WindowFocused>,
    mut windows: Query<(Entity, &mut CursorOptions), With<PrimaryWindow>>,
    mut focus: Local<Focus>,
) {
    let Ok((window, mut cursor)) = windows.single_mut() else {
        focus_changes.clear();
        return;
    };
    for change in focus_changes.read().filter(|change| change.window == window) {
        focus.focused = change.focused;
        focus.tries = 0;
    }
    let grab = cursor_grab(focus.focused, raw.is_some_and(|r| r.enabled));
    if cursor.grab_mode == grab {
        return;
    }
    if grab == CursorGrabMode::None {
        cursor.grab_mode = grab;
    } else if focus.tries < CONFINE_TRIES {
        cursor.grab_mode = grab;
        focus.tries += 1;
    }
}
