//! The editor's action layer: everything the editor can do is an [`EditorAction`]. The input layer's actions, UI
//! buttons and text fields, `--editor-script` files and tests all queue actions in [`EditorActions`], and one system
//! applies them in order, generating the locked part of a symmetric map again after each one.

use std::collections::VecDeque;
use std::path::PathBuf;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use sc_sim::{Faction, MapType, ObjectKind, SlotId, SpawnSettings, TileLook, TilePos, TilePresetId};
use serde::{Deserialize, Serialize};

use super::creation::{NewMapDialog, NewMapField};
use super::doc::EditorDoc;
use super::gesture::{Gesture, GestureKind};
use super::orientation::Turn;
use super::playtest::PlayTestSetup;
use super::presets::PresetFlag;
use super::state::{EditorCursor, EditorFiles, EditorProblems, EditorSelection, EditorStatus, EditorUi, Selection};
use super::tools::{BRUSH_SIZES, EditorTools, ElevationMode, TOP_ELEVATION, Tool};
use crate::ClientMode;
use crate::camera::{CameraView, CameraZoom, ViewportInsets};
use crate::coords::drawn_position;
use crate::map_render::MapView;
use crate::platform::SurfaceSize;

/// Everything the editor can be asked to do. Positions are ground points in space units: what the cursor picks on
/// the drawn terrain (`crate::coords::ground_under`).
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
    /// Secondary click: picks the tile's preset and the height there for the terrain tools.
    Pick(Vec2),
    /// Abandons the current gesture, else closes the new map dialog or the help, else clears the selection.
    Cancel,

    // --- Tools ---
    SetTool(Tool),
    SetBrush(u32),
    /// Chooses the preset to paint (switching to Paint unless Fill is active).
    SetPreset(TilePresetId),
    /// Switches to the height tool in this mode.
    SetElevationMode(ElevationMode),
    /// Chooses the level that Set applies (switching to the height tool in Set mode).
    SetElevation(u8),
    /// Raises (positive) or lowers the level that Set applies, like `SetElevation`.
    StepElevation(i32),
    /// Chooses the object type to place (switching to Place).
    SetKind(ObjectKind),
    /// Turns the structure that the Place or the Layout tool places (R, Shift+R, F, Shift+F; `super::orientation`).
    Turn(Turn),
    /// Neutral or a player slot, for placed objects. Skirmish Default maps take neutral objects only.
    SetOwner(Option<SlotId>),
    /// The slot whose markers the marker tools place and whose Starting Layouts the Layout tool edits: an editable
    /// one.
    SelectSlot(SlotId),
    /// Chooses the faction whose Starting Layout the Layout tool edits (switching to it; `super::layouts`).
    SetLayoutFaction(Faction),
    /// Chooses the object type the Layout tool places (switching to it). On a Skirmish Default map, only one of the
    /// faction's Starting Conditions.
    SetLayoutKind(ObjectKind),
    DeleteSelected,
    /// Sets the per-object values of the spawn at this index (`core/map_editor.md > Per-object Resource Values`).
    SetSpawnSettings {
        spawn: usize,
        settings: SpawnSettings,
    },

    // --- Custom tile presets (`core/map_editor.md > Custom Tile Presets`) ---
    /// Adds a custom preset and chooses it for painting.
    AddPreset,
    SetPresetName(TilePresetId, String),
    TogglePresetFlag(TilePresetId, PresetFlag),
    SetPresetLook(TilePresetId, TileLook),
    /// Removes a custom preset that no tile uses.
    RemovePreset(TilePresetId),

    // --- The document ---
    Undo,
    Redo,
    /// Saves to the document's file, or to `maps/<name>.json` for a new map.
    Save,
    /// Opens a map file. With unsaved changes, it must be asked for twice.
    Open(PathBuf),
    /// Opens the creation dialog for a new map (`core/map_editor.md > Map Creation`). With unsaved changes, it
    /// must be asked for twice.
    NewMap,
    /// Changes a setting of the creation dialog.
    EditNewMap(NewMapField),
    /// Creates the map the creation dialog describes, if its settings make one.
    CreateMap,
    /// Closes the creation dialog.
    CancelNewMap,
    SetName(String),
    SetDescription(String),
    /// Resizes the map, anchored at the top-left; what falls outside is dropped and reported.
    Resize {
        width: u32,
        height: u32,
    },
    /// Adds a set of player slots: one, or one per symmetric image.
    AddSlots,
    /// Removes the set of player slots that this slot belongs to.
    RemoveSlots(SlotId),
    SetStart(SlotId, TilePos),
    SetCameraStart(SlotId, Option<Vec2>),

    // --- The play-test's game setup (not part of the map) ---
    SetPlayTestFaction(SlotId, Faction),
    /// The slot the local player takes in play-tests.
    SetPlayTestSlot(SlotId),
    SetPlayTestSeed(u64),

    // --- The view ---
    /// Shows `factor` times as much map, keeping the point `at` under the cursor where it is.
    Zoom {
        factor: f32,
        at: Option<Vec2>,
    },
    /// Zooms out to show the whole map.
    FitMap,
    /// Centres the view on this ground point (where it is drawn).
    CenterOn(Vec2),
    /// Centres the view on the validation problem at this index of the Problems list and selects its spawn.
    FocusProblem(usize),
    ToggleHelp,
    /// Shows or hides the tile grid (like the G key).
    ToggleGrid,
    /// Shows or hides the overlay of flat, sloped and too steep tiles.
    ToggleSlopes,
    RefreshFiles,
    /// Starts a play-test of the map, or ends the running one.
    PlayTest,
    /// Closes the editor (the window's close button). With unsaved changes, it must be asked for twice.
    Quit,
}

impl EditorAction {
    /// Whether the action may apply while the creation dialog is open: the dialog's own, and those that leave
    /// the document alone.
    fn allowed_in_dialog(&self) -> bool {
        use EditorAction as A;
        matches!(
            self,
            A::EditNewMap(_)
                | A::CreateMap
                | A::CancelNewMap
                | A::NewMap
                | A::Cancel
                | A::Release
                | A::Save
                | A::Zoom { .. }
                | A::FitMap
                | A::CenterOn(_)
                | A::ToggleHelp
                | A::ToggleGrid
                | A::ToggleSlopes
                | A::RefreshFiles
                | A::Quit
        )
    }
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
    pub dialog: ResMut<'w, NewMapDialog>,
    pub playtest: ResMut<'w, PlayTestSetup>,
    pub view: ResMut<'w, CameraView>,
    pub zoom: ResMut<'w, CameraZoom>,
    pub size: Res<'w, SurfaceSize>,
    pub insets: Res<'w, ViewportInsets>,
    pub map_view: ResMut<'w, MapView>,
    pub exit: MessageWriter<'w, AppExit>,
}

/// Applies the queued actions in order, generating the locked part of a symmetric map after each one that changed
/// the map. A play-test request stops the queue for this frame, so that the actions after it apply once the switch
/// is done. While a play-test runs, only the requests to end it or to quit apply.
pub fn apply_editor_actions(
    mut queue: ResMut<EditorActions>,
    mode: Res<ClientMode>,
    mut play: ResMut<PlayTestRequest>,
    mut editor: Editor,
) {
    if queue.0.is_empty() {
        return;
    }
    while let Some(action) = queue.0.pop_front() {
        if action == EditorAction::PlayTest {
            if editor.dialog.open {
                editor.status.warn("Create or cancel the new map first");
                continue;
            }
            editor.finish_gesture();
            play.0 = true;
            break;
        }
        if *mode == ClientMode::PlayTest && action != EditorAction::Quit {
            continue;
        }
        editor.apply(action);
        editor.regenerate();
    }
}

impl Editor<'_> {
    pub fn apply(&mut self, action: EditorAction) {
        if let EditorAction::Hover(pos) = action {
            let pos = pos.filter(|p| p.is_finite());
            if self.cursor.0 != pos {
                self.cursor.0 = pos;
            }
            return;
        }
        if self.dialog.open && !action.allowed_in_dialog() {
            self.status.warn("Create or cancel the new map first (Esc cancels)");
            return;
        }
        if !matches!(
            action,
            EditorAction::Open(_) | EditorAction::NewMap | EditorAction::Quit
        ) && self.confirm.0.is_some()
        {
            self.confirm.0 = None;
        }
        match action {
            EditorAction::Hover(_) => {}
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
            EditorAction::Turn(turn) => self.turn(turn),
            EditorAction::SetOwner(owner) => self.set_owner(owner),
            EditorAction::SelectSlot(slot) => {
                if self.slot_editable(slot) {
                    self.tools.slot = slot;
                }
            }
            EditorAction::SetLayoutFaction(faction) => self.set_layout_faction(faction),
            EditorAction::SetLayoutKind(kind) => self.set_layout_kind(kind),
            EditorAction::DeleteSelected => self.delete_selected(),
            EditorAction::SetSpawnSettings { spawn, settings } => self.set_spawn_settings(spawn, settings),
            EditorAction::AddPreset => self.add_preset(),
            EditorAction::SetPresetName(preset, name) => self.set_preset_name(preset, name),
            EditorAction::TogglePresetFlag(preset, flag) => self.toggle_preset_flag(preset, flag),
            EditorAction::SetPresetLook(preset, look) => self.edit_preset(preset, |p| p.look = look),
            EditorAction::RemovePreset(preset) => self.remove_preset(preset),
            EditorAction::Undo => self.undo(true),
            EditorAction::Redo => self.undo(false),
            EditorAction::Save => self.save(),
            EditorAction::Open(path) => self.open(path),
            EditorAction::NewMap => self.new_map(),
            EditorAction::EditNewMap(field) => self.edit_new_map(field),
            EditorAction::CreateMap => self.create_map(),
            EditorAction::CancelNewMap => self.cancel_new_map(),
            EditorAction::SetName(name) => self.set_name(name),
            EditorAction::SetDescription(text) => self.set_description(text),
            EditorAction::Resize { width, height } => self.resize(width, height),
            EditorAction::AddSlots => self.add_slots(),
            EditorAction::RemoveSlots(slot) => self.remove_slots(slot),
            EditorAction::SetStart(slot, tile) => self.set_start(slot, tile),
            EditorAction::SetCameraStart(slot, pos) => self.set_camera_start(slot, pos),
            EditorAction::SetPlayTestFaction(slot, faction) => self.set_play_test_faction(slot, faction),
            EditorAction::SetPlayTestSlot(slot) => self.set_play_test_slot(slot),
            EditorAction::SetPlayTestSeed(seed) => {
                if self.playtest.seed != seed {
                    self.playtest.seed = seed;
                }
            }
            EditorAction::Zoom { factor, at } => self.zoom_by(factor, at),
            EditorAction::FitMap => self.fit_map(),
            EditorAction::CenterOn(pos) if pos.is_finite() => self.center_on(pos),
            EditorAction::CenterOn(_) => {}
            EditorAction::FocusProblem(index) => self.focus_problem(index),
            EditorAction::ToggleHelp => self.ui.help = !self.ui.help,
            EditorAction::ToggleGrid => self.map_view.grid_visible = !self.map_view.grid_visible,
            EditorAction::ToggleSlopes => self.ui.slopes = !self.ui.slopes,
            EditorAction::RefreshFiles => self.files.refresh(),
            EditorAction::Quit => self.quit(),
            // Handled by `apply_editor_actions`.
            EditorAction::PlayTest => {}
        }
        self.keep_references_valid();
    }

    /// Generates the locked part of a symmetric map again if the last action changed the map
    /// ([`EditorDoc::regenerate`]). That lists the spawns of the editable part first, in their order: the
    /// selection and a dragged spawn keep pointing at the same spawn.
    pub(super) fn regenerate(&mut self) {
        if !self.doc.is_stale() {
            return;
        }
        let map = self.doc.map();
        if map.image_count() > 1 {
            let editable: Vec<bool> = map.spawns.iter().map(|s| map.is_spawn_editable(s)).collect();
            let new_index = |i: usize| {
                let kept = *editable.get(i)?;
                kept.then(|| editable[..i].iter().filter(|e| **e).count())
            };
            if let Some(Selection::Spawn(i)) = self.selection.0 {
                let moved = new_index(i).map(Selection::Spawn);
                if moved != self.selection.0 {
                    self.selection.0 = moved;
                }
            }
            if let Some(GestureKind::MoveSpawn { index, .. }) = &self.gesture.0 {
                match new_index(*index) {
                    Some(new) if new == *index => {}
                    Some(new) => {
                        if let Some(GestureKind::MoveSpawn { index, .. }) = &mut self.gesture.0 {
                            *index = new;
                        }
                    }
                    None => self.gesture.0 = None,
                }
            }
        }
        self.doc.regenerate();
    }

    /// Centres the view on where the ground point `pos` is drawn.
    pub(super) fn center_on(&mut self, pos: Vec2) {
        self.view.center = drawn_position(&self.doc.map().terrain, pos);
    }

    /// After a change, drops a selection, slot or owner that no longer exists or lies in the locked part, and
    /// resets a preset that the map no longer has (a map opened, created or resized). Writes only what changes,
    /// so that the UI doesn't redraw for nothing.
    fn keep_references_valid(&mut self) {
        let map = self.doc.map();
        // Review finding: another map's preset table may be shorter than the custom preset chosen before.
        if map.terrain.presets().get(self.tools.preset.0 as usize).is_none() {
            self.tools.preset = TilePresetId::PLANE;
        }
        let valid = match self.selection.0 {
            Some(Selection::Spawn(i)) => map.spawns.get(i).is_some_and(|s| map.is_spawn_editable(s)),
            Some(Selection::Start(s)) => map.slot(s).is_some() && map.is_slot_editable(s),
            Some(Selection::CameraStart(s)) => {
                map.slot(s).is_some_and(|slot| slot.camera_start.is_some()) && map.is_slot_editable(s)
            }
            // Checked with the Layout tool's references, below.
            Some(Selection::LayoutObject { .. }) | None => true,
        };
        if !valid {
            self.selection.0 = None;
        }
        // The marker tools' slot: an editable slot of the map (its set's editable slot, or the first).
        let slot = self.tools.slot;
        if map.slot(slot).is_none() || !map.is_slot_editable(slot) {
            let fixed = if map.slot(slot).is_some() {
                map.source_slot(slot)
            } else {
                SlotId(1)
            };
            if fixed != slot {
                self.tools.slot = fixed;
            }
        }
        let skirmish = map.map_type == MapType::SkirmishDefault;
        if self.tools.owner.is_some_and(|o| skirmish || map.slot(o).is_none()) {
            self.tools.owner = None;
        }
        if map.slot(self.playtest.slot).is_none() && self.playtest.slot != SlotId(1) {
            self.playtest.slot = SlotId(1);
        }
        self.keep_layout_references_valid();
    }

    fn set_preset(&mut self, preset: TilePresetId) {
        let Some(info) = self.doc.map().terrain.presets().get(preset.0 as usize) else {
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

    fn set_owner(&mut self, owner: Option<SlotId>) {
        let map = self.doc.map();
        match owner {
            Some(slot) if map.slot(slot).is_none() => self.status.warn(format!("There is no slot {}", slot.0)),
            Some(_) if map.map_type == MapType::SkirmishDefault => self
                .status
                .warn("A Skirmish Default map takes neutral objects only: owned ones go in the Starting Layouts"),
            _ => self.tools.owner = owner,
        }
    }

    /// `core/map_editor.md > Per-object Resource Values`: values of the spawn's own kind that make sense, on a
    /// spawn of the editable part (its images get them too).
    fn set_spawn_settings(&mut self, spawn: usize, settings: SpawnSettings) {
        let map = self.doc.map();
        let Some(current) = map.spawns.get(spawn) else {
            self.status.warn(format!("There is no spawn #{spawn}"));
            return;
        };
        let name = current.kind.def().name;
        if !map.is_spawn_editable(current) {
            self.refuse_locked(&format!("spawn #{spawn} ({name})"));
            return;
        }
        if settings.kind() != current.kind {
            self.status.warn(format!("These values aren't for a {name}"));
            return;
        }
        if let Err(problem) = settings.check() {
            self.status.warn(format!("{name}: {problem}"));
            return;
        }
        if current.settings == Some(settings) {
            return;
        }
        self.finish_gesture();
        self.doc.edit(|m| m.spawns[spawn].settings = Some(settings));
        self.status.info(format!("Changed the values of #{spawn} {name}"));
    }

    fn undo(&mut self, undo: bool) {
        self.finish_gesture();
        // Review finding: selections are spawn indexes, which an undone or redone edit may shift.
        self.selection.0 = None;
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
