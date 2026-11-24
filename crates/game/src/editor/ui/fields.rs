//! Text fields (Bevy's headless `TextInput`): the map's name, description and size, the play-tests' seed, a custom
//! preset's name, the creation dialog's settings, and the selected spawn's per-object values. A field commits its
//! text as an [`EditorAction`] on Enter or when it loses the keyboard (a click elsewhere), reverts on Escape, and
//! shows the editor's value whenever it doesn't have the keyboard (after an undo, for example).

use bevy::ecs::system::SystemParam;
use bevy::input_focus::InputFocus;
use bevy::input_focus::tab_navigation::TabIndex;
use bevy::prelude::*;
use bevy::text::{EditableText, EditableTextFilter, TextEdit};
use bevy::ui_widgets::TextInput;
use sc_sim::{MapDef, MapSettings, ObjectKind, SpawnSettings, Symmetry, TICKS_PER_SECOND, TilePresetId};

use super::{BORDER, TEXT, TEXT_VH};
use crate::editor::actions::{EditorAction, EditorActions};
use crate::editor::creation::{NewMapDialog, NewMapField};
use crate::editor::doc::EditorDoc;
use crate::editor::document::MAX_DESCRIPTION_CHARS;
use crate::editor::playtest::PlayTestSetup;
use crate::editor::presets::MAX_PRESET_NAME_CHARS;
use crate::editor::state::EditorStatus;

const FIELD_BACKGROUND: Color = Color::srgb(0.04, 0.05, 0.06);
const FIELD_FOCUSED: Color = Color::srgb(0.40, 0.62, 0.95);
/// Longest map name a field accepts.
const MAX_NAME_CHARS: usize = 40;

/// What a text field edits.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub enum TextField {
    MapName,
    Description,
    Width,
    Height,
    /// The play-tests' seed.
    Seed,
    /// A custom preset's name.
    PresetName(TilePresetId),
    NewMapName,
    NewMapDescription,
    NewMapWidth,
    NewMapHeight,
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
            ObjectKind::Peacekeeper
            | ObjectKind::Guard
            | ObjectKind::DeploymentCenter
            | ObjectKind::Generator
            | ObjectKind::Barracks
            | ObjectKind::ExtractionFacility
            | ObjectKind::ExtractionPlate
            | ObjectKind::Drone
            | ObjectKind::Beacon
            | ObjectKind::EgressNode
            | ObjectKind::RoboPortal
            | ObjectKind::BioPortal
            | ObjectKind::SabreLegionnaire
            | ObjectKind::RifleLegionnaire => &[],
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
fn spawn_settings(map: &MapDef, spawn: usize) -> Option<SpawnSettings> {
    let spawn = map.spawns.get(spawn)?;
    spawn
        .settings
        .filter(|s| s.kind() == spawn.kind)
        .or_else(|| SpawnSettings::defaults(spawn.kind))
}

/// What the fields show and edit.
#[derive(Copy, Clone)]
pub struct FieldContext<'a> {
    pub map: &'a MapDef,
    pub dialog: &'a MapSettings,
    pub playtest: &'a PlayTestSetup,
}

/// The resources a [`FieldContext`] comes from.
#[derive(SystemParam)]
pub struct FieldSources<'w> {
    doc: Res<'w, EditorDoc>,
    dialog: Res<'w, NewMapDialog>,
    playtest: Res<'w, PlayTestSetup>,
}

impl FieldSources<'_> {
    pub fn context(&self) -> FieldContext<'_> {
        FieldContext {
            map: self.doc.map(),
            dialog: &self.dialog.settings,
            playtest: &self.playtest,
        }
    }

    fn is_changed(&self) -> bool {
        self.doc.is_changed() || self.dialog.is_changed() || self.playtest.is_changed()
    }
}

impl TextField {
    /// The editor's value of the field.
    pub fn value(self, ctx: FieldContext) -> Option<String> {
        let map = ctx.map;
        Some(match self {
            TextField::MapName => map.name.clone(),
            TextField::Description => map.description.clone(),
            TextField::Width => map.terrain.width().to_string(),
            TextField::Height => map.terrain.height().to_string(),
            TextField::Seed => ctx.playtest.seed.to_string(),
            TextField::PresetName(id) => map.terrain.presets().get(id.0 as usize)?.name.clone(),
            TextField::NewMapName => ctx.dialog.name.clone(),
            TextField::NewMapDescription => ctx.dialog.description.clone(),
            TextField::NewMapWidth => ctx.dialog.width.to_string(),
            TextField::NewMapHeight => ctx.dialog.height.to_string(),
            TextField::SpawnValue { spawn, value } => value.get(spawn_settings(map, spawn)?)?,
        })
    }

    fn numeric(self) -> bool {
        matches!(
            self,
            TextField::Seed
                | TextField::Width
                | TextField::Height
                | TextField::NewMapWidth
                | TextField::NewMapHeight
                | TextField::SpawnValue { .. }
        )
    }

    fn max_characters(self) -> usize {
        match self {
            _ if self.numeric() => 20,
            TextField::Description | TextField::NewMapDescription => MAX_DESCRIPTION_CHARS,
            TextField::PresetName(_) => MAX_PRESET_NAME_CHARS,
            _ => MAX_NAME_CHARS,
        }
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

    /// The action that applies `text`: `None` when it is the editor's value already, an error message when it is no
    /// valid value.
    pub fn commit(self, text: &str, ctx: FieldContext) -> Result<Option<EditorAction>, String> {
        let text = text.trim();
        if self.value(ctx).as_deref() == Some(text) {
            return Ok(None);
        }
        let side = |what: &str| {
            text.parse::<u32>()
                .map_err(|_| format!("The map {what} is a whole number of tiles"))
        };
        let terrain = &ctx.map.terrain;
        let (width, height) = (terrain.width(), terrain.height());
        // A map with 90 degree rotation stays square: either side sets both.
        let square = ctx.map.symmetry == Symmetry::Rotate90;
        Ok(Some(match self {
            TextField::MapName => EditorAction::SetName(text.into()),
            TextField::Description => EditorAction::SetDescription(text.into()),
            TextField::Width => {
                let width = side("width")?;
                EditorAction::Resize {
                    width,
                    height: if square { width } else { height },
                }
            }
            TextField::Height => {
                let height = side("height")?;
                EditorAction::Resize {
                    width: if square { height } else { width },
                    height,
                }
            }
            TextField::Seed => {
                EditorAction::SetPlayTestSeed(text.parse().map_err(|_| "The seed is a whole number".to_string())?)
            }
            TextField::PresetName(id) => EditorAction::SetPresetName(id, text.into()),
            TextField::NewMapName => EditorAction::EditNewMap(NewMapField::Name(text.into())),
            TextField::NewMapDescription => EditorAction::EditNewMap(NewMapField::Description(text.into())),
            TextField::NewMapWidth => EditorAction::EditNewMap(NewMapField::Width(side("width")?)),
            TextField::NewMapHeight => EditorAction::EditNewMap(NewMapField::Height(side("height")?)),
            TextField::SpawnValue { spawn, value } => {
                let settings = spawn_settings(ctx.map, spawn).ok_or_else(|| format!("There is no spawn #{spawn}"))?;
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
            max_characters: Some(field.max_characters()),
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

/// Commits a field on Enter or when it loses the keyboard, except to Escape. Runs before the frame's actions apply,
/// and puts the commit first in the queue: a click that took the keyboard away applies after the text it ended
/// (review finding: the text was lost when that click rebuilt the field's list). Enter and Escape are the field's own
/// keys, read from the keyboard like the text typed into it, not bindings: while a field has the keyboard the input
/// layer fires none (`crate::input::TextFocus`).
pub fn commit_fields(
    focus: Option<ResMut<InputFocus>>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    sources: FieldSources,
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
    let ctx = sources.context();
    let mut commit = |entity: Entity| {
        let Ok((field, text)) = fields.get(entity) else {
            return;
        };
        match field.commit(&text.value().to_string(), ctx) {
            Ok(Some(action)) => actions.0.push_front(action),
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
    if *last_focused != current {
        *last_focused = current;
    }
}

/// Shows the editor's values in the fields without the keyboard, and marks the focused field. Only when a value or
/// the keyboard moved, or new fields appeared.
pub fn refresh_fields(
    focus: Option<Res<InputFocus>>,
    sources: FieldSources,
    actions: Res<EditorActions>,
    mut fields: Query<(Entity, Ref<TextField>, &mut EditableText, &mut BorderColor)>,
    mut focused_before: Local<Option<Entity>>,
    mut pending: Local<bool>,
) {
    let focused = focus.and_then(|f| f.get());
    let focus_moved = std::mem::replace(&mut *focused_before, focused) != focused;
    let added = fields.iter().any(|(_, field, ..)| field.is_added());
    let mut values = std::mem::take(&mut *pending) || sources.is_changed() || focus_moved || added;
    // Queued actions (a commit) may still change the values: refresh after them.
    if values && !actions.0.is_empty() {
        *pending = true;
        values = false;
    }
    if !values && !focus_moved && !added {
        return;
    }
    let ctx = sources.context();
    for (entity, field, mut text, mut border) in &mut fields {
        let has_focus = focused == Some(entity);
        border.set_if_neq(BorderColor::all(if has_focus { FIELD_FOCUSED } else { BORDER }));
        if has_focus || !values {
            continue;
        }
        let Some(value) = field.value(ctx) else {
            continue;
        };
        if text.value().to_string() != value {
            // From its start: a field too short for its value shows how it begins.
            text.editor.set_text(&value);
            text.queue_edit(TextEdit::TextStart(false));
        }
    }
}

#[cfg(test)]
mod tests;
