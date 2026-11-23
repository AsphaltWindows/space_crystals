//! The UI lists that follow the document and the files: tile presets (the map's own table, custom presets
//! included), owners, player slots, the files in `maps/`, and the validation problems. Each is rebuilt when what it
//! shows changes.

use std::path::PathBuf;

use bevy::prelude::*;
use sc_sim::{Faction, PlayerId, Tile, TilePos, TilePresetId};

use super::fields::{TextField, text_field};
use super::{DIM_TEXT, ERROR_TEXT, GAP_VH, OK_TEXT, SMALL_VH, TEXT, TEXT_VH, button, button_frame, label, row};
use crate::editor::actions::EditorAction;
use crate::editor::doc::EditorDoc;
use crate::editor::files;
use crate::editor::objects::faction_name;
use crate::editor::state::{EditorFiles, EditorProblems};
use crate::map_render::tile_color;

#[derive(Component)]
pub struct PresetList;

#[derive(Component)]
pub struct OwnerList;

#[derive(Component)]
pub struct SlotList;

#[derive(Component)]
pub struct FileList;

#[derive(Component)]
pub struct ProblemList;

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

pub fn rebuild_presets(
    mut commands: Commands,
    doc: Res<EditorDoc>,
    list: Query<Entity, With<PresetList>>,
    mut shown: Local<Option<Vec<String>>>,
) {
    let names = doc.scenario().map.presets().iter().map(|p| p.name.clone()).collect();
    rebuild(&mut commands, &list, &mut shown, names, |list, names| {
        for (i, name) in names.iter().enumerate() {
            let id = TilePresetId(i as u16);
            list.spawn(button_frame(EditorAction::SetPreset(id)))
                .with_children(|b| {
                    b.spawn((
                        Node {
                            width: vh(TEXT_VH * 1.2),
                            height: vh(TEXT_VH * 1.2),
                            ..default()
                        },
                        BackgroundColor(tile_color(Tile::new(id, 0)).into()),
                    ));
                    b.spawn(label(name.clone(), TEXT_VH, TEXT));
                });
        }
    });
}

pub fn rebuild_owners(
    mut commands: Commands,
    doc: Res<EditorDoc>,
    list: Query<Entity, With<OwnerList>>,
    mut shown: Local<Option<Vec<PlayerId>>>,
) {
    let slots = doc.scenario().players.iter().map(|p| p.id).collect();
    rebuild(&mut commands, &list, &mut shown, slots, |list, slots| {
        list.spawn(button("Neutral", EditorAction::SetOwner(None)));
        for slot in slots {
            list.spawn(button(format!("P{}", slot.0), EditorAction::SetOwner(Some(*slot))));
        }
    });
}

/// What a slot's row shows.
#[derive(Clone, PartialEq)]
pub struct SlotRow {
    id: PlayerId,
    name: String,
    faction: Faction,
    start: Option<TilePos>,
    camera: Option<Vec2>,
}

pub fn rebuild_slots(
    mut commands: Commands,
    doc: Res<EditorDoc>,
    list: Query<Entity, With<SlotList>>,
    mut shown: Local<Option<Vec<SlotRow>>>,
) {
    let rows = doc
        .scenario()
        .players
        .iter()
        .map(|p| SlotRow {
            id: p.id,
            name: p.name.clone(),
            faction: p.faction,
            start: p.start_location,
            camera: p.camera_start,
        })
        .collect();
    rebuild(&mut commands, &list, &mut shown, rows, |list, rows| {
        for slot in rows {
            let next = Faction::ALL
                [(Faction::ALL.iter().position(|f| *f == slot.faction).unwrap_or(0) + 1) % Faction::ALL.len()];
            list.spawn(row()).with_children(|r| {
                r.spawn(button(format!("P{}", slot.id.0), EditorAction::SelectSlot(slot.id)));
                r.spawn(text_field(TextField::SlotName(slot.id), &slot.name, 10.0));
                r.spawn(button(
                    faction_name(slot.faction),
                    EditorAction::SetSlotFaction(slot.id, next),
                ));
                r.spawn(button("X", EditorAction::RemoveSlot(slot.id)));
            });
            let start = slot.start.map_or("none".into(), |t| format!("({}, {})", t.x, t.y));
            let camera = slot.camera.map_or("none".into(), |c| {
                let t = TilePos::containing(c);
                format!("({}, {})", t.x, t.y)
            });
            list.spawn((
                label(format!("start {start} | camera {camera}"), SMALL_VH, DIM_TEXT),
                Node {
                    margin: UiRect::bottom(vh(GAP_VH)),
                    ..default()
                },
            ));
        }
    });
}

pub fn rebuild_files(
    mut commands: Commands,
    files: Res<EditorFiles>,
    list: Query<Entity, With<FileList>>,
    mut shown: Local<Option<Vec<PathBuf>>>,
) {
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

pub fn rebuild_problems(
    mut commands: Commands,
    problems: Res<EditorProblems>,
    list: Query<Entity, With<ProblemList>>,
    mut shown: Local<Option<Vec<String>>>,
) {
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
