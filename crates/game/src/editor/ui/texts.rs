//! The UI's texts that follow the editor's state: the status line, the height level, the file, the selection,
//! the problem count, and the F1 help.

use bevy::prelude::*;
use sc_sim::{SpawnAt, TilePos, TilePresetId};

use super::{ERROR_TEXT, OK_TEXT, TEXT, WARN_TEXT};
use crate::ClientMode;
use crate::editor::doc::EditorDoc;
use crate::editor::files;
use crate::editor::state::{
    EditorCursor, EditorProblems, EditorSelection, EditorStatus, EditorUi, Selection, StatusLevel,
};
use crate::editor::tools::{EditorTools, ElevationMode, Tool, symmetry_label};

/// The two halves of the status line.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub enum StatusText {
    /// Cursor tile, tool, brush, symmetry and file.
    Info,
    /// The last action's message.
    Message,
}

/// A panel text that follows the editor's state.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub enum PanelText {
    ElevationLevel,
    File,
    Selection,
    ProblemsHeader,
}

#[derive(Component)]
pub struct HelpOverlay;

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
  M    symmetry        G  grid      K  slopes      Home  whole map

The Height tool sets the corners of the brush's tiles; the slopes around
them follow, and stop at cliffs. Plane and Rugged need ground at most
1 level steep; crystals and stations need flat ground.

EDIT
  Delete  delete the selection        Esc  cancel / deselect
  Ctrl+Z  undo    Ctrl+Y  redo    Ctrl+S  save    Ctrl+N  new map
  F5      play-test (Esc or F5 returns)       F1  this help

Text fields apply on Enter or when you click elsewhere; Esc reverts them.
Opening a file or a new map with unsaved changes asks twice."
        .into()
}

/// Changes the text only when it differs, so unchanged text isn't marked changed and laid out again.
fn set_text(text: &mut Mut<Text>, value: String) {
    if text.0 != value {
        text.0 = value;
    }
}

/// The status line's description of the cursor, the tool and the file: the tile under the cursor with the height
/// there (interpolated, `core/map.md > Elevation > Terrain height at a point`), its level, and whether the tile is
/// flat or sloped.
pub fn status_info(doc: &EditorDoc, tools: &EditorTools, cursor: &EditorCursor) -> String {
    let map = &doc.scenario().map;
    // Never indexes: the tools may hold a preset of the map before (review finding).
    let preset_name = |id: TilePresetId| map.presets().get(id.0 as usize).map_or("?", |p| p.name.as_str());
    let under = cursor.0.and_then(|p| {
        let t = TilePos::containing(p);
        Some((p, t, map.tile(t)?))
    });
    let at = match under {
        Some((p, t, tile)) => {
            let shape = if map.is_too_steep(t) {
                "too steep"
            } else if map.is_flat(t) {
                "flat"
            } else {
                "sloped"
            };
            format!(
                "Tile ({}, {}) {}, height {:.1} (level {}), {shape}",
                t.x,
                t.y,
                preset_name(tile.preset),
                map.height_at(p),
                map.elevation_level(p)
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
            let owner = tools.owner.map_or("neutral".into(), |p| format!("P{}", p.0));
            format!("Place {} ({owner})", tools.kind.def().name)
        }
        Tool::StartLocation => format!("Start location of P{}", tools.slot.0),
        Tool::CameraStart => format!("Camera start of P{}", tools.slot.0),
        Tool::Select => "Select".into(),
    };
    // The file name only: the properties panel shows where it is.
    let file = match doc.path().and_then(|p| p.file_name()) {
        Some(name) => name.to_string_lossy().into_owned(),
        None => "new map".into(),
    };
    let dirty = if doc.is_dirty() { " (unsaved)" } else { "" };
    format!(
        "{at}  |  {tool}  |  symmetry {}  |  {file}{dirty}",
        symmetry_label(tools.symmetry)
    )
}

pub fn update_status_line(
    doc: Res<EditorDoc>,
    tools: Res<EditorTools>,
    cursor: Res<EditorCursor>,
    status: Res<EditorStatus>,
    mut texts: Query<(&StatusText, &mut Text, &mut TextColor)>,
) {
    for (which, mut text, mut color) in &mut texts {
        match which {
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
    let scenario = doc.scenario();
    match selection.0 {
        Some(Selection::Spawn(i)) => match scenario.spawns.get(i) {
            Some(spawn) => {
                let owner = spawn.owner.map_or("neutral".into(), |p| format!("player {}", p.0));
                let at = match spawn.at {
                    SpawnAt::Tile(t) => format!("tile {}, {}", t.x, t.y),
                    SpawnAt::Point(p) => format!("point {:.0}, {:.0}", p.x, p.y),
                };
                format!("#{i} {} at {at}, {owner}", spawn.kind.def().name)
            }
            None => String::new(),
        },
        Some(Selection::StartLocation(p)) => format!("Start location of player {}", p.0),
        Some(Selection::CameraStart(p)) => format!("Camera start of player {}", p.0),
        None => "Nothing. The Select tool (V) picks objects and markers; drag to move them.".into(),
    }
}

pub fn update_panel_texts(
    doc: Res<EditorDoc>,
    tools: Res<EditorTools>,
    selection: Res<EditorSelection>,
    problems: Res<EditorProblems>,
    mut texts: Query<(&PanelText, &mut Text, &mut TextColor)>,
) {
    for (which, mut text, mut color) in &mut texts {
        match which {
            PanelText::ElevationLevel => set_text(&mut text, format!("Set level {}", tools.elevation)),
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
        }
    }
}

pub fn show_help(mode: Res<ClientMode>, ui: Res<EditorUi>, mut help: Query<&mut Visibility, With<HelpOverlay>>) {
    let shown = ui.help && *mode == ClientMode::Editor;
    for mut visibility in &mut help {
        visibility.set_if_neq(if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}
