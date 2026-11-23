//! The UI lists that follow the document and the files: tile presets (the map's own table, custom presets
//! included) and the chosen preset's editor, owners, player slots, the play-test setup, the files in `maps/`, the
//! validation problems, and the selected spawn's per-object values. Each is rebuilt when what it shows changes,
//! and only then: a list with a text field is keyed so that the field survives edits elsewhere (review finding:
//! moving a marker rebuilt the slots' name fields).

use std::path::PathBuf;

use bevy::prelude::*;
use sc_sim::{Faction, ObjectKind, SlotId, TileLook, TilePos, TilePresetId};

use super::fields::{SpawnValue, TextField, text_field};
use super::{
    DIM_TEXT, ERROR_TEXT, GAP_VH, OK_TEXT, SMALL_VH, TEXT, TEXT_VH, button, button_frame, column, label, row,
    small_button,
};
use crate::editor::actions::EditorAction;
use crate::editor::doc::EditorDoc;
use crate::editor::files;
use crate::editor::objects::faction_name;
use crate::editor::overlay::slot_color;
use crate::editor::playtest::PlayTestSetup;
use crate::editor::presets::PresetFlag;
use crate::editor::state::{EditorFiles, EditorProblems, EditorSelection, Selection};
use crate::editor::tools::EditorTools;
use crate::map_render::tile_color;

#[derive(Component)]
pub struct PresetList;

/// The chosen preset's properties: editable for a custom preset.
#[derive(Component)]
pub struct PresetEditor;

#[derive(Component)]
pub struct OwnerList;

#[derive(Component)]
pub struct SlotList;

/// The play-test setup: each slot's faction and the slot you play.
#[derive(Component)]
pub struct PlayTestList;

#[derive(Component)]
pub struct FileList;

#[derive(Component)]
pub struct ProblemList;

/// The selected spawn's per-object values (`core/map_editor.md > Per-object Resource Values`).
#[derive(Component)]
pub struct SpawnFieldList;

/// Replaces the children of the single `L` list when `wanted` differs from what it shows.
fn rebuild<L: Component, T: PartialEq>(
    commands: &mut Commands,
    list: &Query<Entity, With<L>>,
    shown: &mut Option<T>,
    wanted: T,
    fill: impl FnOnce(&mut ChildSpawnerCommands, &T),
) {
    if shown.as_ref() == Some(&wanted) {
        return;
    }
    let Ok(entity) = list.single() else {
        return;
    };
    let mut list = commands.entity(entity);
    list.despawn_children();
    list.with_children(|children| fill(children, &wanted));
    *shown = Some(wanted);
}

/// A small square of colour.
fn swatch(color: Color) -> impl Bundle {
    (
        Node {
            width: vh(TEXT_VH * 1.2),
            height: vh(TEXT_VH * 1.2),
            ..default()
        },
        BackgroundColor(color),
    )
}

pub fn rebuild_presets(
    mut commands: Commands,
    doc: Res<EditorDoc>,
    list: Query<Entity, With<PresetList>>,
    mut shown: Local<Option<Vec<(String, TileLook)>>>,
) {
    if shown.is_some() && !doc.is_changed() {
        return;
    }
    let presets = doc.map().terrain.presets();
    let wanted = presets.iter().map(|p| (p.name.clone(), p.look)).collect();
    rebuild(&mut commands, &list, &mut shown, wanted, |list, presets| {
        for (i, (name, look)) in presets.iter().enumerate() {
            let id = TilePresetId(i as u16);
            list.spawn(button_frame(EditorAction::SetPreset(id)))
                .with_children(|b| {
                    b.spawn(swatch(tile_color(*look, 0.0).into()));
                    b.spawn(label(name.clone(), TEXT_VH, TEXT));
                    if !id.is_default() {
                        b.spawn(label("custom", SMALL_VH, DIM_TEXT));
                    }
                });
        }
        list.spawn(button("Add custom preset", EditorAction::AddPreset));
    });
}

/// The chosen preset's name, properties and look. A custom preset's can be changed, and it can be removed; a
/// default preset's are listed in one line, fixed. Keyed by the preset only: its name field stays while it is edited.
pub fn rebuild_preset_editor(
    mut commands: Commands,
    doc: Res<EditorDoc>,
    tools: Res<EditorTools>,
    list: Query<Entity, With<PresetEditor>>,
    mut shown: Local<Option<(TilePresetId, Option<String>)>>,
) {
    if shown.is_some() && !doc.is_changed() && !tools.is_changed() {
        return;
    }
    let id = tools.preset;
    // A default preset is fixed: one line says what it is.
    let fixed = id
        .is_default()
        .then(|| doc.map().terrain.presets().get(id.0 as usize).map(default_preset_line))
        .flatten();
    rebuild(&mut commands, &list, &mut shown, (id, fixed), |list, (id, fixed)| {
        let id = *id;
        if let Some(line) = fixed {
            list.spawn(label(line.clone(), SMALL_VH, DIM_TEXT));
            return;
        }
        list.spawn(row()).with_children(|r| {
            r.spawn(label("Name", SMALL_VH, DIM_TEXT));
            r.spawn(text_field(TextField::PresetName(id), "", 14.0));
            r.spawn(small_button("Remove", EditorAction::RemovePreset(id)));
        });
        list.spawn(row()).with_children(|r| {
            for flag in PresetFlag::ALL {
                r.spawn(small_button(flag.label(), EditorAction::TogglePresetFlag(id, flag)));
            }
        });
        list.spawn(row()).with_children(|r| {
            r.spawn(label("Look", SMALL_VH, DIM_TEXT));
            for look in TileLook::ALL {
                r.spawn(button_frame(EditorAction::SetPresetLook(id, look)))
                    .with_children(|b| {
                        b.spawn(swatch(tile_color(look, 0.0).into()));
                        b.spawn(label(look.label(), SMALL_VH, TEXT));
                    });
            }
        });
    });
}

/// What a default preset is, in one line: its name and its properties.
fn default_preset_line(preset: &sc_sim::TilePreset) -> String {
    let properties: Vec<String> = PresetFlag::ALL
        .into_iter()
        .filter(|flag| flag.get(preset))
        .map(|flag| flag.label().to_lowercase())
        .collect();
    let properties = if properties.is_empty() {
        "no properties".into()
    } else {
        properties.join(", ")
    };
    format!("{}: a default preset, fixed; {properties}", preset.name)
}

pub fn rebuild_owners(
    mut commands: Commands,
    doc: Res<EditorDoc>,
    list: Query<Entity, With<OwnerList>>,
    mut shown: Local<Option<(Vec<SlotId>, bool)>>,
) {
    if shown.is_some() && !doc.is_changed() {
        return;
    }
    let map = doc.map();
    let skirmish = map.map_type == sc_sim::MapType::SkirmishDefault;
    let wanted = (map.slot_ids().collect(), skirmish);
    rebuild(&mut commands, &list, &mut shown, wanted, |list, (slots, skirmish)| {
        list.spawn(button("Neutral", EditorAction::SetOwner(None)));
        if *skirmish {
            list.spawn(label(
                "Skirmish Default: owned objects go in the Starting Layouts",
                SMALL_VH,
                DIM_TEXT,
            ));
            return;
        }
        for slot in slots {
            list.spawn(button(format!("S{}", slot.0), EditorAction::SetOwner(Some(*slot))));
        }
    });
}

/// What a slot's row shows.
#[derive(Clone, PartialEq)]
pub struct SlotRow {
    id: SlotId,
    /// The editable slot it is an image of, if it is generated.
    source: Option<SlotId>,
    start: TilePos,
    camera: Option<Vec2>,
}

/// The slots, each set's editable slot first: it can be chosen for the marker tools and its set removed; the
/// generated slots are listed dimmed, as images.
pub fn rebuild_slots(
    mut commands: Commands,
    doc: Res<EditorDoc>,
    list: Query<Entity, With<SlotList>>,
    mut shown: Local<Option<(Vec<SlotRow>, bool)>>,
) {
    if shown.is_some() && !doc.is_changed() {
        return;
    }
    let map = doc.map();
    let rows = map
        .slot_ids()
        .zip(&map.slots)
        .map(|(id, slot)| SlotRow {
            id,
            source: (!map.is_slot_editable(id)).then(|| map.source_slot(id)),
            start: slot.start,
            camera: slot.camera_start,
        })
        .collect();
    let removable = map.slots.len() > map.image_count();
    rebuild(
        &mut commands,
        &list,
        &mut shown,
        (rows, removable),
        |list, (rows, removable)| {
            for slot in rows {
                let start = format!("start {}, {}", slot.start.x, slot.start.y);
                let camera = slot.camera.map_or("camera -".into(), |c| {
                    let t = TilePos::containing(c);
                    format!("camera {}, {}", t.x, t.y)
                });
                list.spawn(row()).with_children(|r| {
                    r.spawn(swatch(slot_color(slot.id)));
                    match slot.source {
                        None => {
                            r.spawn(button(format!("S{}", slot.id.0), EditorAction::SelectSlot(slot.id)));
                            r.spawn(label(format!("{start} | {camera}"), SMALL_VH, TEXT));
                            if *removable {
                                r.spawn(small_button("X", EditorAction::RemoveSlots(slot.id)));
                            }
                        }
                        Some(source) => {
                            r.spawn(label(format!("S{}", slot.id.0), TEXT_VH, DIM_TEXT));
                            r.spawn(label(
                                format!("image of S{}: {start} | {camera}", source.0),
                                SMALL_VH,
                                DIM_TEXT,
                            ));
                        }
                    }
                });
            }
        },
    );
}

/// One row per slot: its player's faction (click for the next one) and whether you play it.
pub fn rebuild_play_test(
    mut commands: Commands,
    doc: Res<EditorDoc>,
    setup: Res<PlayTestSetup>,
    list: Query<Entity, With<PlayTestList>>,
    mut shown: Local<Option<Vec<(SlotId, Faction)>>>,
) {
    if shown.is_some() && !doc.is_changed() && !setup.is_changed() {
        return;
    }
    let wanted = doc.map().slot_ids().map(|s| (s, setup.faction(s))).collect();
    rebuild(&mut commands, &list, &mut shown, wanted, |list, rows| {
        for &(slot, faction) in rows {
            let next =
                Faction::ALL[(Faction::ALL.iter().position(|f| *f == faction).unwrap_or(0) + 1) % Faction::ALL.len()];
            list.spawn(row()).with_children(|r| {
                r.spawn(swatch(slot_color(slot)));
                r.spawn((
                    label(format!("S{}", slot.0), TEXT_VH, TEXT),
                    Node {
                        width: vh(3.0),
                        ..default()
                    },
                ));
                // A fixed width, so that the column of faction buttons lines up (replaces the frame's `Node`).
                r.spawn((
                    button_frame(EditorAction::SetPlayTestFaction(slot, next)),
                    children![label(faction_name(faction), TEXT_VH, TEXT)],
                ))
                .insert(Node {
                    width: vh(11.0),
                    padding: UiRect::axes(vh(0.7), vh(0.35)),
                    border: UiRect::all(px(1)),
                    ..default()
                });
                r.spawn(small_button("you", EditorAction::SetPlayTestSlot(slot)));
            });
        }
    });
}

pub fn rebuild_files(
    mut commands: Commands,
    files: Res<EditorFiles>,
    list: Query<Entity, With<FileList>>,
    mut shown: Local<Option<Vec<PathBuf>>>,
) {
    if shown.is_some() && !files.is_changed() {
        return;
    }
    rebuild(&mut commands, &list, &mut shown, files.maps.clone(), |list, maps| {
        if maps.is_empty() {
            list.spawn(label("No maps yet", SMALL_VH, DIM_TEXT));
        }
        for path in maps {
            let name = path
                .file_name()
                .map_or_else(|| files::display_path(path), |n| n.to_string_lossy().into());
            list.spawn(button(name, EditorAction::Open(path.clone())));
        }
    });
}

/// One text field per value of the selected spawn's kind: a patch's amount, a station's supplies, delivery size
/// and interval (in seconds). The fields show the document's values and commit changes as
/// [`EditorAction::SetSpawnSettings`].
pub fn rebuild_spawn_fields(
    mut commands: Commands,
    doc: Res<EditorDoc>,
    selection: Res<EditorSelection>,
    list: Query<Entity, With<SpawnFieldList>>,
    mut shown: Local<Option<Option<(usize, ObjectKind)>>>,
) {
    if shown.is_some() && !doc.is_changed() && !selection.is_changed() {
        return;
    }
    let map = doc.map();
    let selected = match selection.0 {
        Some(Selection::Spawn(i)) => map.spawns.get(i).map(|s| (i, s.kind)),
        _ => None,
    };
    rebuild(&mut commands, &list, &mut shown, selected, |list, selected| {
        let Some((spawn, kind)) = *selected else {
            return;
        };
        for &value in SpawnValue::of(kind) {
            // The text arrives with the next refresh of the fields.
            let field = TextField::SpawnValue { spawn, value };
            list.spawn(row()).with_children(|r| {
                r.spawn((
                    label(value.label(), SMALL_VH, DIM_TEXT),
                    Node {
                        width: vh(11.0),
                        ..default()
                    },
                ));
                r.spawn(text_field(field, "", 9.0));
            });
        }
    });
}

pub fn rebuild_problems(
    mut commands: Commands,
    problems: Res<EditorProblems>,
    list: Query<Entity, With<ProblemList>>,
    mut shown: Local<Option<Vec<String>>>,
) {
    if shown.is_some() && !problems.is_changed() {
        return;
    }
    let texts = problems.0.iter().map(ToString::to_string).collect();
    rebuild(&mut commands, &list, &mut shown, texts, |list, texts| {
        if texts.is_empty() {
            list.spawn(label("None: the map is playable", SMALL_VH, OK_TEXT));
        }
        for (i, text) in texts.iter().enumerate() {
            list.spawn((
                button_frame(EditorAction::FocusProblem(i)),
                children![label(text.clone(), SMALL_VH, ERROR_TEXT)],
            ));
        }
    });
}

/// A list container with a little room below it.
pub fn list_node() -> Node {
    Node {
        margin: UiRect::bottom(vh(GAP_VH)),
        ..column()
    }
}
