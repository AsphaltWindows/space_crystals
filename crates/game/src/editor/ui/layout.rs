//! The editor UI's fixed structure, spawned once. Lists and texts that follow the document are filled in by
//! [`super::lists`] and [`super::texts`], and the text fields by [`super::fields::refresh_fields`].

use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui_widgets::ScrollArea;

use super::dialog::spawn_new_map_dialog;
use super::fields::{TextField, text_field};
use super::lists::{
    FileList, OwnerList, PlayTestList, PresetEditor, PresetList, ProblemList, SlotList, SpawnFieldList, list_node,
};
use super::texts::{HelpOverlay, PanelText, StatusText, help_text};
use super::{
    BORDER, DIM_TEXT, GAP_VH, LEFT_PANEL_VH, PANEL_BACKGROUND, RIGHT_PANEL_VH, SMALL_VH, STATUS_VH, ShownIn, TEXT,
    TEXT_VH, TOOLBAR_VH, WARN_TEXT, button, header, label, row,
};
use crate::ClientMode;
use crate::editor::actions::EditorAction;
use crate::editor::objects::palette_groups;
use crate::editor::tools::{BRUSH_SIZES, ElevationMode, Tool};
use crate::hud::TOP_BAR_HEIGHT_VH;

pub fn spawn_editor_ui(mut commands: Commands) {
    spawn_toolbar(&mut commands);
    spawn_palette(&mut commands);
    spawn_properties(&mut commands);
    spawn_status_line(&mut commands);
    spawn_overlays(&mut commands);
    spawn_new_map_dialog(&mut commands);
}

fn separator() -> impl Bundle {
    (
        Node {
            width: px(1),
            height: vh(TOOLBAR_VH * 0.6),
            margin: UiRect::horizontal(vh(0.6)),
            ..default()
        },
        BackgroundColor(BORDER),
    )
}

fn spawn_toolbar(commands: &mut Commands) {
    commands
        .spawn((
            Name::new("Editor toolbar"),
            ShownIn(ClientMode::Editor),
            Node {
                position_type: PositionType::Absolute,
                top: px(0),
                left: px(0),
                width: percent(100),
                height: vh(TOOLBAR_VH),
                padding: UiRect::horizontal(vh(1.0)),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: vh(GAP_VH),
                border: UiRect::bottom(px(1)),
                ..default()
            },
            BackgroundColor(PANEL_BACKGROUND),
            BorderColor::all(BORDER),
        ))
        .with_children(|bar| {
            for (text, action) in [
                ("New", EditorAction::NewMap),
                ("Save", EditorAction::Save),
                ("Undo", EditorAction::Undo),
                ("Redo", EditorAction::Redo),
            ] {
                bar.spawn(button(text, action));
            }
            bar.spawn(separator());
            for tool in Tool::ALL {
                bar.spawn(button(tool.label(), EditorAction::SetTool(tool)));
            }
            bar.spawn(Node {
                flex_grow: 1.0,
                ..default()
            });
            bar.spawn(button("Grid (G)", EditorAction::ToggleGrid));
            bar.spawn(button("Slopes (K)", EditorAction::ToggleSlopes));
            bar.spawn(button("Whole map", EditorAction::FitMap));
            bar.spawn(button("Play-test (F5)", EditorAction::PlayTest));
            bar.spawn(button("Help (F1)", EditorAction::ToggleHelp));
        });
}

/// A panel down one side, between the toolbar and the status line, scrolled with the wheel.
fn side_panel(name: &'static str, left: bool, width_vh: f32) -> impl Bundle {
    let (left, right, border) = if left {
        (px(0), auto(), UiRect::right(px(1)))
    } else {
        (auto(), px(0), UiRect::left(px(1)))
    };
    (
        Name::new(name),
        ShownIn(ClientMode::Editor),
        ScrollArea,
        Node {
            position_type: PositionType::Absolute,
            top: vh(TOOLBAR_VH),
            bottom: vh(STATUS_VH),
            left,
            right,
            width: vh(width_vh),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(vh(1.0)),
            row_gap: vh(GAP_VH),
            overflow: Overflow {
                x: OverflowAxis::Clip,
                y: OverflowAxis::Scroll,
            },
            border,
            ..default()
        },
        BackgroundColor(PANEL_BACKGROUND),
        BorderColor::all(BORDER),
    )
}

fn note(text: &str) -> impl Bundle {
    label(text.to_string(), SMALL_VH, DIM_TEXT)
}

fn spawn_palette(commands: &mut Commands) {
    commands
        .spawn(side_panel("Editor palette", true, LEFT_PANEL_VH))
        .with_children(|panel| {
            panel.spawn(header("Terrain"));
            panel.spawn((PresetList, list_node()));
            panel.spawn(row()).with_children(|r| {
                r.spawn(label("Brush", SMALL_VH, DIM_TEXT));
                for size in BRUSH_SIZES {
                    r.spawn(button(size.to_string(), EditorAction::SetBrush(size)));
                }
            });
            panel.spawn(label("Chosen preset", SMALL_VH, DIM_TEXT));
            panel.spawn((PresetEditor, list_node()));

            panel.spawn(header("Height"));
            panel.spawn(row()).with_children(|r| {
                for mode in ElevationMode::ALL {
                    r.spawn(button(mode.label(), EditorAction::SetElevationMode(mode)));
                }
            });
            panel.spawn(row()).with_children(|r| {
                r.spawn(button("-", EditorAction::StepElevation(-1)));
                r.spawn((PanelText::ElevationLevel, label("", TEXT_VH, TEXT)));
                r.spawn(button("+", EditorAction::StepElevation(1)));
            });
            panel.spawn(note("Sets the brush's tile corners; slopes follow and stop at cliffs."));

            panel.spawn(header("Objects"));
            for group in palette_groups() {
                panel.spawn(label(group.title, SMALL_VH, DIM_TEXT));
                panel.spawn(row()).with_children(|r| {
                    for kind in group.kinds {
                        r.spawn(button(kind.def().name, EditorAction::SetKind(kind)));
                    }
                });
            }
            panel.spawn(note("Owner (resources are always neutral)"));
            panel.spawn((OwnerList, row()));

            panel.spawn(header("Player slots"));
            panel.spawn(note(
                "Each slot is a start position. On a symmetric map slots come in sets: you edit the first, its \
                 images follow.",
            ));
            panel.spawn((SlotList, list_node()));
            panel.spawn(row()).with_children(|r| {
                r.spawn(button("Add slots", EditorAction::AddSlots));
                r.spawn(button("Place start", EditorAction::SetTool(Tool::Start)));
                r.spawn(button("Place camera", EditorAction::SetTool(Tool::CameraStart)));
            });
            panel.spawn(note(
                "Markers belong to the lit slot. Select a camera start and press Delete to clear it.",
            ));
        });
}

fn field_row(name: &str) -> impl Bundle {
    (
        row(),
        children![(
            label(name.to_string(), SMALL_VH, DIM_TEXT),
            Node {
                width: vh(6.0),
                ..default()
            }
        )],
    )
}

fn spawn_properties(commands: &mut Commands) {
    commands
        .spawn(side_panel("Editor properties", false, RIGHT_PANEL_VH))
        .with_children(|panel| {
            panel.spawn(header("Map"));
            panel.spawn(field_row("Name")).with_children(|r| {
                r.spawn(text_field(TextField::MapName, "", 20.0));
            });
            panel.spawn(note("Description"));
            panel.spawn(text_field(TextField::Description, "", 27.0));
            panel.spawn(field_row("Type")).with_children(|r| {
                r.spawn((PanelText::MapType, label("", SMALL_VH, TEXT)));
            });
            panel.spawn(field_row("Symmetry")).with_children(|r| {
                r.spawn((PanelText::Symmetry, label("", SMALL_VH, TEXT)));
            });
            panel.spawn(field_row("Size")).with_children(|r| {
                r.spawn(text_field(TextField::Width, "", 6.0));
                r.spawn(label("x", TEXT_VH, DIM_TEXT));
                r.spawn(text_field(TextField::Height, "", 6.0));
            });
            panel.spawn(note("Resizing keeps the top-left corner; Enter applies a field."));
            panel.spawn((PanelText::File, label("", SMALL_VH, DIM_TEXT)));

            panel.spawn(header("Selection"));
            panel.spawn((PanelText::Selection, label("", SMALL_VH, TEXT)));
            panel.spawn((SpawnFieldList, list_node()));
            panel.spawn(row()).with_children(|r| {
                r.spawn(button("Delete", EditorAction::DeleteSelected));
            });

            panel.spawn(header("Play-test"));
            panel.spawn(note(
                "Who plays which slot (not saved with the map): click a faction to change it.",
            ));
            panel.spawn((PlayTestList, list_node()));
            panel.spawn(field_row("Seed")).with_children(|r| {
                r.spawn(text_field(TextField::Seed, "", 20.0));
            });
            panel.spawn((PanelText::PlayTestOptions, label("", SMALL_VH, DIM_TEXT)));

            panel.spawn((PanelText::ProblemsHeader, header("Problems")));
            panel.spawn((ProblemList, list_node()));

            panel.spawn(header("Maps"));
            panel.spawn((FileList, list_node()));
            panel.spawn(row()).with_children(|r| {
                r.spawn(button("New map", EditorAction::NewMap));
                r.spawn(button("Refresh", EditorAction::RefreshFiles));
            });
        });
}

fn spawn_status_line(commands: &mut Commands) {
    commands.spawn((
        Name::new("Editor status line"),
        ShownIn(ClientMode::Editor),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(0),
            left: px(0),
            width: percent(100),
            height: vh(STATUS_VH),
            padding: UiRect::horizontal(vh(1.0)),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            column_gap: vh(2.0),
            overflow: Overflow::clip(),
            border: UiRect::top(px(1)),
            ..default()
        },
        BackgroundColor(PANEL_BACKGROUND),
        BorderColor::all(BORDER),
        children![
            (StatusText::Info, label("", SMALL_VH, TEXT), TextLayout::no_wrap()),
            (StatusText::Message, label("", SMALL_VH, TEXT), TextLayout::no_wrap()),
        ],
    ));
}

/// The F1 help and the play-test banner.
fn spawn_overlays(commands: &mut Commands) {
    commands.spawn((
        Name::new("Editor help"),
        HelpOverlay,
        Node {
            position_type: PositionType::Absolute,
            top: px(0),
            left: px(0),
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        GlobalZIndex(10),
        Visibility::Hidden,
        children![(
            Node {
                padding: UiRect::all(vh(2.0)),
                border: UiRect::all(px(1)),
                ..default()
            },
            BackgroundColor(PANEL_BACKGROUND),
            BorderColor::all(BORDER),
            children![label(help_text(), TEXT_VH, TEXT)],
        )],
    ));
    commands.spawn((
        Name::new("Play-test banner"),
        ShownIn(ClientMode::PlayTest),
        Node {
            position_type: PositionType::Absolute,
            top: px(0),
            left: px(0),
            width: percent(100),
            height: vh(TOP_BAR_HEIGHT_VH),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        Pickable::IGNORE,
        GlobalZIndex(5),
        Visibility::Hidden,
        children![label("PLAY-TEST: F5 returns to the editor", 2.0, WARN_TEXT)],
    ));
}
