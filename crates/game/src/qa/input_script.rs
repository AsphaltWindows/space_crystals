//! `--input-script`: the virtual keyboard and mouse ([`InjectedInput`]) driven at given ticks, for QA screenshots of
//! what only input shows, such as a selection box being dragged. The sim waits at each step's tick until the step
//! has been applied ([`super::QaState::next_stop`]), so a step lands at its tick however the frames fall.
//!
//! (Slice M5-S7) One step a frame: the steps of one tick apply in consecutive frames, in file order, the sim waiting at
//! the tick for the last of them, so that each step's input is read before the next one's. A step that opens the
//! playtest menu (F10, `crate::playtest_menu`) pauses the game, and the one that clicks in the menu comes at the same
//! tick, in a frame of its own, once the menu holds the input.

use std::path::Path;

use bevy::prelude::*;
use sc_sim::Tick;
use serde::Deserialize;

use super::QaState;
use crate::input::InjectedInput;
use crate::sim_driver::SimRes;

/// One entry of an `--input-script` file: what the virtual keyboard and mouse do in the first frame at `tick`, in
/// this order: the cursor moves, buttons go down, buttons are tapped (down and up before the frame), buttons go up.
/// A button stays down until a later step releases it. E.g. `{"tick": 3, "cursor": [640, 300], "press":
/// ["MouseLeft"]}`, then `{"tick": 5, "cursor": [800, 420]}`, then `{"tick": 8, "release": ["MouseLeft"]}` drags a
/// box; `{"tick": 9, "press": ["ControlLeft"], "tap": ["Digit1"], "release": ["ControlLeft"]}` is Ctrl+1.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputStep {
    pub tick: Tick,
    /// Where the cursor goes, in logical pixels from the surface's top-left corner.
    #[serde(default)]
    pub cursor: Option<Vec2>,
    #[serde(default)]
    pub press: Vec<ScriptButton>,
    #[serde(default)]
    pub tap: Vec<ScriptButton>,
    #[serde(default)]
    pub release: Vec<ScriptButton>,
}

/// A key or mouse button, by name: `MouseLeft`, `MouseRight`, `MouseMiddle`, or a key as Bevy's `KeyCode` names it,
/// among `KeyA`–`KeyZ`, `Digit0`–`Digit9`, `F1`–`F12`, `ShiftLeft`, `ControlLeft`, `AltLeft` (and their `Right`
/// twins), `Tab`, `Escape`, `Enter`, `Space`, `Period`, `Minus`, `Equal` and `ArrowLeft`/`Right`/`Up`/`Down`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(try_from = "String")]
pub enum ScriptButton {
    Key(KeyCode),
    Mouse(MouseButton),
}

impl TryFrom<String> for ScriptButton {
    type Error = String;

    fn try_from(name: String) -> Result<Self, String> {
        match name.as_str() {
            "MouseLeft" => Ok(ScriptButton::Mouse(MouseButton::Left)),
            "MouseRight" => Ok(ScriptButton::Mouse(MouseButton::Right)),
            "MouseMiddle" => Ok(ScriptButton::Mouse(MouseButton::Middle)),
            _ => key_named(&name)
                .map(ScriptButton::Key)
                .ok_or_else(|| format!("unknown button {name:?}")),
        }
    }
}

impl ScriptButton {
    fn press(self, input: &mut InjectedInput) {
        match self {
            ScriptButton::Key(key) => input.keys.press(key),
            ScriptButton::Mouse(button) => input.mouse.press(button),
        }
    }

    fn release(self, input: &mut InjectedInput) {
        match self {
            ScriptButton::Key(key) => input.keys.release(key),
            ScriptButton::Mouse(button) => input.mouse.release(button),
        }
    }
}

/// The key a script names ([`ScriptButton`]).
fn key_named(name: &str) -> Option<KeyCode> {
    use KeyCode::*;
    const DIGITS: [KeyCode; 10] = [
        Digit0, Digit1, Digit2, Digit3, Digit4, Digit5, Digit6, Digit7, Digit8, Digit9,
    ];
    const LETTERS: [KeyCode; 26] = [
        KeyA, KeyB, KeyC, KeyD, KeyE, KeyF, KeyG, KeyH, KeyI, KeyJ, KeyK, KeyL, KeyM, KeyN, KeyO, KeyP, KeyQ, KeyR,
        KeyS, KeyT, KeyU, KeyV, KeyW, KeyX, KeyY, KeyZ,
    ];
    const FUNCTION: [KeyCode; 12] = [F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12];
    let indexed = |prefix: &str, base: usize| {
        name.strip_prefix(prefix)
            .filter(|n| !n.starts_with('0') || n.len() == 1)
            .and_then(|n| n.parse::<usize>().ok())
            .and_then(|n| n.checked_sub(base))
    };
    if let Some(d) = indexed("Digit", 0) {
        return DIGITS.get(d).copied();
    }
    if let Some(f) = indexed("F", 1) {
        return FUNCTION.get(f).copied();
    }
    if let Some(letter) = name.strip_prefix("Key")
        && let [c @ b'A'..=b'Z'] = letter.as_bytes()
    {
        return Some(LETTERS[usize::from(c - b'A')]);
    }
    Some(match name {
        "ShiftLeft" => ShiftLeft,
        "ShiftRight" => ShiftRight,
        "ControlLeft" => ControlLeft,
        "ControlRight" => ControlRight,
        "AltLeft" => AltLeft,
        "AltRight" => AltRight,
        "Tab" => Tab,
        "Escape" => Escape,
        "Enter" => Enter,
        "Space" => Space,
        "Period" => Period,
        "Minus" => Minus,
        "Equal" => Equal,
        "ArrowLeft" => ArrowLeft,
        "ArrowRight" => ArrowRight,
        "ArrowUp" => ArrowUp,
        "ArrowDown" => ArrowDown,
        _ => return None,
    })
}

/// Reads an `--input-script` file: a JSON list of [`InputStep`]s, any order (the same tick keeps file order).
pub fn load_input_script(path: &Path) -> Result<Vec<InputStep>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("can't read {}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{} is not a list of input steps: {e}", path.display()))
}

/// Applies the next step of the sim's tick to the virtual keyboard and mouse, before the input layer reads them: one
/// a frame (module docs).
pub(super) fn apply_input_script(mut qa: ResMut<QaState>, sim: Res<SimRes>, mut input: ResMut<InjectedInput>) {
    let tick = sim.tick();
    if let Some(step) = qa.input_steps.pop_front_if(|s| s.tick <= tick) {
        if let Some(cursor) = step.cursor {
            input.cursor = Some(cursor);
        }
        for &button in &step.press {
            button.press(&mut input);
        }
        for &button in &step.tap {
            button.press(&mut input);
            button.release(&mut input);
        }
        for &button in &step.release {
            button.release(&mut input);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buttons_go_by_their_names() {
        let button = |name: &str| ScriptButton::try_from(name.to_string());
        assert_eq!(button("MouseLeft"), Ok(ScriptButton::Mouse(MouseButton::Left)));
        assert_eq!(button("Digit0"), Ok(ScriptButton::Key(KeyCode::Digit0)));
        assert_eq!(button("Digit9"), Ok(ScriptButton::Key(KeyCode::Digit9)));
        assert_eq!(button("KeyQ"), Ok(ScriptButton::Key(KeyCode::KeyQ)));
        assert_eq!(button("F2"), Ok(ScriptButton::Key(KeyCode::F2)));
        assert_eq!(button("F12"), Ok(ScriptButton::Key(KeyCode::F12)));
        assert_eq!(button("ControlLeft"), Ok(ScriptButton::Key(KeyCode::ControlLeft)));
        for bad in ["Digit10", "Digit01", "F0", "F13", "Keyq", "KeyAB", "Mouse", "Shift"] {
            assert!(button(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn steps_read_from_json_with_defaults() {
        let steps: Vec<InputStep> = serde_json::from_str(
            r#"[{"tick": 3, "cursor": [640, 300], "press": ["MouseLeft"]}, {"tick": 9, "tap": ["Tab"]}]"#,
        )
        .unwrap();
        assert_eq!(steps[0].cursor, Some(Vec2::new(640.0, 300.0)));
        assert_eq!(steps[0].press, [ScriptButton::Mouse(MouseButton::Left)]);
        assert!(steps[1].press.is_empty() && steps[1].release.is_empty());
        assert_eq!(steps[1].tap, [ScriptButton::Key(KeyCode::Tab)]);
        let unknown = serde_json::from_str::<Vec<InputStep>>(r#"[{"tick": 1, "click": []}]"#);
        assert!(unknown.is_err());
    }
}
