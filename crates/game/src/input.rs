//! Raw input → [`InputAction`]s (`ARCHITECTURE.md > Client architecture > Input`).
//!
//! One binding table turns keys into actions; the screen edge pans like the arrow keys. Game systems read only
//! [`ActionState`]. Tests and QA scripts inject actions through [`InjectedActions`], never raw `ButtonInput`.

use std::collections::BTreeSet;

use bevy::input::InputSystems;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

/// Everything the player can ask the client to do.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InputAction {
    /// `core/camera.md > Panning` (bindings: Q0002).
    PanLeft,
    PanRight,
    /// Towards the top of the screen (north).
    PanUp,
    /// Towards the bottom of the screen (south).
    PanDown,
    /// Show or hide the tile grid overlay.
    ToggleGrid,
}

/// The binding table. Placeholder bindings until Q0002 settles the pan inputs.
pub const KEY_BINDINGS: &[(KeyCode, InputAction)] = &[
    (KeyCode::ArrowLeft, InputAction::PanLeft),
    (KeyCode::ArrowRight, InputAction::PanRight),
    (KeyCode::ArrowUp, InputAction::PanUp),
    (KeyCode::ArrowDown, InputAction::PanDown),
    (KeyCode::KeyG, InputAction::ToggleGrid),
];

/// Screen-edge panning: a cursor this close to a window edge (logical pixels) pans that way. Placeholder until
/// Q0002 settles the pan inputs.
pub const EDGE_PAN_MARGIN_PX: f32 = 8.0;

/// This frame's actions, rebuilt every frame from raw input plus injected actions.
#[derive(Resource, Default, Debug)]
pub struct ActionState {
    held: BTreeSet<InputAction>,
    just_pressed: BTreeSet<InputAction>,
}

impl ActionState {
    /// The action is active this frame (key held, cursor at the edge, …).
    pub fn held(&self, action: InputAction) -> bool {
        self.held.contains(&action)
    }

    /// The action started this frame.
    pub fn just_pressed(&self, action: InputAction) -> bool {
        self.just_pressed.contains(&action)
    }
}

/// Actions supplied by tests and scripts. `held` stays active until removed; each `pressed` entry fires once, on
/// the next frame.
#[derive(Resource, Default, Debug)]
pub struct InjectedActions {
    pub held: BTreeSet<InputAction>,
    pub pressed: Vec<InputAction>,
}

/// Whether keyboard and mouse affect the game. Off in QA runs, so a stray cursor or key can't change a screenshot.
#[derive(Resource, Debug)]
pub struct RawInput {
    pub enabled: bool,
}

/// Builds [`ActionState`] in `PreUpdate`, after Bevy has read this frame's raw input.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReadActions;

pub struct ActionInputPlugin {
    pub raw_input: bool,
}

impl Plugin for ActionInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActionState>()
            .init_resource::<InjectedActions>()
            .insert_resource(RawInput {
                enabled: self.raw_input,
            })
            .add_systems(PreUpdate, update_action_state.in_set(ReadActions).after(InputSystems));
    }
}

fn update_action_state(
    raw: Res<RawInput>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut injected: ResMut<InjectedActions>,
    mut state: ResMut<ActionState>,
) {
    let state = &mut *state;
    state.held.clear();
    state.just_pressed.clear();
    if raw.enabled {
        if let Some(keys) = keys {
            for &(key, action) in KEY_BINDINGS {
                if keys.pressed(key) {
                    state.held.insert(action);
                }
                if keys.just_pressed(key) {
                    state.just_pressed.insert(action);
                }
            }
        }
        if let Ok(window) = window.single()
            && window.focused
            && let Some(cursor) = window.cursor_position()
        {
            state.held.extend(edge_pan_actions(cursor, window.size()));
        }
    }
    state.held.extend(injected.held.iter().copied());
    state.just_pressed.extend(injected.pressed.drain(..));
}

/// The pan actions for a cursor at `cursor` (logical pixels, origin top-left) in a window of `size`.
pub fn edge_pan_actions(cursor: Vec2, size: Vec2) -> impl Iterator<Item = InputAction> {
    let m = EDGE_PAN_MARGIN_PX;
    [
        (cursor.x < m, InputAction::PanLeft),
        (cursor.x >= size.x - m, InputAction::PanRight),
        (cursor.y < m, InputAction::PanUp),
        (cursor.y >= size.y - m, InputAction::PanDown),
    ]
    .into_iter()
    .filter_map(|(active, action)| active.then_some(action))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_action_has_exactly_one_key() {
        for action in [
            InputAction::PanLeft,
            InputAction::PanRight,
            InputAction::PanUp,
            InputAction::PanDown,
            InputAction::ToggleGrid,
        ] {
            assert_eq!(
                KEY_BINDINGS.iter().filter(|(_, a)| *a == action).count(),
                1,
                "{action:?}"
            );
        }
    }

    #[test]
    fn screen_edges_pan_towards_that_edge() {
        let size = Vec2::new(800.0, 600.0);
        let at = |x, y| edge_pan_actions(Vec2::new(x, y), size).collect::<Vec<_>>();
        assert_eq!(at(400.0, 300.0), vec![]);
        assert_eq!(at(0.0, 300.0), vec![InputAction::PanLeft]);
        assert_eq!(at(799.0, 599.0), vec![InputAction::PanRight, InputAction::PanDown]);
        assert_eq!(at(400.0, 2.0), vec![InputAction::PanUp]);
    }
}
