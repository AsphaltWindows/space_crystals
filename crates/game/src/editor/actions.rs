//! The editor's action layer: everything the editor can do is an [`EditorAction`]. Raw input, UI buttons and
//! text fields, `--editor-script` files and tests all queue actions in [`EditorActions`], and one system applies
//! them in order. Tests and scripts never touch raw input.

use std::collections::VecDeque;
use std::path::PathBuf;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use sc_sim::{Faction, ObjectKind, PlayerId, TilePos, TilePresetId};
use serde::{Deserialize, Serialize};

use super::doc::EditorDoc;
use super::gesture::Gesture;
use super::state::{EditorCursor, EditorFiles, EditorProblems, EditorSelection, EditorStatus, EditorUi, Selection};
use super::tools::{BRUSH_SIZES, EditorTools, ElevationMode, Symmetry, TOP_ELEVATION, Tool};
use crate::ClientMode;
use crate::camera::{CameraView, CameraZoom, ViewportInsets};
use crate::platform::SurfaceSize;

/// Everything the editor can be asked to do. Positions are sim space units.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EditorAction {
    // --- The pointer on the map ---
    /// The cursor is over this map point, or off the map view: moves the brush outline and the ghost.
    Hover(Option<Vec2>),
    /// Primary button down: starts a brush stroke, a fill, a placement, a selection with a drag, or a marker drag.
    Press(Vec2),
    /// The pointer moved with the primary button held: continues the stroke or the drag.
    Drag(Vec2),
    /// Primary button up: the gesture becomes one undo step.
    Release,
    /// Secondary click: picks the tile's preset and elevation for the terrain tools.
    Pick(Vec2),
    /// Abandons the current gesture, else closes the help, else clears the selection.
    Cancel,

    // --- Tools ---
    SetTool(Tool),
    SetBrush(u32),
    /// Chooses the preset to paint (switching to Paint unless Fill is active).
    SetPreset(TilePresetId),
    /// Switches to the elevation tool in this mode.
    SetElevationMode(ElevationMode),
    /// Chooses the level that Set applies (switching to the elevation tool in Set mode).
    SetElevation(u8),
    /// Raises (positive) or lowers the level that Set applies, like `SetElevation`.
    StepElevation(i32),
    /// Chooses the object type to place (switching to Place).
    SetKind(ObjectKind),
    /// Neutral or a player slot, for placed objects.
    SetOwner(Option<PlayerId>),
    SetSymmetry(Symmetry),
    /// The slot whose markers the marker tools place and as whom play-tests run.
    SelectSlot(PlayerId),
    DeleteSelected,

    // --- The document ---
    Undo,
    Redo,
    /// Saves to the document's file, or to `maps/<name>.json` for a new map.
    Save,
    /// Opens a scenario file. With unsaved changes, it must be asked for twice.
    Open(PathBuf),
    /// Starts a new 64×48 map. With unsaved changes, it must be asked for twice.
    NewMap,
    SetName(String),
    SetSeed(u64),
    /// Resizes the map, anchored at the top-left; what falls outside is dropped and reported.
    Resize {
        width: u32,
        height: u32,
    },
    AddSlot,
    RemoveSlot(PlayerId),
    SetSlotName(PlayerId, String),
    SetSlotFaction(PlayerId, Faction),
    SetStartLocation(PlayerId, Option<TilePos>),
    SetCameraStart(PlayerId, Option<Vec2>),

    // --- The view ---
    /// Shows `factor` times as much map, keeping the point `at` under the cursor where it is.
    Zoom {
        factor: f32,
        at: Option<Vec2>,
    },
    /// Zooms out to show the whole map.
    FitMap,
    CenterOn(Vec2),
    /// Centres the view on the validation problem at this index of the Problems list and selects its spawn.
    FocusProblem(usize),
    ToggleHelp,
    RefreshFiles,
    /// Starts a play-test of the map, or ends the running one.
    PlayTest,
}

/// Actions waiting to be applied, oldest first.
#[derive(Resource, Default, Debug)]
pub struct EditorActions(pub VecDeque<EditorAction>);

impl EditorActions {
    pub fn push(&mut self, action: EditorAction) {
        self.0.push_back(action);
    }
}

/// Set when a play-test should start or stop; the play-test system does the switch.
#[derive(Resource, Default, Debug)]
pub struct PlayTestRequest(pub bool);

/// A destructive action waiting for its confirmation (asking for it again), because unsaved changes would be lost.
#[derive(Resource, Default, Debug)]
pub struct PendingConfirmation(pub Option<EditorAction>);

/// Everything the actions change.
#[derive(SystemParam)]
pub struct Editor<'w> {
    pub doc: ResMut<'w, EditorDoc>,
    pub tools: ResMut<'w, EditorTools>,
    pub selection: ResMut<'w, EditorSelection>,
    pub cursor: ResMut<'w, EditorCursor>,
    pub status: ResMut<'w, EditorStatus>,
    pub gesture: ResMut<'w, Gesture>,
    pub files: ResMut<'w, EditorFiles>,
    pub ui: ResMut<'w, EditorUi>,
    pub problems: Res<'w, EditorProblems>,
    pub confirm: ResMut<'w, PendingConfirmation>,
    pub view: ResMut<'w, CameraView>,
    pub zoom: ResMut<'w, CameraZoom>,
    pub size: Res<'w, SurfaceSize>,
    pub insets: Res<'w, ViewportInsets>,
}

/// Applies the queued actions in order. A play-test request stops the queue for this frame, so that the actions
/// after it apply once the switch is done. While a play-test runs, only the request to end it applies.
pub fn apply_editor_actions(
    mut queue: ResMut<EditorActions>,
    mode: Res<ClientMode>,
    mut play: ResMut<PlayTestRequest>,
    mut editor: Editor,
) {
    while let Some(action) = queue.0.pop_front() {
        if action == EditorAction::PlayTest {
            editor.finish_gesture();
            play.0 = true;
            break;
        }
        if *mode == ClientMode::PlayTest {
            continue;
        }
        editor.apply(action);
    }
}

impl Editor<'_> {
    pub fn apply(&mut self, action: EditorAction) {
        if !matches!(
            action,
            EditorAction::Hover(_) | EditorAction::Open(_) | EditorAction::NewMap
        ) {
            self.confirm.0 = None;
        }
        match action {
            EditorAction::Hover(pos) => self.cursor.0 = pos.filter(|p| p.is_finite()),
            EditorAction::Press(pos) => self.press(pos),
            EditorAction::Drag(pos) => self.drag(pos),
            EditorAction::Release => self.finish_gesture(),
            EditorAction::Pick(pos) => self.pick(pos),
            EditorAction::Cancel => self.cancel(),
            EditorAction::SetTool(tool) => {
                self.finish_gesture();
                self.tools.tool = tool;
            }
            EditorAction::SetBrush(size) if BRUSH_SIZES.contains(&size) => self.tools.brush = size,
            EditorAction::SetBrush(size) => self.status.warn(format!("Brush size {size} is not one of 1, 3, 5, 9")),
            EditorAction::SetPreset(preset) => self.set_preset(preset),
            EditorAction::SetElevationMode(mode) => {
                self.tools.elevation_mode = mode;
                self.tools.tool = Tool::Elevation;
            }
            EditorAction::SetElevation(level) => self.set_elevation(i32::from(level)),
            EditorAction::StepElevation(step) => self.set_elevation(i32::from(self.tools.elevation) + step),
            EditorAction::SetKind(kind) => {
                self.tools.kind = kind;
                self.tools.tool = Tool::Place;
            }
            EditorAction::SetOwner(owner) => self.set_owner(owner),
            EditorAction::SetSymmetry(symmetry) => self.tools.symmetry = symmetry,
            EditorAction::SelectSlot(slot) => self.select_slot(slot),
            EditorAction::DeleteSelected => self.delete_selected(),
            EditorAction::Undo => self.undo(true),
            EditorAction::Redo => self.undo(false),
            EditorAction::Save => self.save(),
            EditorAction::Open(path) => self.open(path),
            EditorAction::NewMap => self.new_map(),
            EditorAction::SetName(name) => self.set_name(name),
            EditorAction::SetSeed(seed) => {
                self.doc.edit(|s| s.seed = seed);
            }
            EditorAction::Resize { width, height } => self.resize(width, height),
            EditorAction::AddSlot => self.add_slot(),
            EditorAction::RemoveSlot(slot) => self.remove_slot(slot),
            EditorAction::SetSlotName(slot, name) => self.set_slot_name(slot, name),
            EditorAction::SetSlotFaction(slot, faction) => self.edit_slot(slot, |p| p.faction = faction),
            EditorAction::SetStartLocation(slot, tile) => self.set_start_location(slot, tile),
            EditorAction::SetCameraStart(slot, pos) => self.set_camera_start(slot, pos),
            EditorAction::Zoom { factor, at } => self.zoom_by(factor, at),
            EditorAction::FitMap => self.fit_map(),
            EditorAction::CenterOn(pos) if pos.is_finite() => self.view.center = pos,
            EditorAction::CenterOn(_) => {}
            EditorAction::FocusProblem(index) => self.focus_problem(index),
            EditorAction::ToggleHelp => self.ui.help = !self.ui.help,
            EditorAction::RefreshFiles => self.files.refresh(),
            // Handled by `apply_editor_actions`.
            EditorAction::PlayTest => {}
        }
        self.keep_references_valid();
    }

    /// After a change, drops a selection, slot or owner that no longer exists.
    fn keep_references_valid(&mut self) {
        let scenario = self.doc.scenario();
        let slot_exists = |id: PlayerId| scenario.players.iter().any(|p| p.id == id);
        let valid = match self.selection.0 {
            Some(Selection::Spawn(i)) => i < scenario.spawns.len(),
            Some(Selection::StartLocation(p)) => {
                scenario.players.iter().any(|s| s.id == p && s.start_location.is_some())
            }
            Some(Selection::CameraStart(p)) => scenario.players.iter().any(|s| s.id == p && s.camera_start.is_some()),
            None => true,
        };
        if !valid {
            self.selection.0 = None;
        }
        if !slot_exists(self.tools.slot)
            && let Some(first) = scenario.players.first()
        {
            self.tools.slot = first.id;
        }
        if self.tools.owner.is_some_and(|o| !slot_exists(o)) {
            self.tools.owner = None;
        }
    }

    fn set_preset(&mut self, preset: TilePresetId) {
        let Some(info) = self.doc.scenario().map.presets().get(preset.0 as usize) else {
            self.status.warn(format!("The map has no preset {}", preset.0));
            return;
        };
        self.status.info(format!("Preset: {}", info.name));
        self.tools.preset = preset;
        if self.tools.tool != Tool::Fill {
            self.tools.tool = Tool::Paint;
        }
    }

    fn set_elevation(&mut self, level: i32) {
        self.tools.elevation = level.clamp(0, i32::from(TOP_ELEVATION)) as u8;
        self.tools.elevation_mode = ElevationMode::Set;
        self.tools.tool = Tool::Elevation;
    }

    fn set_owner(&mut self, owner: Option<PlayerId>) {
        match owner {
            Some(p) if !self.doc.scenario().players.iter().any(|s| s.id == p) => {
                self.status.warn(format!("There is no player {}", p.0));
            }
            _ => self.tools.owner = owner,
        }
    }

    fn select_slot(&mut self, slot: PlayerId) {
        if self.doc.scenario().players.iter().any(|p| p.id == slot) {
            self.tools.slot = slot;
        } else {
            self.status.warn(format!("There is no player {}", slot.0));
        }
    }

    fn undo(&mut self, undo: bool) {
        self.finish_gesture();
        let (done, verb) = if undo {
            (self.doc.undo(), "Undid")
        } else {
            (self.doc.redo(), "Redid")
        };
        if done {
            self.status.info(format!("{verb} the last edit"));
        } else {
            self.status
                .info(if undo { "Nothing to undo" } else { "Nothing to redo" });
        }
    }
}
