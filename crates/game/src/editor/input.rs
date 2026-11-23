//! The shared input layer's actions → [`EditorAction`]s (`ARCHITECTURE.md > Client architecture > Input`). The
//! editor reads only what the layer produces: the actions of the editor's input context ([`ActionState`]), the
//! pointer ([`Pointer`]: the ground under the cursor, whether it is over the UI, the press on the map) and its clicks
//! ([`MapClick`]); never raw keys, buttons, the wheel or the cursor. Tests and scripts inject input into the layer
//! (`crate::input::InjectedInput`) or queue actions directly.
//!
//! The pointer: the primary action ([`EditorInput::UseTool`]) uses the tool from its press on the map, through the
//! drag, to its release; the eyedropper picks a tile's preset and height; the wheel zooms at the cursor. Keys: see
//! [`editor_action`] and the F1 help; the default bindings are in `crate::input::Bindings::default`. Panning is the
//! game camera's own: the arrow keys, the screen edge and the middle-mouse grab (`crate::camera::GrabPan`). Every
//! position is the ground point drawn under the cursor ([`Pointer::ground`]). The layer's rules keep the map from
//! the pointer while it is over the UI, and the bindings from the keys while a text field has the keyboard.
//!
//! The window's close button becomes [`EditorAction::Quit`], which asks a second time when there are unsaved
//! changes (`crate::platform::ConfirmClose`).

use bevy::prelude::*;
use bevy::window::WindowCloseRequested;

use super::actions::{EditorAction, EditorActions};
use super::tools::{EditorTools, Tool};
use crate::ClientMode;
use crate::input::{ActionState, EditorInput, InputAction, MapClick, Pointer};

/// Zoom per wheel line: one line up shows this fraction of the map shown before.
const ZOOM_PER_LINE: f32 = 0.85;

/// The keys' actions: all of them in the editor; in its play-tests only the play-test switch (F5), because Esc is
/// the game's Cancel there (`systems/controls/selection_and_hud.md > AwaitingTarget`).
pub fn editor_keys(
    actions: Res<ActionState>,
    mode: Res<ClientMode>,
    tools: Res<EditorTools>,
    mut queue: ResMut<EditorActions>,
) {
    for action in actions.pressed_actions() {
        let wanted = match *mode {
            ClientMode::Editor => true,
            ClientMode::PlayTest => action == InputAction::Editor(EditorInput::PlayTest),
            ClientMode::Game => false,
        };
        if let Some(editor_action) = editor_action(action, &tools).filter(|_| wanted) {
            queue.push(editor_action);
        }
    }
}

/// The editor action of a pressed input action, given the tools' settings. `None` for the game's actions and for
/// the pointer's ([`editor_pointer`]); the grid's (G) is the map renderer's own in every mode.
pub fn editor_action(action: InputAction, tools: &EditorTools) -> Option<EditorAction> {
    use EditorAction as A;
    use EditorInput as E;
    let InputAction::Editor(input) = action else {
        return (action == InputAction::Cancel).then_some(A::Cancel);
    };
    Some(match input {
        E::SelectTool => A::SetTool(Tool::Select),
        E::PaintTool => A::SetTool(Tool::Paint),
        E::HeightTool => A::SetTool(Tool::Elevation),
        E::FillTool => A::SetTool(Tool::Fill),
        E::PlaceTool => A::SetTool(Tool::Place),
        E::StartTool => A::SetTool(Tool::Start),
        E::CameraTool => A::SetTool(Tool::CameraStart),
        E::SmallerBrush => A::SetBrush(tools.stepped_brush(-1)),
        E::LargerBrush => A::SetBrush(tools.stepped_brush(1)),
        E::NextHeightMode => A::SetElevationMode(tools.elevation_mode.next()),
        E::RaiseLevel => A::StepElevation(1),
        E::LowerLevel => A::StepElevation(-1),
        E::ToggleSlopes => A::ToggleSlopes,
        E::FitMap => A::FitMap,
        E::DeleteSelection => A::DeleteSelected,
        E::Undo => A::Undo,
        E::Redo => A::Redo,
        E::Save => A::Save,
        E::NewMap => A::NewMap,
        E::PlayTest => A::PlayTest,
        E::ToggleHelp => A::ToggleHelp,
        E::UseTool | E::Eyedropper | E::Zoom => return None,
    })
}

/// The window's close button (review finding: it closed the editor without asking about unsaved changes). Several
/// requests in one frame are one.
pub fn close_requests(mut requests: MessageReader<WindowCloseRequested>, mut actions: ResMut<EditorActions>) {
    if requests.read().count() > 0 {
        actions.push(EditorAction::Quit);
    }
}

/// What the pointer last did, for turning its changes into actions.
#[derive(Default)]
pub struct PointerTrack {
    hovered: Option<Vec2>,
    /// The editor pressed on the map ([`EditorAction::Press`]) and hasn't released yet.
    pressing: bool,
}

/// The pointer over the map view, in the editor: what it hovers, the tool's press, drag and release
/// ([`Pointer::press`]), the eyedropper's clicks and the wheel's zoom at the cursor. Nothing starts over the UI, but
/// a drag goes on over the UI's nodes on the map view.
pub fn editor_pointer(
    actions: Res<ActionState>,
    mode: Res<ClientMode>,
    mut clicks: MessageReader<MapClick>,
    mut track: Local<PointerTrack>,
    mut queue: ResMut<EditorActions>,
) {
    let eyedropper = InputAction::Editor(EditorInput::Eyedropper);
    let picks: Vec<Vec2> = clicks
        .read()
        .filter(|c| c.action == eyedropper)
        .map(|c| c.ground)
        .collect();
    if *mode != ClientMode::Editor {
        return;
    }
    let pointer: &Pointer = actions.pointer();
    let hovered = pointer.ground;
    if hovered != track.hovered {
        track.hovered = hovered;
        queue.push(EditorAction::Hover(hovered));
    }
    let use_tool = InputAction::Editor(EditorInput::UseTool);
    match pointer.press.filter(|press| press.action == use_tool) {
        Some(press) => {
            if actions.just_pressed(use_tool) {
                track.pressing = true;
                queue.push(EditorAction::Press(press.ground));
            }
            // The drag goes on over the UI's nodes on the map view; off the view it waits.
            if let Some(pos) = press.drag_ground.filter(|_| track.pressing) {
                queue.push(EditorAction::Drag(pos));
            }
        }
        None if track.pressing => {
            track.pressing = false;
            queue.push(EditorAction::Release);
        }
        None => {}
    }
    for pos in picks {
        queue.push(EditorAction::Pick(pos));
    }
    // The wheel zooms at the cursor; over the UI it scrolls the panels instead.
    let lines = actions.scrolled(InputAction::Editor(EditorInput::Zoom));
    if let Some(pos) = hovered.filter(|_| lines != 0.0) {
        queue.push(EditorAction::Zoom {
            factor: ZOOM_PER_LINE.powf(lines),
            at: Some(pos),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::{Bindings, Button, InputContext, Modifiers};

    /// The default bindings give the shortcuts of the F1 help.
    #[test]
    fn the_default_bindings_follow_the_help() {
        let tools = EditorTools::default();
        let bindings = Bindings::default();
        let key = |k, modifiers| {
            let action = bindings.resolve(InputContext::Editor, Button::Key(k), modifiers, false)?;
            editor_action(action, &tools)
        };
        let none = Modifiers::NONE;
        let ctrl = Modifiers::CTRL;
        let ctrl_shift = Modifiers { shift: true, ..ctrl };
        assert_eq!(key(KeyCode::KeyZ, ctrl), Some(EditorAction::Undo));
        assert_eq!(key(KeyCode::KeyZ, ctrl_shift), Some(EditorAction::Redo));
        assert_eq!(key(KeyCode::KeyY, ctrl), Some(EditorAction::Redo));
        assert_eq!(key(KeyCode::KeyS, ctrl), Some(EditorAction::Save));
        assert_eq!(key(KeyCode::KeyN, ctrl), Some(EditorAction::NewMap));
        assert_eq!(key(KeyCode::KeyZ, none), None, "Z alone is nothing");
        assert_eq!(key(KeyCode::KeyB, ctrl), None, "Ctrl+B is nothing");
        let tool_keys = [
            (KeyCode::KeyV, Tool::Select),
            (KeyCode::KeyB, Tool::Paint),
            (KeyCode::KeyH, Tool::Elevation),
            (KeyCode::KeyF, Tool::Fill),
            (KeyCode::KeyO, Tool::Place),
            (KeyCode::KeyL, Tool::Start),
            (KeyCode::KeyC, Tool::CameraStart),
        ];
        for (k, tool) in tool_keys {
            assert_eq!(key(k, none), Some(EditorAction::SetTool(tool)), "{k:?}");
        }
        assert_eq!(key(KeyCode::BracketRight, none), Some(EditorAction::SetBrush(5)));
        assert_eq!(key(KeyCode::BracketLeft, none), Some(EditorAction::SetBrush(1)));
        assert_eq!(
            key(KeyCode::KeyE, none),
            Some(EditorAction::SetElevationMode(tools.elevation_mode.next()))
        );
        assert_eq!(key(KeyCode::Equal, none), Some(EditorAction::StepElevation(1)));
        assert_eq!(key(KeyCode::Minus, none), Some(EditorAction::StepElevation(-1)));
        assert_eq!(key(KeyCode::Home, none), Some(EditorAction::FitMap));
        assert_eq!(key(KeyCode::Delete, none), Some(EditorAction::DeleteSelected));
        assert_eq!(key(KeyCode::Escape, none), Some(EditorAction::Cancel));
        assert_eq!(key(KeyCode::F1, none), Some(EditorAction::ToggleHelp));
        assert_eq!(key(KeyCode::F5, none), Some(EditorAction::PlayTest));
        assert_eq!(key(KeyCode::KeyK, none), Some(EditorAction::ToggleSlopes));
        assert_eq!(key(KeyCode::KeyQ, none), None);
        // The symmetry is chosen when the map is created and is fixed after.
        assert_eq!(key(KeyCode::KeyM, none), None);
        // G is the shared grid toggle, which the map's renderer applies in every mode.
        assert_eq!(key(KeyCode::KeyG, none), None);
        // The pointer's actions are the pointer system's.
        assert_eq!(editor_action(InputAction::Editor(EditorInput::UseTool), &tools), None);
        assert_eq!(editor_action(InputAction::Select, &tools), None);
    }
}
