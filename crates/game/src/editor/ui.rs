//! The editor's UI (bevy_ui with the headless `bevy_ui_widgets`): a toolbar at the top, the palette on the left
//! (terrain, elevation, objects, owner, player slots), properties and validation on the right, and a status line
//! at the bottom. Sizes are in `vh` like the game HUD's; colours and sizes are placeholders.
//!
//! Every button carries the [`EditorAction`] it queues ([`UiAction`]) and lights up when that action's choice is
//! the current one. Text fields commit on Enter or when they lose the keyboard, and revert on Escape
//! ([`fields`]). Lists that depend on the document are rebuilt when it changes ([`lists`]).

pub mod fields;
pub mod layout;
pub mod lists;
pub mod texts;

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::Pressed;
use bevy::ui_widgets::{Activate, Button};

use super::actions::{EditorAction, EditorActions};
use super::doc::EditorDoc;
use super::state::{EditorSelection, EditorUi};
use super::tools::{EditorTools, Tool};
use crate::camera::ViewportInsets;
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
            .add_systems(
                Update,
                (
                    (
                        lists::rebuild_presets,
                        lists::rebuild_owners,
                        lists::rebuild_slots,
                        lists::rebuild_files,
                        lists::rebuild_problems,
                    ),
                    (fields::commit_fields, fields::refresh_fields).chain(),
                    (texts::update_status_line, texts::update_panel_texts, texts::show_help),
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

/// Whether a button's action is the current choice (its button lights up).
pub fn is_lit(action: &EditorAction, tools: &EditorTools, doc: &EditorDoc, ui: &EditorUi) -> bool {
    match action {
        EditorAction::SetTool(tool) => tools.tool == *tool,
        EditorAction::SetBrush(size) => tools.brush == *size,
        EditorAction::SetSymmetry(symmetry) => tools.symmetry == *symmetry,
        EditorAction::SetPreset(preset) => tools.preset == *preset,
        EditorAction::SetElevationMode(mode) => tools.tool == Tool::Elevation && tools.elevation_mode == *mode,
        EditorAction::SetKind(kind) => tools.kind == *kind,
        EditorAction::SetOwner(owner) => tools.owner == *owner,
        EditorAction::SelectSlot(slot) => tools.slot == *slot,
        EditorAction::Open(path) => doc.path() == Some(path.as_path()),
        EditorAction::ToggleHelp => ui.help,
        _ => false,
    }
}

/// Whether a button's action can do anything now.
fn is_enabled(action: &EditorAction, doc: &EditorDoc, selection: &EditorSelection) -> bool {
    match action {
        EditorAction::Undo => doc.can_undo(),
        EditorAction::Redo => doc.can_redo(),
        EditorAction::DeleteSelected => selection.0.is_some(),
        _ => true,
    }
}

fn style_buttons(
    tools: Res<EditorTools>,
    doc: Res<EditorDoc>,
    ui: Res<EditorUi>,
    selection: Res<EditorSelection>,
    mut buttons: Query<(&UiAction, Option<&Hovered>, Has<Pressed>, &mut BackgroundColor)>,
) {
    for (action, hovered, pressed, mut background) in &mut buttons {
        let hovered = hovered.is_some_and(Hovered::get);
        let color = if !is_enabled(&action.0, &doc, &selection) {
            BUTTON_DISABLED
        } else if pressed {
            BUTTON_PRESSED
        } else {
            match (is_lit(&action.0, &tools, &doc, &ui), hovered) {
                (true, false) => BUTTON_LIT,
                (true, true) => BUTTON_LIT_HOVER,
                (false, true) => BUTTON_HOVER,
                (false, false) => BUTTON,
            }
        };
        background.set_if_neq(BackgroundColor(color));
    }
}

fn show_by_mode(mode: Res<ClientMode>, mut roots: Query<(&ShownIn, &mut Visibility)>) {
    for (shown_in, mut visibility) in &mut roots {
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
