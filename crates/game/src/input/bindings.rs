//! The bindings table (`ARCHITECTURE.md > Client architecture > Input`): every binding of the game and of the map
//! editor in one resource, so that rebinding (`systems/controls/interface.md > Key Rebinding`) changes
//! data, not gameplay code.
//!
//! Each entry binds an [`InputAction`] to a key or mouse button with the modifiers held with it (a [`Chord`]), to a
//! double tap of a chord, to modifiers alone, or to the mouse wheel. Only the bindings of the input context in use
//! fire ([`InputAction::context`]). A press fires at most one action, the one of its most specific binding
//! ([`Bindings::resolve`]): so Shift + 1 adds to control group 1 instead of recalling it, while Shift + Q still
//! presses command slot Q, with the queue modifier held.

use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::MouseButton;
use bevy::prelude::Resource;

use super::actions::{CommandSlot, ControlGroup, EditorInput, InputAction, InputContext};

pub use crate::tuning::input::DOUBLE_TAP_SECONDS;

/// A key or a mouse button.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Button {
    Key(KeyCode),
    Mouse(MouseButton),
}

/// The modifier keys held, on either side of the keyboard.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

impl Modifiers {
    pub const NONE: Modifiers = Modifiers {
        shift: false,
        ctrl: false,
        alt: false,
    };
    pub const SHIFT: Modifiers = Modifiers {
        shift: true,
        ..Self::NONE
    };
    pub const CTRL: Modifiers = Modifiers {
        ctrl: true,
        ..Self::NONE
    };
    pub const ALT: Modifiers = Modifiers {
        alt: true,
        ..Self::NONE
    };

    /// Every modifier of `other` is held here too.
    pub fn contains(self, other: Modifiers) -> bool {
        (self.shift || !other.shift) && (self.ctrl || !other.ctrl) && (self.alt || !other.alt)
    }

    /// How many modifiers are held.
    pub fn count(self) -> u32 {
        u32::from(self.shift) + u32::from(self.ctrl) + u32::from(self.alt)
    }
}

/// A button pressed while at least some modifiers are held: Q, Ctrl + 1, Shift + Tab.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Chord {
    pub button: Button,
    pub modifiers: Modifiers,
}

impl Chord {
    pub fn key(key: KeyCode) -> Self {
        Self {
            button: Button::Key(key),
            modifiers: Modifiers::NONE,
        }
    }

    pub fn mouse(button: MouseButton) -> Self {
        Self {
            button: Button::Mouse(button),
            modifiers: Modifiers::NONE,
        }
    }

    /// The same button, pressed with `modifiers` held.
    pub fn with(self, modifiers: Modifiers) -> Self {
        Self { modifiers, ..self }
    }
}

/// How an action is triggered.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Binding {
    /// Fires when the chord's button goes down while at least the chord's modifiers are held, and stays held until
    /// the button goes up.
    Press(Chord),
    /// Fires on the second of two presses of the same chord, with the same modifiers, within
    /// [`DOUBLE_TAP_SECONDS`] and with no other bound button pressed in between. Held like [`Binding::Press`].
    DoubleTap(Chord),
    /// Held while at least these modifiers are held: the modifier actions, such as the queue modifier. It never
    /// fires on a press and never competes with other bindings.
    Modifiers(Modifiers),
    /// The mouse wheel, turned while at least these modifiers are held: how far it turned is the action's value in
    /// that frame (`super::ActionState::scrolled`). A turn goes to the most specific wheel binding, like a press.
    Wheel(Modifiers),
}

impl Binding {
    /// The chord of a press or a double tap.
    pub fn chord(self) -> Option<Chord> {
        match self {
            Binding::Press(chord) | Binding::DoubleTap(chord) => Some(chord),
            Binding::Modifiers(_) | Binding::Wheel(_) => None,
        }
    }
}

/// The bindings table: `(action, binding)` entries in precedence order. An action may have several bindings, and
/// a button several actions with different modifiers. Gameplay code never reads it, only the actions it yields
/// (`super::ActionState`). A rebinding UI changes it with [`Bindings::bind`] and [`Bindings::unbind`].
#[derive(Resource, Clone, Debug, PartialEq)]
pub struct Bindings {
    entries: Vec<(InputAction, Binding)>,
}

impl Bindings {
    /// A table of these entries, in precedence order.
    pub fn new(entries: Vec<(InputAction, Binding)>) -> Self {
        Self { entries }
    }

    pub fn entries(&self) -> &[(InputAction, Binding)] {
        &self.entries
    }

    /// The bindings of `action`.
    pub fn of(&self, action: InputAction) -> impl Iterator<Item = Binding> + '_ {
        self.entries.iter().filter(move |(a, _)| *a == action).map(|(_, b)| *b)
    }

    /// Adds a binding of `action`, after the existing ones.
    pub fn bind(&mut self, action: InputAction, binding: Binding) {
        self.entries.push((action, binding));
    }

    /// Removes every binding of `action`.
    pub fn unbind(&mut self, action: InputAction) {
        self.entries.retain(|(a, _)| *a != action);
    }

    /// The entries whose actions work in `context` ([`InputAction::works_in`]), in precedence order.
    pub fn in_context(&self, context: InputContext) -> impl Iterator<Item = (InputAction, Binding)> + '_ {
        self.entries
            .iter()
            .copied()
            .filter(move |(action, _)| action.works_in(context))
    }

    /// The action that a press of `button` fires in `context` while `modifiers` are held and nothing is being
    /// placed; `double` says whether the press completes a double tap. Of the bindings of that button in that
    /// context whose modifiers are all held, the one requiring the most modifiers wins, then a double tap over a
    /// single press, then the earlier entry. Modifiers held beyond a binding's don't block it, except Ctrl for the
    /// editor's own keys ([`InputContext::ctrl_blocks`]). The placement actions' bindings take no press
    /// ([`Bindings::resolve_placing`]).
    pub fn resolve(
        &self,
        context: InputContext,
        button: Button,
        modifiers: Modifiers,
        double: bool,
    ) -> Option<InputAction> {
        self.resolve_placing(context, button, modifiers, double, false)
    }

    /// [`Bindings::resolve`], with `placing` saying whether a structure is being placed (`super::Placing`). While
    /// one is, the bindings of the placement actions ([`InputAction::is_placement`]) take part and win over the
    /// others of the press, which they override; otherwise they take no press.
    pub fn resolve_placing(
        &self,
        context: InputContext,
        button: Button,
        modifiers: Modifiers,
        double: bool,
        placing: bool,
    ) -> Option<InputAction> {
        let mut best: Option<((bool, u32, bool), InputAction)> = None;
        for (action, binding) in self.in_context(context) {
            let (chord, is_double) = match binding {
                Binding::Press(chord) => (chord, false),
                Binding::DoubleTap(chord) if double => (chord, true),
                Binding::DoubleTap(_) | Binding::Modifiers(_) | Binding::Wheel(_) => continue,
            };
            if chord.button != button || !modifiers.contains(chord.modifiers) {
                continue;
            }
            let placement = action.is_placement();
            if placement && !placing {
                continue;
            }
            if modifiers.ctrl
                && !chord.modifiers.ctrl
                && matches!(button, Button::Key(_))
                && context.ctrl_blocks(action)
            {
                continue;
            }
            let rank = (placement, chord.modifiers.count(), is_double);
            if best.is_none_or(|(best_rank, _)| rank > best_rank) {
                best = Some((rank, action));
            }
        }
        best.map(|(_, action)| action)
    }

    /// The action that a turn of the wheel drives in `context` while `modifiers` are held: of the wheel bindings in
    /// that context whose modifiers are all held, the one requiring the most modifiers, then the earlier entry.
    pub fn resolve_wheel(&self, context: InputContext, modifiers: Modifiers) -> Option<InputAction> {
        let mut best: Option<(u32, InputAction)> = None;
        for (action, binding) in self.in_context(context) {
            let Binding::Wheel(needed) = binding else {
                continue;
            };
            let rank = needed.count();
            if modifiers.contains(needed) && best.is_none_or(|(best_rank, _)| rank > best_rank) {
                best = Some((rank, action));
            }
        }
        best.map(|(_, action)| action)
    }

    /// The actions that modifier bindings hold while `modifiers` are held, in every context.
    pub fn modifier_actions(&self, modifiers: Modifiers) -> impl Iterator<Item = InputAction> + '_ {
        self.entries.iter().filter_map(move |&(action, binding)| match binding {
            Binding::Modifiers(m) if m != Modifiers::NONE && modifiers.contains(m) => Some(action),
            _ => None,
        })
    }

    /// Whether `modifiers` hold the modifier action `action`: how a click with these modifiers is meant, e.g.
    /// queued (`InputAction::Queue`).
    pub fn holds(&self, action: InputAction, modifiers: Modifiers) -> bool {
        self.modifier_actions(modifiers).any(|a| a == action)
    }
}

/// The defaults: `core/camera.md > Panning`, `systems/controls/selection.md` (`> MapClickSelection`,
/// `> SelectionPanel > Interactions`, `> AwaitingTarget`, `> GroupCycling`, `> ControlGroups > Default keys`,
/// `> CommandPanel > Grid Layout`) and `systems/controls/commands.md > Unit Command` (design changes 0002, 0010 and
/// 0011), and placeholder debug and replay playback keys; then the map editor's, which its F1 help lists (the design
/// gives none).
impl Default for Bindings {
    fn default() -> Self {
        use Binding::{DoubleTap, Modifiers as Held, Press, Wheel};
        use EditorInput as E;
        use InputAction as A;
        let key = Chord::key;
        let mut entries = vec![
            (A::PanLeft, Press(key(KeyCode::ArrowLeft))),
            (A::PanRight, Press(key(KeyCode::ArrowRight))),
            (A::PanUp, Press(key(KeyCode::ArrowUp))),
            (A::PanDown, Press(key(KeyCode::ArrowDown))),
            (A::GrabPan, Press(Chord::mouse(MouseButton::Middle))),
            (A::ToggleGrid, Press(key(KeyCode::KeyG))),
            (A::TogglePaths, Press(key(KeyCode::KeyP))),
            (A::Select, Press(Chord::mouse(MouseButton::Left))),
            (A::ContextCommand, Press(Chord::mouse(MouseButton::Right))),
            (A::Cancel, Press(key(KeyCode::Escape))),
            (A::NextGroup, Press(key(KeyCode::Tab))),
            (A::PreviousGroup, Press(key(KeyCode::Tab).with(Modifiers::SHIFT))),
            (A::Queue, Held(Modifiers::SHIFT)),
            (A::ToggleSelection, Held(Modifiers::SHIFT)),
            (A::SelectSameType, Held(Modifiers::CTRL)),
            (A::CenterOnObject, Held(Modifiers::ALT)),
            // Placeholder: the design has no debug keys.
            (A::DebugSwitchPlayer, Press(key(KeyCode::F2))),
            (A::DebugToggleFog, Press(key(KeyCode::F3))),
            // Placeholders: the design has no replays yet.
            (A::ReplayPause, Press(key(KeyCode::Space))),
            (A::ReplayStep, Press(key(KeyCode::Period))),
            (A::ReplaySlower, Press(key(KeyCode::Minus))),
            (A::ReplayFaster, Press(key(KeyCode::Equal))),
            // (M5-S7) The design's placeholder keys: `core/camera.md > Window Mode` ("F11 or Alt+Enter") and
            // `core/game_rules.md > Playtest menu (temporary)` (F10).
            (A::ToggleFullscreen, Press(key(KeyCode::F11))),
            (A::ToggleFullscreen, Press(key(KeyCode::Enter).with(Modifiers::ALT))),
            (A::PlaytestMenu, Press(key(KeyCode::F10))),
        ];
        for group in ControlGroup::all() {
            let digit = key(group.digit_key());
            entries.extend([
                (A::AssignGroup(group), Press(digit.with(Modifiers::CTRL))),
                (A::AddToGroup(group), Press(digit.with(Modifiers::SHIFT))),
                (A::RecallGroup(group), Press(digit)),
                (A::RecallAndCenterGroup(group), DoubleTap(digit)),
            ]);
        }
        entries.extend(CommandSlot::all().map(|slot| (A::CommandSlot(slot), Press(key(slot.default_key())))));
        // The map editor's. Esc, G, the arrow keys and the middle button are the shared actions above.
        let ctrl = |k| Press(key(k).with(Modifiers::CTRL));
        let ctrl_shift = Modifiers {
            shift: true,
            ..Modifiers::CTRL
        };
        entries.extend(
            [
                (E::UseTool, Press(Chord::mouse(MouseButton::Left))),
                (E::Eyedropper, Press(Chord::mouse(MouseButton::Right))),
                (E::Zoom, Wheel(Modifiers::NONE)),
                (E::SelectTool, Press(key(KeyCode::KeyV))),
                (E::PaintTool, Press(key(KeyCode::KeyB))),
                (E::HeightTool, Press(key(KeyCode::KeyH))),
                (E::FillTool, Press(key(KeyCode::KeyF))),
                (E::PlaceTool, Press(key(KeyCode::KeyO))),
                (E::StartTool, Press(key(KeyCode::KeyL))),
                (E::CameraTool, Press(key(KeyCode::KeyC))),
                (E::LayoutTool, Press(key(KeyCode::KeyT))),
                (E::SmallerBrush, Press(key(KeyCode::BracketLeft))),
                (E::LargerBrush, Press(key(KeyCode::BracketRight))),
                (E::NextHeightMode, Press(key(KeyCode::KeyE))),
                (E::RaiseLevel, Press(key(KeyCode::Equal))),
                (E::RaiseLevel, Press(key(KeyCode::NumpadAdd))),
                (E::LowerLevel, Press(key(KeyCode::Minus))),
                (E::LowerLevel, Press(key(KeyCode::NumpadSubtract))),
                (E::ToggleSlopes, Press(key(KeyCode::KeyK))),
                (E::FitMap, Press(key(KeyCode::Home))),
                (E::DeleteSelection, Press(key(KeyCode::Delete))),
                (E::DeleteSelection, Press(key(KeyCode::Backspace))),
                (E::Undo, ctrl(KeyCode::KeyZ)),
                (E::Redo, ctrl(KeyCode::KeyY)),
                (E::Redo, Press(key(KeyCode::KeyZ).with(ctrl_shift))),
                (E::Save, ctrl(KeyCode::KeyS)),
                (E::NewMap, ctrl(KeyCode::KeyN)),
                (E::PlayTest, Press(key(KeyCode::F5))),
                (E::ToggleHelp, Press(key(KeyCode::F1))),
                // Placement actions: only while a structure is being placed, over F's Fill tool in the editor and over
                // the command slots R and F in the game's AwaitingPlacement (they work in both contexts).
                (E::RotateClockwise, Press(key(KeyCode::KeyR))),
                (
                    E::RotateCounterClockwise,
                    Press(key(KeyCode::KeyR).with(Modifiers::SHIFT)),
                ),
                (E::FlipHorizontal, Press(key(KeyCode::KeyF))),
                (E::FlipVertical, Press(key(KeyCode::KeyF).with(Modifiers::SHIFT))),
            ]
            .map(|(input, binding)| (A::Editor(input), binding)),
        );
        Self { entries }
    }
}

/// Recognizes double taps: the same button with the same modifiers pressed twice within [`DOUBLE_TAP_SECONDS`].
/// Every bound press goes through it, so another one in between breaks the pair.
#[derive(Default, Debug)]
pub(crate) struct TapTracker {
    last: Option<(Button, Modifiers, f64)>,
}

impl TapTracker {
    /// Records a press at `now` (seconds) and says whether it completes a double tap. A completed double tap starts
    /// over, so a third quick press is a single one again.
    pub(crate) fn press(&mut self, button: Button, modifiers: Modifiers, now: f64) -> bool {
        let double = self
            .last
            .is_some_and(|(b, m, t)| b == button && m == modifiers && now - t <= DOUBLE_TAP_SECONDS);
        self.last = if double { None } else { Some((button, modifiers, now)) };
        double
    }
}

#[cfg(test)]
mod tests;
