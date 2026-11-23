//! The bindings table (`ARCHITECTURE.md > Client architecture > Input`): every game binding in one resource, so
//! that rebinding (`systems/controls/selection_and_hud.md > Key Rebinding`) changes data, not gameplay code.
//!
//! Each entry binds an [`InputAction`] to a key or mouse button with the modifiers held with it (a [`Chord`]), to a
//! double tap of a chord, or to modifiers alone. A press fires at most one action, the one of its most specific
//! binding ([`Bindings::resolve`]): so Shift + 1 adds to control group 1 instead of recalling it, while Shift + Q
//! still presses command slot Q, with the queue modifier held.

use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::MouseButton;
use bevy::prelude::Resource;

use super::actions::{CommandSlot, ControlGroup, InputAction};

/// Two presses of the same chord within this many seconds are a double tap (`> ControlGroups > Default keys`:
/// "press the same number twice quickly"). Placeholder: the design gives no time.
pub const DOUBLE_TAP_SECONDS: f64 = 0.4;

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
}

impl Binding {
    /// The chord of a press or a double tap.
    pub fn chord(self) -> Option<Chord> {
        match self {
            Binding::Press(chord) | Binding::DoubleTap(chord) => Some(chord),
            Binding::Modifiers(_) => None,
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

    /// The action that a press of `button` fires while `modifiers` are held; `double` says whether the press
    /// completes a double tap. Of the bindings of that button whose modifiers are all held, the one requiring the
    /// most modifiers wins, then a double tap over a single press, then the earlier entry.
    pub fn resolve(&self, button: Button, modifiers: Modifiers, double: bool) -> Option<InputAction> {
        let mut best: Option<((u32, bool), InputAction)> = None;
        for &(action, binding) in &self.entries {
            let (chord, is_double) = match binding {
                Binding::Press(chord) => (chord, false),
                Binding::DoubleTap(chord) if double => (chord, true),
                Binding::DoubleTap(_) | Binding::Modifiers(_) => continue,
            };
            if chord.button != button || !modifiers.contains(chord.modifiers) {
                continue;
            }
            let rank = (chord.modifiers.count(), is_double);
            if best.is_none_or(|(best_rank, _)| rank > best_rank) {
                best = Some((rank, action));
            }
        }
        best.map(|(_, action)| action)
    }

    /// The actions that modifier bindings hold while `modifiers` are held.
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

/// The defaults: `core/camera.md > Panning`, `systems/controls/selection_and_hud.md` (`> MapClickSelection`,
/// `> AwaitingTarget`, `> GroupCycling`, `> ControlGroups > Default keys`, `> CommandPanel > Grid Layout`) and
/// `systems/controls/commands.md > Unit Command` (design changes 0002, 0010 and 0011).
impl Default for Bindings {
    fn default() -> Self {
        use Binding::{DoubleTap, Modifiers as Held, Press};
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
mod tests {
    use super::*;

    fn key(k: KeyCode) -> Button {
        Button::Key(k)
    }

    fn group(n: u8) -> ControlGroup {
        ControlGroup::new(n).unwrap()
    }

    #[test]
    fn the_defaults_follow_the_design() {
        let b = Bindings::default();
        let only = |action| b.of(action).collect::<Vec<_>>();
        let press = |k| vec![Binding::Press(Chord::key(k))];
        assert_eq!(only(InputAction::PanLeft), press(KeyCode::ArrowLeft));
        assert_eq!(only(InputAction::Cancel), press(KeyCode::Escape));
        assert_eq!(only(InputAction::NextGroup), press(KeyCode::Tab));
        assert_eq!(
            only(InputAction::PreviousGroup),
            [Binding::Press(Chord::key(KeyCode::Tab).with(Modifiers::SHIFT))]
        );
        assert_eq!(
            only(InputAction::GrabPan),
            [Binding::Press(Chord::mouse(MouseButton::Middle))]
        );
        assert_eq!(only(InputAction::Queue), [Binding::Modifiers(Modifiers::SHIFT)]);
        assert_eq!(only(InputAction::SelectSameType), [Binding::Modifiers(Modifiers::CTRL)]);
        let seven = Chord::key(KeyCode::Digit7);
        assert_eq!(
            only(InputAction::AssignGroup(group(7))),
            [Binding::Press(seven.with(Modifiers::CTRL))]
        );
        assert_eq!(
            only(InputAction::AddToGroup(group(7))),
            [Binding::Press(seven.with(Modifiers::SHIFT))]
        );
        assert_eq!(only(InputAction::RecallGroup(group(7))), [Binding::Press(seven)]);
        assert_eq!(
            only(InputAction::RecallAndCenterGroup(group(7))),
            [Binding::DoubleTap(seven)]
        );
        // The 4 × 3 grid, standard slots by hotkey letter (0002).
        let hotkeys: Vec<KeyCode> = CommandSlot::all().map(CommandSlot::default_key).collect();
        use KeyCode::*;
        assert_eq!(
            hotkeys,
            [KeyQ, KeyW, KeyE, KeyR, KeyA, KeyS, KeyD, KeyF, KeyZ, KeyX, KeyC, KeyV]
        );
        assert_eq!(only(InputAction::CommandSlot(CommandSlot::BACK)), press(KeyZ));
        assert_eq!(
            only(InputAction::CommandSlot(CommandSlot::CANCEL_PRODUCTION)),
            press(KeyX)
        );
        assert_eq!(only(InputAction::CommandSlot(CommandSlot::RALLY_POINT)), press(KeyC));
        assert_eq!(b.entries().len(), 15 + 4 * 10 + 12);
    }

    #[test]
    fn no_two_default_bindings_share_a_chord() {
        let b = Bindings::default();
        let mut seen: Vec<Binding> = Vec::new();
        for &(action, binding) in b.entries() {
            if matches!(binding, Binding::Modifiers(_)) {
                continue;
            }
            assert!(!seen.contains(&binding), "{action:?}: {binding:?} is bound twice");
            seen.push(binding);
        }
        // Every action is bound, each command slot and control group once per operation.
        for slot in CommandSlot::all() {
            assert_eq!(b.of(InputAction::CommandSlot(slot)).count(), 1);
        }
    }

    #[test]
    fn the_most_specific_binding_wins() {
        let b = Bindings::default();
        let one = key(KeyCode::Digit1);
        let resolve = |button, modifiers, double| b.resolve(button, modifiers, double);
        assert_eq!(
            resolve(one, Modifiers::NONE, false),
            Some(InputAction::RecallGroup(group(1)))
        );
        assert_eq!(
            resolve(one, Modifiers::CTRL, false),
            Some(InputAction::AssignGroup(group(1)))
        );
        assert_eq!(
            resolve(one, Modifiers::SHIFT, false),
            Some(InputAction::AddToGroup(group(1)))
        );
        // A double tap beats the single press, but not a chord with more modifiers.
        assert_eq!(
            resolve(one, Modifiers::NONE, true),
            Some(InputAction::RecallAndCenterGroup(group(1)))
        );
        assert_eq!(
            resolve(one, Modifiers::SHIFT, true),
            Some(InputAction::AddToGroup(group(1)))
        );
        // Modifiers nothing asks for don't block a binding: Shift + Q is slot Q, with the queue modifier held.
        let q = key(KeyCode::KeyQ);
        let slot_q = InputAction::CommandSlot(CommandSlot::new(0, 0).unwrap());
        assert_eq!(resolve(q, Modifiers::SHIFT, false), Some(slot_q));
        assert_eq!(
            resolve(key(KeyCode::Tab), Modifiers::SHIFT, false),
            Some(InputAction::PreviousGroup)
        );
        assert_eq!(
            resolve(key(KeyCode::Tab), Modifiers::CTRL, false),
            Some(InputAction::NextGroup)
        );
        // Equally specific bindings: the earlier entry wins (Ctrl + Shift + 1: Assign is listed before Add).
        let both = Modifiers {
            shift: true,
            ctrl: true,
            alt: false,
        };
        assert_eq!(resolve(one, both, false), Some(InputAction::AssignGroup(group(1))));
        assert_eq!(resolve(key(KeyCode::KeyO), Modifiers::NONE, false), None);
        assert_eq!(
            resolve(Button::Mouse(MouseButton::Left), Modifiers::SHIFT, false),
            Some(InputAction::Select)
        );
    }

    #[test]
    fn modifier_actions_are_held_with_their_modifiers() {
        let b = Bindings::default();
        let held = |m| b.modifier_actions(m).collect::<Vec<_>>();
        assert_eq!(held(Modifiers::NONE), []);
        assert_eq!(
            held(Modifiers::SHIFT),
            [InputAction::Queue, InputAction::ToggleSelection]
        );
        assert_eq!(held(Modifiers::CTRL), [InputAction::SelectSameType]);
        assert!(b.holds(InputAction::Queue, Modifiers::SHIFT));
        assert!(!b.holds(InputAction::Queue, Modifiers::ALT));
        assert!(!b.holds(InputAction::Cancel, Modifiers::SHIFT), "not a modifier action");
    }

    #[test]
    fn rebinding_changes_the_table_only() {
        let mut b = Bindings::default();
        b.unbind(InputAction::Queue);
        b.bind(InputAction::Queue, Binding::Modifiers(Modifiers::ALT));
        assert!(b.holds(InputAction::Queue, Modifiers::ALT));
        assert!(!b.holds(InputAction::Queue, Modifiers::SHIFT));
        b.unbind(InputAction::Cancel);
        assert_eq!(b.resolve(key(KeyCode::Escape), Modifiers::NONE, false), None);
        b.bind(InputAction::Cancel, Binding::Press(Chord::key(KeyCode::Backquote)));
        assert_eq!(
            b.resolve(key(KeyCode::Backquote), Modifiers::NONE, false),
            Some(InputAction::Cancel)
        );
    }

    #[test]
    fn double_taps_need_the_same_chord_quickly_and_start_over() {
        let mut taps = TapTracker::default();
        let one = key(KeyCode::Digit1);
        let two = key(KeyCode::Digit2);
        assert!(!taps.press(one, Modifiers::NONE, 10.0));
        assert!(taps.press(one, Modifiers::NONE, 10.0 + DOUBLE_TAP_SECONDS - 0.01));
        // A third quick press is a single one again, and a fourth completes another double tap.
        assert!(!taps.press(one, Modifiers::NONE, 10.5));
        assert!(taps.press(one, Modifiers::NONE, 10.6));
        // Too slow, another key in between, or other modifiers: no double tap.
        assert!(!taps.press(one, Modifiers::NONE, 20.0));
        assert!(!taps.press(one, Modifiers::NONE, 20.0 + DOUBLE_TAP_SECONDS + 0.01));
        assert!(!taps.press(two, Modifiers::NONE, 20.5));
        assert!(!taps.press(one, Modifiers::NONE, 20.6));
        assert!(!taps.press(one, Modifiers::SHIFT, 20.7));
    }
}
