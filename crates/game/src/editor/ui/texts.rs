//! The UI's texts that follow the editor's state: the status line, the height level, the map's fixed type and
//! symmetry, the file, the selection, the problem count, the play-test's game options, the creation dialog's
//! verdict, and the overlays (the F1 help and the creation dialog). Each is updated only when what it shows changed
//! (review finding: the status line compared whole maps every frame).

use bevy::prelude::*;
use sc_sim::{SpawnAt, TilePos, TilePresetId};

use super::{ERROR_TEXT, OK_TEXT, TEXT, WARN_TEXT};
use crate::ClientMode;
use crate::editor::creation::NewMapDialog;
use crate::editor::doc::EditorDoc;
use crate::editor::files;
use crate::editor::playtest::PlayTestSetup;
use crate::editor::state::{
    EditorCursor, EditorProblems, EditorSelection, EditorStatus, EditorUi, Selection, StatusLevel,
};
use crate::editor::tools::{EditorTools, ElevationMode, Tool, symmetry_label};

/// The two halves of the status line.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub enum StatusText {
    /// Cursor tile, tool and file.
    Info,
    /// The last action's message.
    Message,
}

/// A panel text that follows the editor's state.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub enum PanelText {
    ElevationLevel,
    MapType,
    Symmetry,
    File,
    Selection,
    ProblemsHeader,
    /// Whether play-tests accept debug commands.
    PlayTestOptions,
    /// Whether the creation dialog's settings make a map.
    NewMapVerdict,
}

#[derive(Component)]
pub struct HelpOverlay;

/// The creation dialog's backdrop.
#[derive(Component)]
pub struct NewMapOverlay;

/// The F1 help: mouse and keys.
pub fn help_text() -> String {
    "\
MOUSE
  Left           use the tool: drag to paint, move, or place more
  Right          pick the tile's preset and height
  Wheel          zoom at the cursor
  Middle drag    grab and pan; arrow keys and screen edge pan too

TOOLS
  V Select   B Paint   H Height   F Fill   O Place   L Start   C Camera
  [ ]  brush size      E  height mode         + -  height level
  G    grid            K  slopes              Home  whole map

The Height tool sets the corners of the brush's tiles; the slopes around
them follow, and stop at cliffs. Plane and Rugged need ground at most
1 level steep; crystals and stations need flat ground.

SYMMETRY
On a symmetric map you edit the bright part; the dimmed part is generated
from it after every edit: tiles, heights, objects, slots and their
markers. The map type and the symmetry are chosen when the map is created.

EDIT
  Delete  delete the selection        Esc  cancel / deselect
  Ctrl+Z  undo    Ctrl+Y  redo    Ctrl+S  save    Ctrl+N  new map
  F5      play-test (F5 returns)      F1  this help

Text fields apply on Enter or when you click elsewhere; Esc reverts them.
Opening a file, a new map or closing the window with unsaved changes
asks twice."
        .into()
}

/// Changes the text only when it differs, so unchanged text isn't marked changed and laid out again.
fn set_text(text: &mut Mut<Text>, value: String) {
    if text.0 != value {
        text.0 = value;
    }
}

/// The status line's description of the cursor, the tool and the file: the tile under the cursor with the height
/// there (interpolated, `core/map.md > Elevation > Terrain height at a point`), its level, whether the tile is flat
/// or sloped, and whether it is in the locked part.
pub fn status_info(doc: &EditorDoc, tools: &EditorTools, cursor: &EditorCursor) -> String {
    let map = doc.map();
    let terrain = &map.terrain;
    // Never indexes: the tools may hold a preset of the map before (review finding).
    let preset_name = |id: TilePresetId| terrain.presets().get(id.0 as usize).map_or("?", |p| p.name.as_str());
    let under = cursor.0.and_then(|p| {
        let t = TilePos::containing(p);
        Some((p, t, terrain.tile(t)?))
    });
    let at = match under {
        Some((p, t, tile)) => {
            let shape = if terrain.is_too_steep(t) {
                "too steep"
            } else if terrain.is_flat(t) {
                "flat"
            } else {
                "sloped"
            };
            let locked = if map.region().tile(t) { "" } else { ", locked" };
            format!(
                "Tile ({}, {}) {}, height {:.1} (level {}), {shape}{locked}",
                t.x,
                t.y,
                preset_name(tile.preset),
                terrain.height_at(p),
                terrain.elevation_level(p)
            )
        }
        None => "Off the map".into(),
    };
    let tool = match tools.tool {
        Tool::Paint => format!("Paint {}, brush {}", preset_name(tools.preset), tools.brush),
        Tool::Elevation => match tools.elevation_mode {
            ElevationMode::Set => format!("Height: set {}, brush {}", tools.elevation, tools.brush),
            mode => format!("Height: {}, brush {}", mode.label().to_lowercase(), tools.brush),
        },
        Tool::Fill => format!("Fill with {}", preset_name(tools.preset)),
        Tool::Place => {
            let owner = tools.owner.map_or("neutral".into(), |s| format!("S{}", s.0));
            format!("Place {} ({owner})", tools.kind.def().name)
        }
        Tool::Start => format!("Start of S{}", tools.slot.0),
        Tool::CameraStart => format!("Camera start of S{}", tools.slot.0),
        Tool::Select => "Select".into(),
    };
    // The file name only: the properties panel shows where it is.
    let file = match doc.path().and_then(|p| p.file_name()) {
        Some(name) => name.to_string_lossy().into_owned(),
        None => "new map".into(),
    };
    let dirty = if doc.is_dirty() { " (unsaved)" } else { "" };
    format!("{at}  |  {tool}  |  {file}{dirty}")
}

pub fn update_status_line(
    doc: Res<EditorDoc>,
    tools: Res<EditorTools>,
    cursor: Res<EditorCursor>,
    status: Res<EditorStatus>,
    mut texts: Query<(Ref<StatusText>, &mut Text, &mut TextColor)>,
) {
    let changed = doc.is_changed() || tools.is_changed() || cursor.is_changed() || status.is_changed();
    for (which, mut text, mut color) in &mut texts {
        if !changed && !which.is_added() {
            continue;
        }
        match *which {
            StatusText::Info => set_text(&mut text, status_info(&doc, &tools, &cursor)),
            StatusText::Message => {
                set_text(&mut text, status.message.clone());
                let wanted = match status.level {
                    StatusLevel::Info => TEXT,
                    StatusLevel::Warning => WARN_TEXT,
                    StatusLevel::Error => ERROR_TEXT,
                };
                color.set_if_neq(TextColor(wanted));
            }
        }
    }
}

/// What the selection panel says.
pub fn selection_text(doc: &EditorDoc, selection: &EditorSelection) -> String {
    let map = doc.map();
    match selection.0 {
        Some(Selection::Spawn(i)) => match map.spawns.get(i) {
            Some(spawn) => {
                let owner = spawn.owner.map_or("neutral".into(), |s| format!("slot {}", s.0));
                let at = match spawn.at {
                    SpawnAt::Tile(t) => format!("tile {}, {}", t.x, t.y),
                    SpawnAt::Point(p) => format!("point {:.0}, {:.0}", p.x, p.y),
                };
                format!("#{i} {} at {at}, {owner}", spawn.kind.def().name)
            }
            None => String::new(),
        },
        Some(Selection::Start(s)) => format!("Start of slot {}", s.0),
        Some(Selection::CameraStart(s)) => format!("Camera start of slot {}", s.0),
        None => "Nothing. The Select tool (V) picks objects and markers; drag to move them.".into(),
    }
}

pub fn update_panel_texts(
    doc: Res<EditorDoc>,
    tools: Res<EditorTools>,
    selection: Res<EditorSelection>,
    problems: Res<EditorProblems>,
    dialog: Res<NewMapDialog>,
    setup: Res<PlayTestSetup>,
    mut texts: Query<(Ref<PanelText>, &mut Text, &mut TextColor)>,
) {
    let changed = doc.is_changed()
        || tools.is_changed()
        || selection.is_changed()
        || problems.is_changed()
        || dialog.is_changed()
        || setup.is_changed();
    let map = doc.map();
    for (which, mut text, mut color) in &mut texts {
        if !changed && !which.is_added() {
            continue;
        }
        match *which {
            PanelText::ElevationLevel => set_text(&mut text, format!("Set level {}", tools.elevation)),
            PanelText::MapType => set_text(&mut text, format!("{} (fixed)", map.map_type.label())),
            PanelText::Symmetry => {
                let parts = match map.image_count() {
                    1 => String::new(),
                    n => format!(", {n} parts"),
                };
                set_text(&mut text, format!("{}{parts} (fixed)", symmetry_label(map.symmetry)));
            }
            PanelText::File => {
                let path = doc
                    .path()
                    .map_or("Not saved yet: Ctrl+S saves it in maps/".into(), |p| {
                        format!("File: {}", files::display_path(p))
                    });
                let dirty = if doc.is_dirty() { " (unsaved changes)" } else { "" };
                set_text(&mut text, format!("{path}{dirty}"));
            }
            PanelText::Selection => set_text(&mut text, selection_text(&doc, &selection)),
            PanelText::ProblemsHeader => {
                let n = problems.0.len();
                set_text(&mut text, format!("Problems ({n})"));
                color.set_if_neq(TextColor(if n == 0 { OK_TEXT } else { ERROR_TEXT }));
            }
            PanelText::PlayTestOptions => {
                let options = if setup.config.debug {
                    "Debug commands on (--no-debug turns them off)"
                } else {
                    "Debug commands off (--no-debug)"
                };
                set_text(&mut text, options.into());
            }
            PanelText::NewMapVerdict => {
                let (verdict, wanted) = match dialog.settings.check() {
                    Ok(()) => ("Ready: Create makes the map".to_string(), OK_TEXT),
                    Err(problem) => (format!("Can't create: {problem}"), WARN_TEXT),
                };
                set_text(&mut text, verdict);
                color.set_if_neq(TextColor(wanted));
            }
        }
    }
}

/// Shows the F1 help and the creation dialog, in the editor only.
pub fn show_overlays(
    mode: Res<ClientMode>,
    ui: Res<EditorUi>,
    dialog: Res<NewMapDialog>,
    mut help: Query<&mut Visibility, (With<HelpOverlay>, Without<NewMapOverlay>)>,
    mut new_map: Query<&mut Visibility, (With<NewMapOverlay>, Without<HelpOverlay>)>,
) {
    if !(mode.is_changed() || ui.is_changed() || dialog.is_changed()) {
        return;
    }
    let editing = *mode == ClientMode::Editor;
    let visibility = |shown: bool| {
        if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        }
    };
    for mut v in &mut help {
        v.set_if_neq(visibility(editing && ui.help));
    }
    for mut v in &mut new_map {
        v.set_if_neq(visibility(editing && dialog.open));
    }
}
