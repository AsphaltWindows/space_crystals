//! The shared input layer (`ARCHITECTURE.md > Client architecture > Input`): raw keyboard and mouse input becomes
//! the input of the game and of the map editor in one place, in `PreUpdate`, before any game or editor system runs.
//!
//! - [`Bindings`] is the one rebindable table that maps keys, mouse buttons and the wheel, with modifiers, to
//!   [`InputAction`]s; the screen edge also pans, like the arrow keys. Only the bindings of the mode's
//!   [`InputContext`] fire: the game's in a game or a play-test, the editor's in the editor. Gameplay and editor
//!   code ask for actions, never for keys.
//! - [`ActionState`] is this frame's input: the actions held, just pressed and just released, the wheel's turn, the
//!   modifiers, and the [`Pointer`] (the cursor, the view and ground points under it, whether it is over the UI, the
//!   press on the map and the drag box).
//! - [`MapClick`] and [`SelectionBox`] messages report the pointer actions' clicks and boxes on the map, never over
//!   the UI.
//! - While a text field has the keyboard ([`TextFocus`]), keys type text and fire no binding.
//! - (Slice M5-S7) While a modal UI holds the input ([`Modal`]: the playtest menu), the frame's presses are its own
//!   ([`ActionState::modal_pressed`]): the other systems see only the actions that pass it
//!   ([`InputAction::passes_modal`]), no press, click or box on the map, and the cursor over the UI.
//! - Tests and scripts drive all of it through [`InjectedInput`], a virtual keyboard and mouse, without winit.
//!
//! Timing (double taps, double-clicks) runs on `Time<Real>`, which tests advance with `TimeUpdateStrategy`.

mod actions;
mod bindings;
mod pointer;

use std::collections::BTreeSet;

use bevy::ecs::system::SystemParam;
use bevy::input::InputSystems;
use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::input_focus::{InputFocus, InputFocusSystems};
use bevy::picking::PickingSystems;
use bevy::prelude::*;
use bevy::text::EditableText;
use bevy::window::PrimaryWindow;

pub use actions::{CommandSlot, ControlGroup, EditorInput, InputAction, InputContext};
pub use bindings::{Binding, Bindings, Button, Chord, Modifiers};
pub use pointer::{MapClick, MapPress, Pointer, SelectionBox};

use crate::ClientMode;
use crate::tuning::ClientTuning;
use bindings::TapTracker;

/// This frame's input, rebuilt every frame from the devices and [`InjectedInput`] through the [`Bindings`].
#[derive(Resource, Default, Debug)]
pub struct ActionState {
    held: BTreeSet<InputAction>,
    just_pressed: BTreeSet<InputAction>,
    just_released: BTreeSet<InputAction>,
    /// The wheel action of this frame and how far the wheel turned, in lines.
    wheel: Option<(InputAction, f32)>,
    /// Whose bindings applied this frame.
    context: InputContext,
    cursor: Option<Vec2>,
    modifiers: Modifiers,
    pointer: Pointer,
    /// (M5-S7) The actions pressed this frame while a modal UI holds the input ([`Modal`]); empty otherwise.
    modal_pressed: BTreeSet<InputAction>,
}

impl ActionState {
    /// The action is active this frame (key held, cursor at the edge, modifier held, …). An action is also held in
    /// the frame it was pressed, even when its button already went up again.
    pub fn held(&self, action: InputAction) -> bool {
        self.held.contains(&action)
    }

    /// The action fired this frame, or started being held (a modifier action, the screen edge).
    pub fn just_pressed(&self, action: InputAction) -> bool {
        self.just_pressed.contains(&action)
    }

    /// The action stopped being held this frame.
    pub fn just_released(&self, action: InputAction) -> bool {
        self.just_released.contains(&action)
    }

    /// The actions [`ActionState::just_pressed`] this frame, in [`InputAction`] order.
    pub fn pressed_actions(&self) -> impl Iterator<Item = InputAction> + '_ {
        self.just_pressed.iter().copied()
    }

    /// How far the wheel turned this frame for `action`, a wheel action ([`Binding::Wheel`]), in lines: positive
    /// away from the user (up). Zero when it didn't turn, or turned for another action.
    pub fn scrolled(&self, action: InputAction) -> f32 {
        self.wheel.filter(|(a, _)| *a == action).map_or(0.0, |(_, lines)| lines)
    }

    /// The cursor's position in logical pixels from the window's top-left corner, while it is over the window.
    pub fn cursor(&self) -> Option<Vec2> {
        self.cursor
    }

    /// The modifier keys held.
    pub fn modifiers(&self) -> Modifiers {
        self.modifiers
    }

    /// The input context whose bindings applied this frame: the mode's (`crate::ClientMode::input_context`).
    pub fn context(&self) -> InputContext {
        self.context
    }

    /// Where the pointer is on the map.
    pub fn pointer(&self) -> &Pointer {
        &self.pointer
    }

    /// (M5-S7) Whether `action` was pressed this frame while a modal UI holds the input ([`Modal`]): what that UI reads,
    /// a click being a press of [`InputAction::Select`] at [`ActionState::cursor`]. The other systems don't see it,
    /// unless the action passes the modal UI ([`InputAction::passes_modal`]), when it is [`ActionState::just_pressed`]
    /// too.
    pub fn modal_pressed(&self, action: InputAction) -> bool {
        self.modal_pressed.contains(&action)
    }
}

/// A virtual keyboard and mouse for tests and scripts, read every frame together with the real devices. Press and
/// release buttons on `keys` and `mouse` like Bevy's `ButtonInput`: a press is seen in the next frame, and a press
/// and release before the same frame is a tap. The layer clears their just-pressed and just-released marks after
/// reading them, never the buttons held. Its keys obey the keyboard's rules: none fires a binding while a text field
/// has the keyboard.
#[derive(Resource, Default, Debug)]
pub struct InjectedInput {
    pub keys: ButtonInput<KeyCode>,
    pub mouse: ButtonInput<MouseButton>,
    /// Lines the mouse wheel turns before the next frame: positive away from the user (up). The layer reads and
    /// clears it.
    pub wheel: f32,
    /// Stands in for the mouse cursor (logical pixels from the surface's top-left corner) while set.
    pub cursor: Option<Vec2>,
    /// The injected cursor is over a UI node (headless apps have no UI picking).
    pub over_ui: bool,
    /// Actions held regardless of the bindings, until removed.
    pub held: BTreeSet<InputAction>,
    /// Actions fired once, in the next frame, regardless of the bindings; they also count as held in that frame.
    pub pressed: Vec<InputAction>,
}

impl InjectedInput {
    /// Presses and releases `key` before the next frame.
    pub fn tap_key(&mut self, key: KeyCode) {
        self.keys.press(key);
        self.keys.release(key);
    }
}

/// Whether keyboard and mouse affect the game. Off in QA runs, so a stray cursor or key can't change a screenshot.
#[derive(Resource, Debug)]
pub struct RawInput {
    pub enabled: bool,
}

/// (Slice M5-S7) Whether a modal UI holds the input: the playtest menu while it is open (`crate::playtest_menu`;
/// `core/game_rules.md > Playtest menu (temporary)`, where game input is blocked). The input layer then gives the
/// frame's presses to that UI alone ([`ActionState::modal_pressed`]) and the other systems only the actions that pass
/// it ([`InputAction::passes_modal`]), held or pressed: no pan, selection, command, control group or camera key, no
/// wheel, no edge pan; the pointer is over the UI, so no press, click or box starts on the map, and one under way when
/// the UI takes the input ends without a click or a box. What the systems were shown held ends when it takes the
/// input; what is still held when it lets go comes back held, never pressed, so the click that closes it does nothing
/// else. Set by its owner; the input of a frame follows the setting of the frame before.
#[derive(Resource, Default, Debug, PartialEq, Eq)]
pub struct Modal(pub bool);

/// Whether a structure is being placed, as the placement's owner last reported it: the map editor while its
/// placement tools hold a structure (`crate::editor::input::report_placing`, in the editor only), the game while the
/// local player places a building in AwaitingPlacement (`crate::command_ui`, in the game modes only). While it is,
/// the placement actions' bindings take their presses first (`Bindings::resolve_placing`); the input of a frame
/// follows the report of the frame before.
#[derive(Resource, Default, Debug, PartialEq, Eq)]
pub struct Placing(pub bool);

/// Builds [`ActionState`] and sends the pointer messages in `PreUpdate`, after Bevy has read this frame's raw input
/// and UI hovering, and after the focused text field has seen this frame's keys (it gives up the keyboard on Esc).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReadActions;

pub struct ActionInputPlugin {
    pub raw_input: bool,
}

impl Plugin for ActionInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ClientTuning>();
        app.init_resource::<ActionState>()
            .init_resource::<InjectedInput>()
            .init_resource::<Bindings>()
            .init_resource::<Placing>()
            .init_resource::<Modal>()
            .insert_resource(RawInput {
                enabled: self.raw_input,
            })
            .add_message::<MapClick>()
            .add_message::<SelectionBox>()
            .add_systems(
                PreUpdate,
                (read_actions, pointer::track_pointer)
                    .chain()
                    .in_set(ReadActions)
                    .after(InputSystems)
                    .after(InputFocusSystems::Dispatch)
                    .after(PickingSystems::Hover),
            );
    }
}

/// Whether a text field (the map editor's) has the keyboard: keys then type text instead of firing bindings.
#[derive(SystemParam)]
pub struct TextFocus<'w, 's> {
    focus: Option<Res<'w, InputFocus>>,
    text_fields: Query<'w, 's, (), With<EditableText>>,
}

impl TextFocus<'_, '_> {
    pub fn typing(&self) -> bool {
        let focused = self.focus.as_ref().and_then(|f| f.get());
        focused.is_some_and(|e| self.text_fields.contains(e))
    }
}

/// The real keyboard, mouse and window, the mode, whose input context decides which bindings apply, and whether a
/// structure is being placed.
#[derive(SystemParam)]
struct Devices<'w, 's> {
    raw: Res<'w, RawInput>,
    keys: Option<Res<'w, ButtonInput<KeyCode>>>,
    mouse: Option<Res<'w, ButtonInput<MouseButton>>>,
    scroll: Option<Res<'w, AccumulatedMouseScroll>>,
    window: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    text_focus: TextFocus<'w, 's>,
    mode: Option<Res<'w, ClientMode>>,
    placing: Option<Res<'w, Placing>>,
    modal: Option<Res<'w, Modal>>,
    /// (M5-X2) The wheel's pixels to a line, the double tap's time and the screen edge's margin.
    tuning: Res<'w, ClientTuning>,
}

impl Devices<'_, '_> {
    /// How far the real wheel turned this frame, in lines (positive: up).
    fn wheel_lines(&self) -> f32 {
        let Some(scroll) = self.scroll.as_ref().filter(|_| self.raw.enabled) else {
            return 0.0;
        };
        match scroll.unit {
            MouseScrollUnit::Line => scroll.delta.y,
            MouseScrollUnit::Pixel => scroll.delta.y / self.tuning.input.pixels_per_line,
        }
    }
}

/// The buttons of this frame: the real devices' (while raw input is on) and the injected; their keys not while typing.
struct Buttons<'a> {
    keys: [Option<&'a ButtonInput<KeyCode>>; 2],
    mouse: [Option<&'a ButtonInput<MouseButton>>; 2],
}

impl Buttons<'_> {
    fn pressed(&self, button: Button) -> bool {
        match button {
            Button::Key(key) => self.keys.iter().flatten().any(|keys| keys.pressed(key)),
            Button::Mouse(b) => self.mouse.iter().flatten().any(|mouse| mouse.pressed(b)),
        }
    }

    fn just_pressed(&self, button: Button) -> bool {
        match button {
            Button::Key(key) => self.keys.iter().flatten().any(|keys| keys.just_pressed(key)),
            Button::Mouse(b) => self.mouse.iter().flatten().any(|mouse| mouse.just_pressed(b)),
        }
    }

    fn modifiers(&self) -> Modifiers {
        let either = |a, b| self.pressed(Button::Key(a)) || self.pressed(Button::Key(b));
        Modifiers {
            shift: either(KeyCode::ShiftLeft, KeyCode::ShiftRight),
            ctrl: either(KeyCode::ControlLeft, KeyCode::ControlRight),
            alt: either(KeyCode::AltLeft, KeyCode::AltRight),
        }
    }
}

/// The bindings' memory between frames.
#[derive(Default, Debug)]
struct BindingTracking {
    /// The action each button that is down fired, held until the button goes up.
    latches: Vec<(Button, InputAction)>,
    taps: TapTracker,
    /// A text field had the keyboard in the last frame.
    typed: bool,
    /// The actions held in the last frame, those a modal UI took included ([`Modal`]).
    held: BTreeSet<InputAction>,
}

/// Turns this frame's buttons and wheel into actions through the bindings of the mode's input context
/// (`ARCHITECTURE.md > Client architecture > Input`).
fn read_actions(
    devices: Devices,
    bindings: Res<Bindings>,
    time: Res<Time<Real>>,
    mut injected: ResMut<InjectedInput>,
    mut state: ResMut<ActionState>,
    mut tracking: Local<BindingTracking>,
) {
    let state = &mut *state;
    let raw = devices.raw.enabled;
    let context = devices
        .mode
        .as_deref()
        .map_or(InputContext::Game, |m| m.input_context());
    state.context = context;
    // A text field takes the keys while it has the keyboard, and the Esc or Enter that gives it up: a field that
    // had the keyboard in the last frame may have lost it to this frame's key.
    let typing = devices.text_focus.typing();
    let typed = std::mem::replace(&mut tracking.typed, typing);
    let keyboard = !typing && !typed;
    let window = devices.window.single().ok();
    // What was held in the last frame, and what the systems were shown held: they differ while a modal UI holds the
    // input, which takes the rest ([`Modal`]).
    let previous = std::mem::take(&mut tracking.held);
    let shown_before = std::mem::take(&mut state.held);
    state.just_pressed.clear();
    state.just_released.clear();
    {
        let buttons = Buttons {
            keys: [
                devices.keys.as_deref().filter(|_| raw && keyboard),
                keyboard.then_some(&injected.keys),
            ],
            mouse: [devices.mouse.as_deref().filter(|_| raw), Some(&injected.mouse)],
        };
        let modifiers = buttons.modifiers();
        state.modifiers = modifiers;
        // A button that went up ends the action it fired (one that went down and up within this frame still
        // counts as held in this frame, and ends in the next); so does a change to another context.
        tracking
            .latches
            .retain(|(button, action)| buttons.pressed(*button) && action.works_in(context));
        // Each bound button that went down fires the action of its most specific binding.
        let mut went_down: Vec<Button> = Vec::new();
        for chord in bindings.in_context(context).filter_map(|(_, b)| b.chord()) {
            if buttons.just_pressed(chord.button) && !went_down.contains(&chord.button) {
                went_down.push(chord.button);
            }
        }
        let now = time.elapsed_secs_f64();
        let placing = devices.placing.as_deref().is_some_and(|p| p.0);
        for button in went_down {
            let double = tracking
                .taps
                .press(button, modifiers, now, devices.tuning.input.double_tap_seconds);
            if let Some(action) = bindings.resolve_placing(context, button, modifiers, double, placing) {
                tracking.latches.push((button, action));
                state.just_pressed.insert(action);
            }
        }
        state.held.extend(tracking.latches.iter().map(|(_, action)| *action));
        let roles = bindings.modifier_actions(modifiers).filter(|a| a.works_in(context));
        state.held.extend(roles);
        // The wheel turns the action of its most specific binding.
        let lines = devices.wheel_lines() + std::mem::take(&mut injected.wheel);
        state.wheel = (lines != 0.0 && lines.is_finite())
            .then(|| bindings.resolve_wheel(context, modifiers))
            .flatten()
            .map(|action| (action, lines));
    }
    state.cursor = window.filter(|_| raw).and_then(Window::cursor_position);
    if raw
        && let Some(window) = window
        && window.focused
        && let Some(cursor) = window.cursor_position()
    {
        let margin = devices.tuning.camera.edge_pan_margin_px;
        state.held.extend(edge_pan_actions(cursor, window.size(), margin));
    }
    state.held.extend(injected.held.iter().copied());
    for action in injected.pressed.drain(..) {
        state.held.insert(action);
        state.just_pressed.insert(action);
    }
    if injected.cursor.is_some() {
        state.cursor = injected.cursor;
    }
    // Actions that start being held (modifiers, the screen edge, injected) start here too.
    state.just_pressed.extend(state.held.difference(&previous).copied());
    tracking.held = state.held.clone();
    state.modal_pressed.clear();
    if devices.modal.as_deref().is_some_and(|m| m.0) {
        // A modal UI holds the input: the presses are its own, and the systems see only what passes it.
        state.modal_pressed = std::mem::take(&mut state.just_pressed);
        state.just_pressed = (state.modal_pressed.iter().copied())
            .filter(|a| a.passes_modal())
            .collect();
        state.held.retain(|a| a.passes_modal());
        state.wheel = None;
    }
    // What the systems no longer see held ends: released, or taken by a modal UI.
    state
        .just_released
        .extend(shown_before.difference(&state.held).copied());
    injected.keys.clear();
    injected.mouse.clear();
}

/// The pan actions for a cursor at `cursor` (logical pixels, origin top-left) in a window of `size`, with a margin
/// of `m` pixels (`CameraTuning::edge_pan_margin_px`).
pub fn edge_pan_actions(cursor: Vec2, size: Vec2, m: f32) -> impl Iterator<Item = InputAction> {
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
    fn screen_edges_pan_towards_that_edge() {
        let size = Vec2::new(800.0, 600.0);
        let margin = crate::tuning::camera::BUILTIN.edge_pan_margin_px;
        let at = |x, y| edge_pan_actions(Vec2::new(x, y), size, margin).collect::<Vec<_>>();
        assert_eq!(at(400.0, 300.0), vec![]);
        assert_eq!(at(0.0, 300.0), vec![InputAction::PanLeft]);
        assert_eq!(at(799.0, 599.0), vec![InputAction::PanRight, InputAction::PanDown]);
        assert_eq!(at(400.0, 2.0), vec![InputAction::PanUp]);
    }

    /// (M5-S7) Design change 0413's Verification: "Hold the cursor 5 px from an edge (not touching): the map pans",
    /// within the 8 px margin (`core/camera.md > Panning > Arrow keys and screen edge`); 9 px from it, it doesn't. A
    /// cursor `d` px from an edge stands on the pixel `d` from it: `d` from the left and top, `size - 1 - d` from the
    /// right and bottom.
    #[test]
    fn the_map_pans_5_px_from_an_edge_and_not_9() {
        let margin = crate::tuning::camera::BUILTIN.edge_pan_margin_px;
        assert_eq!(margin, 8.0);
        let size = Vec2::new(1280.0, 720.0);
        let at = |x, y| edge_pan_actions(Vec2::new(x, y), size, margin).collect::<Vec<_>>();
        let (right, bottom) = (size.x - 1.0, size.y - 1.0);
        for d in [5.0, 9.0] {
            let pans = d < margin;
            let expect = |action| if pans { vec![action] } else { vec![] };
            assert_eq!(at(d, 360.0), expect(InputAction::PanLeft), "{d} px from the left");
            assert_eq!(
                at(right - d, 360.0),
                expect(InputAction::PanRight),
                "{d} px from the right"
            );
            assert_eq!(at(640.0, d), expect(InputAction::PanUp), "{d} px from the top");
            assert_eq!(
                at(640.0, bottom - d),
                expect(InputAction::PanDown),
                "{d} px from the bottom"
            );
        }
    }
}
