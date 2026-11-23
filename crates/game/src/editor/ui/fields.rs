//! Text fields (Bevy's headless `TextInput`): the map's name, seed and size, and the slots' names. A field
//! commits its text as an [`EditorAction`] on Enter or when it loses the keyboard (a click elsewhere), reverts on
//! Escape, and shows the document's value whenever it doesn't have the keyboard (after an undo, for example).

use bevy::input_focus::InputFocus;
use bevy::input_focus::tab_navigation::TabIndex;
use bevy::prelude::*;
use bevy::text::{EditableText, EditableTextFilter, TextEdit};
use bevy::ui_widgets::TextInput;
use sc_sim::{PlayerId, Scenario};

use super::{BORDER, TEXT, TEXT_VH};
use crate::editor::actions::{EditorAction, EditorActions};
use crate::editor::doc::EditorDoc;
use crate::editor::state::EditorStatus;

const FIELD_BACKGROUND: Color = Color::srgb(0.04, 0.05, 0.06);
const FIELD_FOCUSED: Color = Color::srgb(0.40, 0.62, 0.95);
/// Longest name a field accepts.
const MAX_NAME_CHARS: usize = 40;

/// What a text field edits.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub enum TextField {
    MapName,
    Seed,
    Width,
    Height,
    SlotName(PlayerId),
}

impl TextField {
    /// The document's value of the field.
    pub fn value(self, scenario: &Scenario) -> Option<String> {
        Some(match self {
            TextField::MapName => scenario.name.clone(),
            TextField::Seed => scenario.seed.to_string(),
            TextField::Width => scenario.map.width().to_string(),
            TextField::Height => scenario.map.height().to_string(),
            TextField::SlotName(slot) => scenario.players.iter().find(|p| p.id == slot)?.name.clone(),
        })
    }

    fn numeric(self) -> bool {
        matches!(self, TextField::Seed | TextField::Width | TextField::Height)
    }

    /// The action that applies `text`: `None` when it is the document's value already, an error message when it
    /// is no valid value.
    pub fn commit(self, text: &str, scenario: &Scenario) -> Result<Option<EditorAction>, String> {
        let text = text.trim();
        if self.value(scenario).as_deref() == Some(text) {
            return Ok(None);
        }
        let side = |what: &str| {
            text.parse::<u32>()
                .map_err(|_| format!("The map {what} is a whole number of tiles"))
        };
        let (width, height) = (scenario.map.width(), scenario.map.height());
        Ok(Some(match self {
            TextField::MapName => EditorAction::SetName(text.into()),
            TextField::Seed => {
                EditorAction::SetSeed(text.parse().map_err(|_| "The seed is a whole number".to_string())?)
            }
            TextField::Width => EditorAction::Resize {
                width: side("width")?,
                height,
            },
            TextField::Height => EditorAction::Resize {
                width,
                height: side("height")?,
            },
            TextField::SlotName(slot) => EditorAction::SetSlotName(slot, text.into()),
        }))
    }
}

/// A text field showing `value`, `width_vh` wide.
pub fn text_field(field: TextField, value: &str, width_vh: f32) -> impl Bundle {
    let filter = if field.numeric() {
        EditableTextFilter::new(|c: char| c.is_ascii_digit())
    } else {
        EditableTextFilter::default()
    };
    (
        field,
        TextInput,
        // Pointer clicks only focus entities with a tab index.
        TabIndex(0),
        EditableText {
            max_characters: Some(if field.numeric() { 20 } else { MAX_NAME_CHARS }),
            ..EditableText::new(value)
        },
        filter,
        TextFont::from_font_size(FontSize::Vh(TEXT_VH)),
        TextColor(TEXT),
        Node {
            width: vh(width_vh),
            padding: UiRect::axes(vh(0.5), vh(0.3)),
            border: UiRect::all(px(1)),
            ..default()
        },
        BackgroundColor(FIELD_BACKGROUND),
        BorderColor::all(BORDER),
    )
}

/// Commits a field on Enter or when it loses the keyboard, except to Escape.
pub fn commit_fields(
    focus: Option<ResMut<InputFocus>>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    doc: Res<EditorDoc>,
    fields: Query<(&TextField, &EditableText)>,
    mut last_focused: Local<Option<Entity>>,
    mut actions: ResMut<EditorActions>,
    mut status: ResMut<EditorStatus>,
) {
    let Some(mut focus) = focus else {
        return;
    };
    let pressed = |key| keys.as_ref().is_some_and(|k| k.just_pressed(key));
    let enter = pressed(KeyCode::Enter) || pressed(KeyCode::NumpadEnter);
    let mut commit = |entity: Entity| {
        let Ok((field, text)) = fields.get(entity) else {
            return;
        };
        match field.commit(&text.value().to_string(), doc.scenario()) {
            Ok(Some(action)) => actions.push(action),
            Ok(None) => {}
            Err(message) => status.warn(message),
        }
    };
    let mut current = focus.get().filter(|e| fields.contains(*e));
    if let Some(previous) = *last_focused
        && current != Some(previous)
        && !pressed(KeyCode::Escape)
    {
        commit(previous);
    }
    if let Some(entity) = current
        && enter
    {
        commit(entity);
        focus.clear();
        current = None;
    }
    *last_focused = current;
}

/// Shows the document's values in the fields without the keyboard, and marks the focused field.
pub fn refresh_fields(
    focus: Option<Res<InputFocus>>,
    doc: Res<EditorDoc>,
    actions: Res<EditorActions>,
    mut fields: Query<(Entity, &TextField, &mut EditableText, &mut BorderColor)>,
) {
    let focused = focus.and_then(|f| f.get());
    for (entity, field, mut text, mut border) in &mut fields {
        let has_focus = focused == Some(entity);
        border.set_if_neq(BorderColor::all(if has_focus { FIELD_FOCUSED } else { BORDER }));
        // Queued actions (a commit) may still change the value: refresh after them.
        if has_focus || !actions.0.is_empty() {
            continue;
        }
        let Some(value) = field.value(doc.scenario()) else {
            continue;
        };
        if text.value().to_string() != value {
            text.editor.set_text(&value);
            text.queue_edit(TextEdit::TextEnd(false));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sc_sim::scenarios;

    #[test]
    fn fields_commit_valid_changes_only() {
        let scenario = scenarios::empty(10, 8);
        let commit = |field: TextField, text: &str| field.commit(text, &scenario);
        assert_eq!(commit(TextField::MapName, " empty "), Ok(None), "unchanged");
        assert_eq!(
            commit(TextField::MapName, "Ridge"),
            Ok(Some(EditorAction::SetName("Ridge".into())))
        );
        assert_eq!(commit(TextField::Seed, "42"), Ok(Some(EditorAction::SetSeed(42))));
        assert!(commit(TextField::Seed, "").is_err());
        assert_eq!(
            commit(TextField::Width, "20"),
            Ok(Some(EditorAction::Resize { width: 20, height: 8 }))
        );
        assert_eq!(
            commit(TextField::Height, "3"),
            Ok(Some(EditorAction::Resize { width: 10, height: 3 }))
        );
        assert!(commit(TextField::Height, "99999999999").is_err());
        assert_eq!(
            commit(TextField::SlotName(PlayerId(2)), "Blue"),
            Ok(Some(EditorAction::SetSlotName(PlayerId(2), "Blue".into())))
        );
        assert_eq!(TextField::SlotName(PlayerId(7)).value(&scenario), None);
    }
}
