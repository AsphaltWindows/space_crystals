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
    assert_eq!(only(InputAction::CenterOnObject), [Binding::Modifiers(Modifiers::ALT)]);
    assert_eq!(only(InputAction::DebugSwitchPlayer), press(KeyCode::F2));
    assert_eq!(only(InputAction::DebugToggleFog), press(KeyCode::F3));
    // (M5-MM2) `systems/controls/hud.md > Minimap > Alerts`: "Jump to the latest alert: Space"; the replay's pause
    // moved to K.
    assert_eq!(only(InputAction::JumpToAlert), press(KeyCode::Space));
    assert_eq!(only(InputAction::ReplayPause), press(KeyCode::KeyK));
    assert_eq!(only(InputAction::ReplayStep), press(KeyCode::Period));
    assert_eq!(only(InputAction::ReplaySlower), press(KeyCode::Minus));
    assert_eq!(only(InputAction::ReplayFaster), press(KeyCode::Equal));
    // (M5-S7) `core/camera.md > Window Mode` and `core/game_rules.md > Playtest menu (temporary)`.
    assert_eq!(
        only(InputAction::ToggleFullscreen),
        [
            Binding::Press(Chord::key(KeyCode::F11)),
            Binding::Press(Chord::key(KeyCode::Enter).with(Modifiers::ALT))
        ]
    );
    assert_eq!(only(InputAction::PlaytestMenu), press(KeyCode::F10));
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
    let game = b.entries().iter().filter(|(a, _)| !matches!(a, InputAction::Editor(_)));
    assert_eq!(game.count(), 26 + 4 * 10 + 12);
}

/// Except that a placement action's binding overrides another while a structure is being placed: the placement
/// actions' bindings don't share a chord among themselves, nor the others among themselves.
#[test]
fn no_two_default_bindings_of_a_context_share_a_chord() {
    let b = Bindings::default();
    for context in InputContext::ALL {
        let mut seen: Vec<(bool, Binding)> = Vec::new();
        for (action, binding) in b.in_context(context) {
            if matches!(binding, Binding::Modifiers(_)) {
                continue;
            }
            let entry = (action.is_placement(), binding);
            assert!(
                !seen.contains(&entry),
                "{context:?}: {action:?}: {binding:?} is bound twice"
            );
            seen.push(entry);
        }
    }
    // Every action is bound, each command slot and control group once per operation.
    for slot in CommandSlot::all() {
        assert_eq!(b.of(InputAction::CommandSlot(slot)).count(), 1);
    }
}

/// The editor's keys (its F1 help): its own in the editor, the camera's, the grid's and Cancel shared with the
/// game, and the play-test switch in both, so that F5 also ends a play-test.
#[test]
fn the_editor_has_its_own_bindings_and_shares_some() {
    use InputContext::{Editor, Game};
    let b = Bindings::default();
    let ed = InputAction::Editor;
    let press = |context, k, modifiers| b.resolve(context, key(k), modifiers, false);
    assert_eq!(
        press(Editor, KeyCode::KeyV, Modifiers::NONE),
        Some(ed(EditorInput::SelectTool))
    );
    assert_eq!(
        press(Game, KeyCode::KeyV, Modifiers::NONE),
        Some(InputAction::CommandSlot(CommandSlot::new(2, 3).unwrap()))
    );
    assert_eq!(
        press(Editor, KeyCode::KeyZ, Modifiers::NONE),
        None,
        "Z alone is nothing in the editor"
    );
    assert_eq!(
        press(Editor, KeyCode::KeyZ, Modifiers::CTRL),
        Some(ed(EditorInput::Undo))
    );
    let ctrl_shift = Modifiers {
        shift: true,
        ..Modifiers::CTRL
    };
    assert_eq!(press(Editor, KeyCode::KeyZ, ctrl_shift), Some(ed(EditorInput::Redo)));
    assert_eq!(
        press(Editor, KeyCode::KeyY, Modifiers::CTRL),
        Some(ed(EditorInput::Redo))
    );
    assert_eq!(
        press(Editor, KeyCode::NumpadAdd, Modifiers::NONE),
        Some(ed(EditorInput::RaiseLevel))
    );
    assert_eq!(
        press(Editor, KeyCode::Backspace, Modifiers::NONE),
        Some(ed(EditorInput::DeleteSelection))
    );
    assert_eq!(
        press(Editor, KeyCode::Digit1, Modifiers::NONE),
        None,
        "no control groups in the editor"
    );
    assert_eq!(
        press(Game, KeyCode::F1, Modifiers::NONE),
        None,
        "no editor help in the game"
    );
    for context in InputContext::ALL {
        assert_eq!(
            press(context, KeyCode::Escape, Modifiers::NONE),
            Some(InputAction::Cancel)
        );
        assert_eq!(
            press(context, KeyCode::KeyG, Modifiers::NONE),
            Some(InputAction::ToggleGrid)
        );
        assert_eq!(
            press(context, KeyCode::F5, Modifiers::NONE),
            Some(ed(EditorInput::PlayTest))
        );
        assert_eq!(
            b.resolve(context, Button::Mouse(MouseButton::Middle), Modifiers::NONE, false),
            Some(InputAction::GrabPan)
        );
    }
    let left = Button::Mouse(MouseButton::Left);
    assert_eq!(
        b.resolve(Editor, left, Modifiers::NONE, false),
        Some(ed(EditorInput::UseTool))
    );
    assert_eq!(b.resolve(Game, left, Modifiers::NONE, false), Some(InputAction::Select));
    assert_eq!(b.resolve_wheel(Editor, Modifiers::NONE), Some(ed(EditorInput::Zoom)));
    assert_eq!(b.resolve_wheel(Editor, Modifiers::CTRL), Some(ed(EditorInput::Zoom)));
    assert_eq!(b.resolve_wheel(Game, Modifiers::NONE), None, "the game has no zoom");
    // In the editor Ctrl + key is a command of its own: a held Ctrl blocks the editor's other keys, not its mouse
    // buttons, the shared keys or anything in the game; Shift and Alt never block.
    let ctrl = Modifiers::CTRL;
    assert_eq!(press(Editor, KeyCode::KeyB, ctrl), None, "Ctrl + B is no Paint tool");
    assert_eq!(press(Editor, KeyCode::F5, ctrl), None);
    assert_eq!(
        press(Game, KeyCode::F5, ctrl),
        Some(ed(EditorInput::PlayTest)),
        "a play-test"
    );
    assert_eq!(press(Editor, KeyCode::KeyS, ctrl_shift), Some(ed(EditorInput::Save)));
    assert_eq!(press(Editor, KeyCode::KeyG, ctrl), Some(InputAction::ToggleGrid));
    assert_eq!(b.resolve(Editor, left, ctrl, false), Some(ed(EditorInput::UseTool)));
    assert_eq!(
        press(Editor, KeyCode::Equal, Modifiers::SHIFT),
        Some(ed(EditorInput::RaiseLevel)),
        "+ is Shift + = on many keyboards"
    );
    assert_eq!(
        press(Editor, KeyCode::KeyB, Modifiers::ALT),
        Some(ed(EditorInput::PaintTool))
    );
    let slot_c = InputAction::CommandSlot(CommandSlot::new(2, 2).unwrap());
    assert_eq!(press(Game, KeyCode::KeyC, ctrl), Some(slot_c));
    assert_eq!(ed(EditorInput::Undo).context(), Some(Editor));
    assert!(InputAction::Queue.works_in(Game) && !InputAction::Queue.works_in(Editor));
}

/// The editor places with the keys of the game's placement ghost (`factions/gdo/structures/deployment_center.md >
/// ObjectInterfaceState[DeploymentCenter] > AwaitingPlacement`): while a structure is being placed, R and Shift + R
/// turn it and F and Shift + F flip it, F over the Fill tool; while none is, they are nothing or the Fill tool.
#[test]
fn placement_keys_work_only_while_placing_and_override_the_others() {
    use InputContext::{Editor, Game};
    let b = Bindings::default();
    let ed = InputAction::Editor;
    let press = |k, modifiers, placing| b.resolve_placing(Editor, key(k), modifiers, false, placing);
    let shift = Modifiers::SHIFT;
    assert_eq!(
        press(KeyCode::KeyR, Modifiers::NONE, true),
        Some(ed(EditorInput::RotateClockwise))
    );
    assert_eq!(
        press(KeyCode::KeyR, shift, true),
        Some(ed(EditorInput::RotateCounterClockwise))
    );
    assert_eq!(
        press(KeyCode::KeyF, Modifiers::NONE, true),
        Some(ed(EditorInput::FlipHorizontal))
    );
    assert_eq!(press(KeyCode::KeyF, shift, true), Some(ed(EditorInput::FlipVertical)));
    assert_eq!(press(KeyCode::KeyR, Modifiers::NONE, false), None);
    assert_eq!(press(KeyCode::KeyR, shift, false), None);
    assert_eq!(
        press(KeyCode::KeyF, Modifiers::NONE, false),
        Some(ed(EditorInput::FillTool))
    );
    assert_eq!(
        press(KeyCode::KeyF, shift, false),
        Some(ed(EditorInput::FillTool)),
        "Shift doesn't block"
    );
    assert_eq!(
        b.resolve(Editor, key(KeyCode::KeyF), Modifiers::NONE, false),
        Some(ed(EditorInput::FillTool)),
        "resolve: nothing is being placed"
    );
    // The other keys work as ever while placing; Ctrl still blocks the editor's keys.
    assert_eq!(
        press(KeyCode::KeyV, Modifiers::NONE, true),
        Some(ed(EditorInput::SelectTool))
    );
    assert_eq!(press(KeyCode::KeyR, Modifiers::CTRL, true), None);
    assert!(ed(EditorInput::FlipVertical).is_placement() && !ed(EditorInput::FillTool).is_placement());
    // One definition for the game and the editor: the placement keys work in either context.
    for action in [
        EditorInput::RotateClockwise,
        EditorInput::RotateCounterClockwise,
        EditorInput::FlipHorizontal,
        EditorInput::FlipVertical,
    ] {
        assert!(ed(action).works_in(Game) && ed(action).works_in(Editor), "{action:?}");
    }
}

/// Risk R11 (plan decision T10): in the game's AwaitingPlacement (`factions/gdo/structures/deployment_center.md >
/// ObjectInterfaceState[DeploymentCenter] > AwaitingPlacement`) R and F turn the ghost instead of pressing the command
/// slots R and F, and Shift + R and Shift + F turn it the other way although Shift is the Queue modifier; otherwise R
/// and F are the command slots, with or without Shift (Shift + R is slot R with the queue modifier held), and Shift
/// still holds Queue in both.
#[test]
fn the_games_placement_keys_turn_the_ghost_over_the_command_slots() {
    use InputContext::Game;
    let b = Bindings::default();
    let ed = InputAction::Editor;
    let press = |k, modifiers, placing| b.resolve_placing(Game, key(k), modifiers, false, placing);
    let (none, shift) = (Modifiers::NONE, Modifiers::SHIFT);
    let slot_r = InputAction::CommandSlot(CommandSlot::new(0, 3).unwrap());
    let slot_f = InputAction::CommandSlot(CommandSlot::new(1, 3).unwrap());
    // Placing.
    assert_eq!(press(KeyCode::KeyR, none, true), Some(ed(EditorInput::RotateClockwise)));
    assert_eq!(
        press(KeyCode::KeyR, shift, true),
        Some(ed(EditorInput::RotateCounterClockwise))
    );
    assert_eq!(press(KeyCode::KeyF, none, true), Some(ed(EditorInput::FlipHorizontal)));
    assert_eq!(press(KeyCode::KeyF, shift, true), Some(ed(EditorInput::FlipVertical)));
    // In the game a held modifier is a role and never blocks: Ctrl + R still turns.
    assert_eq!(
        press(KeyCode::KeyR, Modifiers::CTRL, true),
        Some(ed(EditorInput::RotateClockwise))
    );
    // The other keys are the game's as ever: Q, Z (Back), X, Esc.
    assert_eq!(
        press(KeyCode::KeyQ, none, true),
        Some(InputAction::CommandSlot(CommandSlot::new(0, 0).unwrap()))
    );
    assert_eq!(
        press(KeyCode::KeyZ, none, true),
        Some(InputAction::CommandSlot(CommandSlot::BACK))
    );
    assert_eq!(press(KeyCode::Escape, none, true), Some(InputAction::Cancel));
    // Not placing.
    assert_eq!(press(KeyCode::KeyR, none, false), Some(slot_r));
    assert_eq!(press(KeyCode::KeyR, shift, false), Some(slot_r));
    assert_eq!(press(KeyCode::KeyF, none, false), Some(slot_f));
    assert_eq!(press(KeyCode::KeyF, shift, false), Some(slot_f));
    assert_eq!(b.resolve(Game, key(KeyCode::KeyR), none, false), Some(slot_r));
    // Shift holds Queue whatever is placed: the placement keys are presses, Queue a modifier binding.
    assert!(b.holds(InputAction::Queue, shift));
}

#[test]
fn the_most_specific_binding_wins() {
    let b = Bindings::default();
    let one = key(KeyCode::Digit1);
    let resolve = |button, modifiers, double| b.resolve(InputContext::Game, button, modifiers, double);
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
    assert_eq!(held(Modifiers::ALT), [InputAction::CenterOnObject]);
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
    let game = InputContext::Game;
    assert_eq!(b.resolve(game, key(KeyCode::Escape), Modifiers::NONE, false), None);
    b.bind(InputAction::Cancel, Binding::Press(Chord::key(KeyCode::Backquote)));
    assert_eq!(
        b.resolve(game, key(KeyCode::Backquote), Modifiers::NONE, false),
        Some(InputAction::Cancel)
    );
    // The editor's bindings are rebound the same way.
    let zoom = InputAction::Editor(EditorInput::Zoom);
    b.unbind(zoom);
    b.bind(zoom, Binding::Wheel(Modifiers::CTRL));
    assert_eq!(b.resolve_wheel(InputContext::Editor, Modifiers::NONE), None);
    assert_eq!(b.resolve_wheel(InputContext::Editor, Modifiers::CTRL), Some(zoom));
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

/// (M5-S7) The window's full-screen toggle (`core/camera.md > Window Mode`) works in the game and in the editor, by F11
/// and by Alt + Enter, and Enter alone is nothing; the playtest menu's F10 (`core/game_rules.md > Playtest menu
/// (temporary)`) works in the game only.
#[test]
fn f11_and_alt_enter_toggle_full_screen_everywhere_and_f10_is_the_games() {
    use InputContext::{Editor, Game};
    let b = Bindings::default();
    let press = |context, k, modifiers| b.resolve(context, key(k), modifiers, false);
    for context in [Game, Editor] {
        assert_eq!(
            press(context, KeyCode::F11, Modifiers::NONE),
            Some(InputAction::ToggleFullscreen)
        );
        assert_eq!(
            press(context, KeyCode::Enter, Modifiers::ALT),
            Some(InputAction::ToggleFullscreen)
        );
        assert_eq!(press(context, KeyCode::Enter, Modifiers::NONE), None);
    }
    assert_eq!(
        press(Game, KeyCode::F10, Modifiers::NONE),
        Some(InputAction::PlaytestMenu)
    );
    assert_eq!(press(Editor, KeyCode::F10, Modifiers::NONE), None);
}

/// (M5-MM2) Design change 0443: Space jumps to the latest alert in the game (a game, a play-test and a replay's
/// playback, whose input context is the game's) and is nothing in the editor; the replay's pause, a placeholder that
/// had Space, is K in the game, where nothing else has it, while K stays the editor's slope tint.
#[test]
fn space_jumps_to_the_latest_alert_in_the_game_and_k_pauses_a_replay() {
    use InputContext::{Editor, Game};
    let b = Bindings::default();
    let press = |context, k, modifiers| b.resolve(context, key(k), modifiers, false);
    assert_eq!(
        press(Game, KeyCode::Space, Modifiers::NONE),
        Some(InputAction::JumpToAlert)
    );
    assert_eq!(
        press(Game, KeyCode::Space, Modifiers::SHIFT),
        Some(InputAction::JumpToAlert),
        "Shift, the Queue modifier, doesn't block it"
    );
    assert_eq!(press(Editor, KeyCode::Space, Modifiers::NONE), None);
    assert_eq!(
        press(Game, KeyCode::KeyK, Modifiers::NONE),
        Some(InputAction::ReplayPause)
    );
    assert_eq!(
        press(Editor, KeyCode::KeyK, Modifiers::NONE),
        Some(InputAction::Editor(EditorInput::ToggleSlopes))
    );
    assert!(
        !InputAction::JumpToAlert.passes_modal(),
        "not while the playtest menu holds the input"
    );
}
