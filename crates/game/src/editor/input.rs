//! Raw keyboard and mouse → [`EditorAction`]s. Off in QA runs (`RawInput`), like the game's input; tests and
//! scripts queue actions directly.
//!
//! Mouse: left uses the tool, right picks a tile's preset and height, the wheel zooms at the cursor. Keys: see
//! [`key_actions`] and the F1 help. Panning is the game camera's own: the arrow keys, the screen edge and the
//! middle-mouse grab (`crate::camera::GrabPan`). Every position is the ground point drawn under the cursor
//! (`crate::camera::cursor_to_ground`). Nothing reaches the map while the pointer is over the editor's UI or a text
//! field has the keyboard.
//!
//! The window's close button becomes [`EditorAction::Quit`], which asks a second time when there are unsaved
//! changes (`crate::platform::ConfirmClose`).

use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::picking::hover::HoverMap;
use bevy::picking::pointer::PointerId;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowCloseRequested};

use super::actions::{EditorAction, EditorActions};
use super::doc::EditorDoc;
use super::state::EditorUi;
use super::tools::{EditorTools, Tool};
use crate::ClientMode;
use crate::camera::{CameraView, CameraZoom, ViewportInsets, cursor_to_ground};
use crate::input::{RawInput, TextFocus};
use crate::platform::SurfaceSize;

/// Zoom per wheel line: one line up shows this fraction of the map shown before.
const ZOOM_PER_LINE: f32 = 0.85;
/// Wheels that report pixels: this many pixels count as one line.
const PIXELS_PER_LINE: f32 = 40.0;

/// Keyboard shortcuts, for a frame in which no text field has the keyboard.
pub fn editor_keys(
    raw: Res<RawInput>,
    mode: Res<ClientMode>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    text_focus: TextFocus,
    tools: Res<EditorTools>,
    mut typed_last_frame: Local<bool>,
    mut actions: ResMut<EditorActions>,
) {
    let Some(keys) = keys.filter(|_| raw.enabled) else {
        return;
    };
    // A text field takes the keys while it has the keyboard, and the Escape or Enter that gives it up.
    let typing_now = text_focus.typing();
    let typed = std::mem::replace(&mut *typed_last_frame, typing_now);
    if typing_now || typed {
        return;
    }
    if *mode == ClientMode::PlayTest {
        // Only F5: in the game Esc cancels (`systems/controls/selection_and_hud.md`).
        if keys.just_pressed(KeyCode::F5) {
            actions.push(EditorAction::PlayTest);
        }
        return;
    }
    let ctrl = keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
    let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    for key in keys.get_just_pressed() {
        if let Some(action) = key_actions(*key, ctrl, shift, &tools) {
            actions.push(action);
        }
    }
}

/// The action of a key press, given the modifiers held and the tools' settings.
pub fn key_actions(key: KeyCode, ctrl: bool, shift: bool, tools: &EditorTools) -> Option<EditorAction> {
    use EditorAction as A;
    if ctrl {
        return match key {
            KeyCode::KeyZ if shift => Some(A::Redo),
            KeyCode::KeyZ => Some(A::Undo),
            KeyCode::KeyY => Some(A::Redo),
            KeyCode::KeyS => Some(A::Save),
            KeyCode::KeyN => Some(A::NewMap),
            _ => None,
        };
    }
    Some(match key {
        KeyCode::F1 => A::ToggleHelp,
        KeyCode::F5 => A::PlayTest,
        KeyCode::Escape => A::Cancel,
        KeyCode::Delete | KeyCode::Backspace => A::DeleteSelected,
        KeyCode::Home => A::FitMap,
        KeyCode::KeyV => A::SetTool(Tool::Select),
        KeyCode::KeyB => A::SetTool(Tool::Paint),
        KeyCode::KeyH => A::SetTool(Tool::Elevation),
        KeyCode::KeyF => A::SetTool(Tool::Fill),
        KeyCode::KeyO => A::SetTool(Tool::Place),
        KeyCode::KeyL => A::SetTool(Tool::Start),
        KeyCode::KeyC => A::SetTool(Tool::CameraStart),
        KeyCode::BracketLeft => A::SetBrush(tools.stepped_brush(-1)),
        KeyCode::BracketRight => A::SetBrush(tools.stepped_brush(1)),
        KeyCode::KeyK => A::ToggleSlopes,
        KeyCode::KeyE => A::SetElevationMode(tools.elevation_mode.next()),
        KeyCode::Equal | KeyCode::NumpadAdd => A::StepElevation(1),
        KeyCode::Minus | KeyCode::NumpadSubtract => A::StepElevation(-1),
        _ => return None,
    })
}

/// The window's close button (review finding: it closed the editor without asking about unsaved changes). Several
/// requests in one frame are one.
pub fn close_requests(mut requests: MessageReader<WindowCloseRequested>, mut actions: ResMut<EditorActions>) {
    if requests.read().count() > 0 {
        actions.push(EditorAction::Quit);
    }
}

/// Where the pointer was, for turning pointer changes into actions.
#[derive(Default)]
pub struct PointerTrack {
    hovered: Option<Vec2>,
    /// The primary button went down on the map and is still held.
    dragging: bool,
}

/// Mouse buttons, wheel and cursor over the map view.
#[expect(clippy::too_many_arguments, reason = "a system's parameters")]
pub fn editor_pointer(
    raw: Res<RawInput>,
    mode: Res<ClientMode>,
    mouse: Option<Res<ButtonInput<MouseButton>>>,
    scroll: Option<Res<AccumulatedMouseScroll>>,
    window: Query<&Window, With<PrimaryWindow>>,
    hover: Option<Res<HoverMap>>,
    ui_nodes: Query<(), With<Node>>,
    view_state: (Res<SurfaceSize>, Res<ViewportInsets>, Res<CameraView>, Res<CameraZoom>),
    ui: Res<EditorUi>,
    doc: Res<EditorDoc>,
    mut track: Local<PointerTrack>,
    mut actions: ResMut<EditorActions>,
) {
    let (size, insets, view, zoom) = view_state;
    let Some(mouse) = mouse.filter(|_| raw.enabled && *mode == ClientMode::Editor) else {
        return;
    };
    let cursor = window.single().ok().and_then(Window::cursor_position);
    let over_ui = ui.help
        || hover.as_ref().is_some_and(|hover| {
            hover
                .get(&PointerId::Mouse)
                .is_some_and(|hits| hits.keys().any(|e| ui_nodes.contains(*e)))
        });
    let map = &doc.map().terrain;
    let on_map = cursor
        .filter(|_| !over_ui || track.dragging)
        .and_then(|c| cursor_to_ground(c, *size, *insets, *view, zoom.0, map));

    let hovered = on_map.filter(|_| !over_ui);
    if hovered != track.hovered {
        track.hovered = hovered;
        actions.push(EditorAction::Hover(hovered));
    }
    if let Some(pos) = hovered {
        if mouse.just_pressed(MouseButton::Left) {
            track.dragging = true;
            actions.push(EditorAction::Press(pos));
        } else if mouse.just_pressed(MouseButton::Right) {
            actions.push(EditorAction::Pick(pos));
        }
    }
    if track.dragging {
        if !mouse.pressed(MouseButton::Left) {
            track.dragging = false;
            actions.push(EditorAction::Release);
        } else if let Some(pos) = on_map {
            actions.push(EditorAction::Drag(pos));
        }
    }

    // The wheel zooms at the cursor; over the UI it scrolls the panels instead.
    if let (Some(scroll), Some(pos)) = (scroll, hovered) {
        let lines = match scroll.unit {
            MouseScrollUnit::Line => scroll.delta.y,
            MouseScrollUnit::Pixel => scroll.delta.y / PIXELS_PER_LINE,
        };
        if lines != 0.0 && lines.is_finite() {
            actions.push(EditorAction::Zoom {
                factor: ZOOM_PER_LINE.powf(lines),
                at: Some(pos),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortcuts_follow_the_help() {
        let tools = EditorTools::default();
        let key = |k, ctrl, shift| key_actions(k, ctrl, shift, &tools);
        assert_eq!(key(KeyCode::KeyZ, true, false), Some(EditorAction::Undo));
        assert_eq!(key(KeyCode::KeyZ, true, true), Some(EditorAction::Redo));
        assert_eq!(key(KeyCode::KeyY, true, false), Some(EditorAction::Redo));
        assert_eq!(key(KeyCode::KeyS, true, false), Some(EditorAction::Save));
        assert_eq!(key(KeyCode::KeyN, true, false), Some(EditorAction::NewMap));
        assert_eq!(key(KeyCode::KeyB, true, false), None, "Ctrl+B is nothing");
        assert_eq!(
            key(KeyCode::KeyB, false, false),
            Some(EditorAction::SetTool(Tool::Paint))
        );
        assert_eq!(
            key(KeyCode::KeyL, false, false),
            Some(EditorAction::SetTool(Tool::Start))
        );
        assert_eq!(
            key(KeyCode::BracketRight, false, false),
            Some(EditorAction::SetBrush(5))
        );
        assert_eq!(key(KeyCode::Minus, false, false), Some(EditorAction::StepElevation(-1)));
        assert_eq!(key(KeyCode::F5, false, false), Some(EditorAction::PlayTest));
        assert_eq!(key(KeyCode::KeyK, false, false), Some(EditorAction::ToggleSlopes));
        assert_eq!(key(KeyCode::KeyQ, false, false), None);
        // The symmetry is chosen when the map is created and is fixed after.
        assert_eq!(key(KeyCode::KeyM, false, false), None);
    }
}
