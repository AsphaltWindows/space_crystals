//! The creation dialog's layout (`core/map_editor.md > Map Creation`): a modal panel over the editor with the map
//! type, size, symmetry, slot count, name and description, the placeholder limits, and Create / Cancel. Its state
//! is [`crate::editor::creation::NewMapDialog`]; [`super::texts::show_overlays`] shows it while it is open.

use bevy::prelude::*;
use sc_sim::scenario::{MAX_SLOTS, MIN_MAP_SIDE};
use sc_sim::{MapType, Symmetry};

use super::fields::{TextField, text_field};
use super::texts::{NewMapOverlay, PanelText};
use super::{BORDER, DIM_TEXT, GAP_VH, PANEL_BACKGROUND, SMALL_VH, TEXT, WARN_TEXT, button, label, row, small_button};
use crate::editor::actions::EditorAction;
use crate::editor::creation::NewMapField;
use crate::editor::files::MAX_MAP_SIDE;
use crate::editor::tools::symmetry_label;

/// The dialog panel's width.
const DIALOG_VH: f32 = 60.0;

fn caption(text: &str) -> impl Bundle {
    (
        label(text.to_string(), SMALL_VH, DIM_TEXT),
        Node {
            width: vh(10.0),
            ..default()
        },
    )
}

pub fn spawn_new_map_dialog(commands: &mut Commands) {
    commands
        .spawn((
            Name::new("New map dialog"),
            NewMapOverlay,
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
            GlobalZIndex(8),
            Visibility::Hidden,
        ))
        .with_children(|backdrop| {
            backdrop
                .spawn((
                    Node {
                        width: vh(DIALOG_VH),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(vh(2.0)),
                        row_gap: vh(GAP_VH * 2.0),
                        border: UiRect::all(px(1)),
                        ..default()
                    },
                    BackgroundColor(PANEL_BACKGROUND),
                    BorderColor::all(BORDER),
                ))
                .with_children(|d| {
                    d.spawn(label("New map", 2.4, TEXT));
                    d.spawn(row()).with_children(|r| {
                        r.spawn(caption("Type"));
                        for map_type in MapType::ALL {
                            r.spawn(button(
                                map_type.label(),
                                EditorAction::EditNewMap(NewMapField::Type(map_type)),
                            ));
                        }
                    });
                    d.spawn(label(
                        "Skirmish Default: symmetric, any faction in any slot, owned objects only in the Starting \
                         Layouts. Custom: no restrictions.",
                        SMALL_VH,
                        DIM_TEXT,
                    ));
                    d.spawn(row()).with_children(|r| {
                        r.spawn(caption("Size"));
                        r.spawn(text_field(TextField::NewMapWidth, "", 7.0));
                        r.spawn(label("x", SMALL_VH, DIM_TEXT));
                        r.spawn(text_field(TextField::NewMapHeight, "", 7.0));
                        r.spawn(label("tiles", SMALL_VH, DIM_TEXT));
                    });
                    d.spawn(row()).with_children(|r| {
                        r.spawn(caption("Symmetry"));
                        for symmetry in Symmetry::ALL {
                            r.spawn(small_button(
                                symmetry_label(symmetry),
                                EditorAction::EditNewMap(NewMapField::Symmetry(symmetry)),
                            ));
                        }
                    });
                    d.spawn(row()).with_children(|r| {
                        r.spawn(caption("Player slots"));
                        for n in 1..=MAX_SLOTS {
                            r.spawn(small_button(
                                n.to_string(),
                                EditorAction::EditNewMap(NewMapField::Slots(n)),
                            ));
                        }
                    });
                    d.spawn(row()).with_children(|r| {
                        r.spawn(caption("Name"));
                        r.spawn(text_field(TextField::NewMapName, "", 30.0));
                    });
                    d.spawn(row()).with_children(|r| {
                        r.spawn(caption("Description"));
                        r.spawn(text_field(TextField::NewMapDescription, "", 44.0));
                    });
                    d.spawn(label(
                        format!(
                            "Placeholder limits: sides from {MIN_MAP_SIDE} to {MAX_MAP_SIDE} tiles; at most \
                             {MAX_SLOTS} slots, in whole sets of the symmetry's parts. 90 degree rotation makes the \
                             map square. The type and the symmetry are fixed once the map exists."
                        ),
                        SMALL_VH,
                        WARN_TEXT,
                    ));
                    d.spawn((PanelText::NewMapVerdict, label("", SMALL_VH, TEXT)));
                    d.spawn(row()).with_children(|r| {
                        r.spawn(button("Create", EditorAction::CreateMap));
                        r.spawn(button("Cancel (Esc)", EditorAction::CancelNewMap));
                    });
                });
        });
}
