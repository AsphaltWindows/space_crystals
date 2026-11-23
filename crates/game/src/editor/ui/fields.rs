//! Text fields (Bevy's headless `TextInput`): the map's name, seed and size, the slots' names, and the selected
//! spawn's per-object values. A field commits its text as an [`EditorAction`] on Enter or when it loses the
//! keyboard (a click elsewhere), reverts on Escape, and shows the document's value whenever it doesn't have the
//! keyboard (after an undo, for example).

use bevy::input_focus::InputFocus;
use bevy::input_focus::tab_navigation::TabIndex;
use bevy::prelude::*;
use bevy::text::{EditableText, EditableTextFilter, TextEdit};
use bevy::ui_widgets::TextInput;
use sc_sim::{ObjectKind, PlayerId, Scenario, SpawnSettings, TICKS_PER_SECOND};

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
    /// One per-object value of the spawn at this index.
    SpawnValue {
        spawn: usize,
        value: SpawnValue,
    },
}

/// One per-object value (`core/map_editor.md > Per-object Resource Values`).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SpawnValue {
    /// A Space Crystals Patch's StartingAmount.
    StartingAmount,
    /// A Supply Delivery Station's StartingSupplies.
    StartingSupplies,
    /// A Supply Delivery Station's DeliverySize.
    DeliverySize,
    /// A Supply Delivery Station's DeliveryInterval, shown in seconds (stored in ticks).
    DeliveryInterval,
}

impl SpawnValue {
    /// The values objects of `kind` have.
    pub fn of(kind: ObjectKind) -> &'static [SpawnValue] {
        match kind {
            ObjectKind::CrystalPatch => &[SpawnValue::StartingAmount],
            ObjectKind::SupplyDeliveryStation => &[
                SpawnValue::StartingSupplies,
                SpawnValue::DeliverySize,
                SpawnValue::DeliveryInterval,
            ],
            ObjectKind::Peacekeeper | ObjectKind::Guard => &[],
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SpawnValue::StartingAmount => "Amount",
            SpawnValue::StartingSupplies => "Supplies",
            SpawnValue::DeliverySize => "Delivery",
            SpawnValue::DeliveryInterval => "Interval (s)",
        }
    }

    /// The value in `settings`, as the field shows it.
    fn get(self, settings: SpawnSettings) -> Option<String> {
        use SpawnSettings::{CrystalPatch, SupplyDeliveryStation};
        Some(match (self, settings) {
            (SpawnValue::StartingAmount, CrystalPatch { starting_amount }) => starting_amount.to_string(),
            (SpawnValue::StartingSupplies, SupplyDeliveryStation { starting_supplies, .. }) => {
                starting_supplies.to_string()
            }
            (SpawnValue::DeliverySize, SupplyDeliveryStation { delivery_size, .. }) => delivery_size.to_string(),
            (SpawnValue::DeliveryInterval, SupplyDeliveryStation { delivery_interval, .. }) => {
                // Ticks are sixteenths of a second, which an f64 shows exactly ("60", "7.5", "0.0625").
                (f64::from(delivery_interval) / f64::from(TICKS_PER_SECOND)).to_string()
            }
            _ => return None,
        })
    }

    /// `settings` with this value read from `text`; the action checks that the values make sense.
    fn set(self, settings: SpawnSettings, text: &str) -> Result<SpawnSettings, String> {
        use SpawnSettings::{CrystalPatch, SupplyDeliveryStation};
        let whole = || {
            text.parse::<u32>()
                .map_err(|_| format!("{} is a whole number", self.label()))
        };
        let mut settings = settings;
        match (self, &mut settings) {
            (SpawnValue::StartingAmount, CrystalPatch { starting_amount }) => *starting_amount = whole()?,
            (SpawnValue::StartingSupplies, SupplyDeliveryStation { starting_supplies, .. }) => {
                *starting_supplies = whole()?
            }
            (SpawnValue::DeliverySize, SupplyDeliveryStation { delivery_size, .. }) => *delivery_size = whole()?,
            (SpawnValue::DeliveryInterval, SupplyDeliveryStation { delivery_interval, .. }) => {
                let seconds = text
                    .parse::<f64>()
                    .ok()
                    .filter(|s| s.is_finite() && *s >= 0.0)
                    .ok_or_else(|| "The interval is a number of seconds".to_string())?;
                let ticks = (seconds * f64::from(TICKS_PER_SECOND)).round();
                if ticks > f64::from(u32::MAX) {
                    return Err("The interval is too long".into());
                }
                *delivery_interval = ticks as u32;
            }
            _ => return Err(format!("A {} has no {}", settings.kind().def().name, self.label())),
        }
        Ok(settings)
    }
}

/// The values of a spawn: its own, or its kind's defaults (`None` for kinds without values).
fn spawn_settings(scenario: &Scenario, spawn: usize) -> Option<SpawnSettings> {
    let spawn = scenario.spawns.get(spawn)?;
    spawn
        .settings
        .filter(|s| s.kind() == spawn.kind)
        .or_else(|| SpawnSettings::defaults(spawn.kind))
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
            TextField::SpawnValue { spawn, value } => value.get(spawn_settings(scenario, spawn)?)?,
        })
    }

    fn numeric(self) -> bool {
        matches!(
            self,
            TextField::Seed | TextField::Width | TextField::Height | TextField::SpawnValue { .. }
        )
    }

    /// The characters the field takes: digits for numbers, and a decimal point for seconds.
    fn filter(self) -> EditableTextFilter {
        match self {
            TextField::SpawnValue {
                value: SpawnValue::DeliveryInterval,
                ..
            } => EditableTextFilter::new(|c: char| c.is_ascii_digit() || c == '.'),
            _ if self.numeric() => EditableTextFilter::new(|c: char| c.is_ascii_digit()),
            _ => EditableTextFilter::default(),
        }
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
            TextField::SpawnValue { spawn, value } => {
                let settings = spawn_settings(scenario, spawn).ok_or_else(|| format!("There is no spawn #{spawn}"))?;
                EditorAction::SetSpawnSettings {
                    spawn,
                    settings: value.set(settings, text)?,
                }
            }
        }))
    }
}

/// A text field showing `value`, `width_vh` wide.
pub fn text_field(field: TextField, value: &str, width_vh: f32) -> impl Bundle {
    (
        field,
        TextInput,
        // Pointer clicks only focus entities with a tab index.
        TabIndex(0),
        EditableText {
            max_characters: Some(if field.numeric() { 20 } else { MAX_NAME_CHARS }),
            ..EditableText::new(value)
        },
        field.filter(),
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

    #[test]
    fn spawn_value_fields_show_and_commit_the_spawns_values() {
        use sc_sim::{Spawn, SpawnAt, TilePos};
        let mut scenario = scenarios::empty(10, 8);
        let at = SpawnAt::Tile(TilePos::new(1, 1));
        scenario.spawns = vec![
            Spawn::new(ObjectKind::CrystalPatch, None, at),
            Spawn::new(
                ObjectKind::SupplyDeliveryStation,
                None,
                SpawnAt::Tile(TilePos::new(4, 4)),
            ),
        ];
        let field = |spawn, value| TextField::SpawnValue { spawn, value };
        assert_eq!(
            field(0, SpawnValue::StartingAmount).value(&scenario),
            Some(sc_sim::scenario::DEFAULT_STARTING_AMOUNT.to_string())
        );
        assert_eq!(
            field(1, SpawnValue::DeliveryInterval).value(&scenario),
            Some("60".into())
        );
        assert_eq!(
            field(0, SpawnValue::DeliverySize).value(&scenario),
            None,
            "patches have no deliveries"
        );
        assert_eq!(field(5, SpawnValue::StartingAmount).value(&scenario), None);
        assert_eq!(
            field(0, SpawnValue::StartingAmount).commit("900", &scenario),
            Ok(Some(EditorAction::SetSpawnSettings {
                spawn: 0,
                settings: SpawnSettings::CrystalPatch { starting_amount: 900 }
            }))
        );
        // Seconds become ticks, rounded to the nearest tick.
        let Ok(Some(EditorAction::SetSpawnSettings { settings, .. })) =
            field(1, SpawnValue::DeliveryInterval).commit("7.5", &scenario)
        else {
            panic!("the interval should commit");
        };
        assert_eq!(
            settings,
            SpawnSettings::SupplyDeliveryStation {
                starting_supplies: sc_sim::scenario::DEFAULT_STARTING_SUPPLIES,
                delivery_size: sc_sim::scenario::DEFAULT_DELIVERY_SIZE,
                delivery_interval: 120,
            }
        );
        assert_eq!(SpawnValue::DeliveryInterval.get(settings), Some("7.5".into()));
        assert!(field(1, SpawnValue::DeliverySize).commit("lots", &scenario).is_err());
        assert!(
            field(1, SpawnValue::DeliveryInterval)
                .commit("1e99", &scenario)
                .is_err()
        );
        assert_eq!(
            field(1, SpawnValue::DeliveryInterval).commit("60", &scenario),
            Ok(None),
            "unchanged"
        );
    }
}
