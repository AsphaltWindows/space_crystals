//! The editor's UI (bevy_ui with the headless `bevy_ui_widgets`): a toolbar at the top, the palette on the left
//! (terrain and custom presets, height, objects, owner, player slots), the map's properties, the play-test setup,
//! validation and files on the right, a status line at the bottom, and the creation dialog for new maps over it
//! all. Sizes are in `vh` like the game HUD's; colours and sizes are placeholders.
//!
//! Every button carries the [`EditorAction`] it queues ([`UiAction`]) and lights up when that action's choice is
//! the current one. Text fields commit on Enter or when they lose the keyboard, and revert on Escape
//! ([`fields`]); they commit before the frame's actions apply, so that a click elsewhere applies after the text.
//! Lists that depend on the document are rebuilt when what they show changes ([`lists`]), and nothing in the UI
//! is touched in a frame in which nothing it shows changed.

pub mod dialog;
pub mod fields;
pub mod layout;
pub mod lists;
pub mod texts;

use bevy::ecs::system::SystemParam;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::Pressed;
use bevy::ui_widgets::{Activate, Button};
use sc_sim::MapSettings;

use super::EditorSystems;
use super::actions::{EditorAction, EditorActions};
use super::creation::{NewMapDialog, NewMapField};
use super::doc::EditorDoc;
use super::playtest::PlayTestSetup;
use super::state::{EditorSelection, EditorUi};
use super::tools::{EditorTools, Tool};
use crate::camera::ViewportInsets;
use crate::map_render::MapView;
use crate::{ClientMode, ClientSystems};

/// Height of the toolbar, in percent of the surface height.
pub const TOOLBAR_VH: f32 = 5.0;
/// Height of the status line.
pub const STATUS_VH: f32 = 3.4;
/// Width of the left palette.
pub const LEFT_PANEL_VH: f32 = 31.0;
/// Width of the right panel.
pub const RIGHT_PANEL_VH: f32 = 31.0;
/// The world view lies between the editor's panels.
pub const EDITOR_INSETS: ViewportInsets = ViewportInsets {
    top_vh: TOOLBAR_VH,
    bottom_vh: STATUS_VH,
    left_vh: LEFT_PANEL_VH,
    right_vh: RIGHT_PANEL_VH,
};

pub const TEXT_VH: f32 = 1.7;
pub const SMALL_VH: f32 = 1.5;
pub const HEADER_VH: f32 = 1.9;
pub const GAP_VH: f32 = 0.5;

pub const PANEL_BACKGROUND: Color = Color::srgb(0.07, 0.08, 0.10);
pub const BORDER: Color = Color::srgb(0.24, 0.27, 0.32);
pub const TEXT: Color = Color::srgb(0.88, 0.90, 0.93);
pub const DIM_TEXT: Color = Color::srgb(0.55, 0.60, 0.66);
pub const HEADER_TEXT: Color = Color::srgb(0.62, 0.78, 0.95);
pub const WARN_TEXT: Color = Color::srgb(0.98, 0.76, 0.32);
pub const ERROR_TEXT: Color = Color::srgb(1.0, 0.45, 0.40);
pub const OK_TEXT: Color = Color::srgb(0.50, 0.85, 0.55);
const BUTTON: Color = Color::srgb(0.15, 0.17, 0.21);
const BUTTON_HOVER: Color = Color::srgb(0.22, 0.25, 0.31);
const BUTTON_LIT: Color = Color::srgb(0.18, 0.36, 0.60);
const BUTTON_LIT_HOVER: Color = Color::srgb(0.24, 0.44, 0.70);
const BUTTON_PRESSED: Color = Color::srgb(0.10, 0.11, 0.14);
const BUTTON_DISABLED: Color = Color::srgb(0.10, 0.11, 0.13);
/// A choice that is current but can't be changed (a default preset's properties).
const BUTTON_LIT_DISABLED: Color = Color::srgb(0.13, 0.21, 0.32);
/// The opacity of a disabled button's text: dimmed, in its own colour.
const DISABLED_TEXT_ALPHA: f32 = 0.35;

/// The action a button queues when activated.
#[derive(Component, Clone, Debug)]
pub struct UiAction(pub EditorAction);

/// A top-level UI node shown only in this mode.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub struct ShownIn(pub ClientMode);

pub struct EditorUiPlugin;

impl Plugin for EditorUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(queue_button_action)
            .add_systems(Startup, layout::spawn_editor_ui)
            // Review finding: fields commit before the frame's actions apply and before any list is rebuilt, so
            // that neither a click elsewhere nor a rebuild loses the text.
            .add_systems(Update, fields::commit_fields.in_set(EditorSystems::Input))
            .add_systems(
                Update,
                (
                    (
                        lists::rebuild_presets,
                        lists::rebuild_preset_editor,
                        lists::rebuild_owners,
                        lists::rebuild_slots,
                        lists::rebuild_play_test,
                        lists::rebuild_files,
                        lists::rebuild_problems,
                        lists::rebuild_spawn_fields,
                    ),
                    fields::refresh_fields,
                    (
                        texts::update_status_line,
                        texts::update_panel_texts,
                        texts::show_overlays,
                    ),
                    (style_buttons, show_by_mode),
                )
                    .chain()
                    .in_set(ClientSystems::Present),
            );
    }
}

fn queue_button_action(activate: On<Activate>, buttons: Query<&UiAction>, mut actions: ResMut<EditorActions>) {
    if let Ok(action) = buttons.get(activate.entity) {
        actions.push(action.0.clone());
    }
}

/// What the buttons' looks depend on.
#[derive(SystemParam)]
pub struct UiState<'w> {
    pub tools: Res<'w, EditorTools>,
    pub doc: Res<'w, EditorDoc>,
    pub ui: Res<'w, EditorUi>,
    pub selection: Res<'w, EditorSelection>,
    pub map_view: Res<'w, MapView>,
    pub dialog: Res<'w, NewMapDialog>,
    pub playtest: Res<'w, PlayTestSetup>,
}

impl UiState<'_> {
    fn is_changed(&self) -> bool {
        self.tools.is_changed()
            || self.doc.is_changed()
            || self.ui.is_changed()
            || self.selection.is_changed()
            || self.map_view.is_changed()
            || self.dialog.is_changed()
            || self.playtest.is_changed()
    }

    /// Whether a button's action is the current choice (its button lights up).
    pub fn is_lit(&self, action: &EditorAction) -> bool {
        let tools = &self.tools;
        let map = self.doc.map();
        let preset = |id: &sc_sim::TilePresetId| map.terrain.presets().get(id.0 as usize);
        let settings = &self.dialog.settings;
        match action {
            EditorAction::SetTool(tool) => tools.tool == *tool,
            EditorAction::SetBrush(size) => tools.brush == *size,
            EditorAction::SetPreset(id) => tools.preset == *id,
            EditorAction::SetElevationMode(mode) => tools.tool == Tool::Elevation && tools.elevation_mode == *mode,
            EditorAction::SetKind(kind) => tools.kind == *kind,
            EditorAction::SetOwner(owner) => tools.owner == *owner,
            EditorAction::SelectSlot(slot) => tools.slot == *slot,
            EditorAction::TogglePresetFlag(id, flag) => preset(id).is_some_and(|p| flag.get(p)),
            EditorAction::SetPresetLook(id, look) => preset(id).is_some_and(|p| p.look == *look),
            EditorAction::SetPlayTestSlot(slot) => self.playtest.slot == *slot,
            EditorAction::Open(path) => self.doc.path() == Some(path.as_path()),
            EditorAction::EditNewMap(NewMapField::Type(t)) => settings.map_type == *t,
            EditorAction::EditNewMap(NewMapField::Symmetry(s)) => settings.symmetry == *s,
            EditorAction::EditNewMap(NewMapField::Slots(n)) => settings.slots == *n,
            EditorAction::ToggleHelp => self.ui.help,
            EditorAction::ToggleGrid => self.map_view.grid_visible,
            EditorAction::ToggleSlopes => self.ui.slopes,
            _ => false,
        }
    }

    /// Whether a button's action can do anything now.
    pub fn is_enabled(&self, action: &EditorAction) -> bool {
        let settings = &self.dialog.settings;
        match action {
            EditorAction::Undo => self.doc.can_undo(),
            EditorAction::Redo => self.doc.can_redo(),
            EditorAction::DeleteSelected => self.selection.0.is_some(),
            // The default presets are fixed.
            EditorAction::TogglePresetFlag(id, _)
            | EditorAction::SetPresetLook(id, _)
            | EditorAction::RemovePreset(id) => !id.is_default(),
            EditorAction::EditNewMap(NewMapField::Slots(n)) => MapSettings::slot_counts(settings.symmetry).contains(n),
            EditorAction::EditNewMap(NewMapField::Symmetry(sc_sim::Symmetry::None)) => {
                settings.map_type != sc_sim::MapType::SkirmishDefault
            }
            EditorAction::CreateMap => settings.check().is_ok(),
            _ => true,
        }
    }
}

/// What a button's look depends on, and its look.
type ButtonLook = (
    Entity,
    Ref<'static, UiAction>,
    Option<Ref<'static, Hovered>>,
    Option<Ref<'static, Pressed>>,
    &'static mut BackgroundColor,
);

/// Colours the buttons: disabled (with dimmed text), pressed, lit, hovered. Only the buttons whose look may have
/// changed: all of them when the editor's state changed, else the new ones and those hovered, pressed or released
/// (review finding).
fn style_buttons(
    state: UiState,
    mut released: RemovedComponents<Pressed>,
    mut buttons: Query<ButtonLook>,
    children: Query<&Children>,
    mut texts: Query<&mut TextColor>,
) {
    let everything = state.is_changed();
    let released: Vec<Entity> = released.read().collect();
    for (entity, action, hovered, pressed, mut background) in &mut buttons {
        let changed = everything
            || action.is_added()
            || hovered.as_ref().is_some_and(Ref::is_changed)
            || pressed.as_ref().is_some_and(Ref::is_added)
            || released.contains(&entity);
        if !changed {
            continue;
        }
        let hovered = hovered.is_some_and(|h| h.get());
        let lit = state.is_lit(&action.0);
        let enabled = state.is_enabled(&action.0);
        let alpha = if enabled { 1.0 } else { DISABLED_TEXT_ALPHA };
        for child in children.get(entity).into_iter().flatten() {
            if let Ok(mut text) = texts.get_mut(*child) {
                let dimmed = TextColor(text.0.with_alpha(alpha));
                text.set_if_neq(dimmed);
            }
        }
        let color = if !enabled {
            if lit { BUTTON_LIT_DISABLED } else { BUTTON_DISABLED }
        } else if pressed.is_some() {
            BUTTON_PRESSED
        } else {
            match (lit, hovered) {
                (true, false) => BUTTON_LIT,
                (true, true) => BUTTON_LIT_HOVER,
                (false, true) => BUTTON_HOVER,
                (false, false) => BUTTON,
            }
        };
        background.set_if_neq(BackgroundColor(color));
    }
}

fn show_by_mode(mode: Res<ClientMode>, mut roots: Query<(Ref<ShownIn>, &mut Visibility)>) {
    for (shown_in, mut visibility) in &mut roots {
        if !mode.is_changed() && !shown_in.is_added() {
            continue;
        }
        let shown = shown_in.0 == *mode;
        visibility.set_if_neq(if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}

/// A line of text.
pub fn label(content: impl Into<String>, size_vh: f32, color: Color) -> impl Bundle {
    (
        Text::new(content),
        TextFont::from_font_size(FontSize::Vh(size_vh)),
        TextColor(color),
    )
}

/// A section heading in a panel.
pub fn header(title: impl Into<String>) -> impl Bundle {
    (
        label(title, HEADER_VH, HEADER_TEXT),
        Node {
            margin: UiRect::top(vh(1.0)),
            ..default()
        },
    )
}

/// A row of controls that wraps when it runs out of room.
pub fn row() -> Node {
    Node {
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::Wrap,
        align_items: AlignItems::Center,
        column_gap: vh(GAP_VH),
        row_gap: vh(GAP_VH),
        ..default()
    }
}

/// A column of rows.
pub fn column() -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        row_gap: vh(0.3),
        ..default()
    }
}

/// A button that queues `action`, without content: spawn its content as children.
pub fn button_frame(action: EditorAction) -> impl Bundle {
    (
        Button,
        UiAction(action),
        Hovered::default(),
        Node {
            padding: UiRect::axes(vh(0.7), vh(0.35)),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: vh(0.8),
            border: UiRect::all(px(1)),
            ..default()
        },
        BackgroundColor(BUTTON),
        BorderColor::all(BORDER),
    )
}

/// A button showing `text` that queues `action`.
pub fn button(text: impl Into<String>, action: EditorAction) -> impl Bundle {
    (button_frame(action), children![label(text, TEXT_VH, TEXT)])
}

/// A small button, for rows of many choices.
pub fn small_button(text: impl Into<String>, action: EditorAction) -> impl Bundle {
    (button_frame(action), children![label(text, SMALL_VH, TEXT)])
}
