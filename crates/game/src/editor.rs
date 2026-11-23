//! The map editor (`cargo run -- --editor [path.json]`, `ARCHITECTURE.md > Map editor`).
//!
//! It edits a `Scenario` (the document, [`doc::EditorDoc`]) and draws it with the game's own renderer, camera and
//! object visuals: the sim shows [`sc_sim::Sim::preview`] of the document, paused at tick 0, and is replaced
//! whenever the document changes, so the map mesh and the object visuals catch up through their usual paths.
//!
//! Everything the editor does is an [`actions::EditorAction`]: raw input ([`input`]), UI buttons and fields
//! ([`ui`]), `--editor-script` files and tests queue them, and [`actions::apply_editor_actions`] applies them.
//! F5 play-tests the map in game mode ([`playtest`]) and comes back with the document and its undo history intact.
//!
//! Map design is `core/map.md` (tiles, presets, corner heights 0–16, resources on flat ground); player slots and
//! start locations are a placeholder model until design question Q0004 is answered.

pub mod actions;
pub mod doc;
pub mod document;
pub mod files;
pub mod gesture;
pub mod input;
pub mod objects;
pub mod overlay;
pub mod playtest;
pub mod state;
pub mod terrain;
pub mod tools;
pub mod ui;
pub mod view;

use std::path::PathBuf;

use bevy::prelude::*;
use sc_sim::{Scenario, Sim};

use self::actions::{EditorAction, EditorActions, PendingConfirmation, PlayTestRequest, apply_editor_actions};
use self::doc::EditorDoc;
use self::gesture::Gesture;
use self::state::{EditorCursor, EditorFiles, EditorProblems, EditorSelection, EditorStatus, EditorUi, StatusLevel};
use self::tools::EditorTools;
use crate::camera::ViewportInsets;
use crate::hud::HUD_INSETS;
use crate::sim_driver::{LoadedScenario, LocalPlayer, SimRes};
use crate::{ClientMode, ClientSystems};

/// How the editor starts.
#[derive(Clone, Debug, Default)]
pub struct EditorConfig {
    /// The document's file: Ctrl+S saves there. `None` for a new map, which is saved as `maps/<name>.json`.
    pub path: Option<PathBuf>,
    /// The directory the files panel lists and new maps are saved in (`maps/` at the repository root).
    pub maps_dir: PathBuf,
    /// Actions applied on the first frame (`--editor-script`): QA screenshots of scripted edits.
    pub script: Vec<EditorAction>,
}

/// Order of the editor's `Update` systems, all before [`ClientSystems::SyncSim`]: the frame's input becomes actions,
/// the actions change the document, and the sim catches up with it before the views sync with the sim.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum EditorSystems {
    /// Raw input → [`EditorAction`]s.
    Input,
    /// Apply the queued actions; start or end a play-test.
    Apply,
    /// Bring the sim, the validation problems and the viewport insets up to date with the document and the mode.
    Sync,
}

pub struct EditorPlugin {
    pub config: EditorConfig,
    /// The document as it starts.
    pub scenario: Scenario,
}

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        let problems = self.scenario.validate();
        let file = self
            .config
            .path
            .as_deref()
            .map(|p| (files::display_path(p), p.exists()));
        // Logged at startup (`log_starting_status`): a headless run's logger arrives after the plugins are built.
        let (level, message) = match (&file, problems.len()) {
            (Some((path, false)), _) => (
                StatusLevel::Info,
                format!("New map: Ctrl+S saves it as {path} (F1: help)"),
            ),
            (Some((path, true)), 0) => (StatusLevel::Info, format!("Editing {path} (F1: help)")),
            (Some((path, true)), n) => (
                StatusLevel::Warning,
                format!("Editing {path}: {n} problem(s), see Problems (F1: help)"),
            ),
            (None, _) => (StatusLevel::Info, "New map: Ctrl+S saves it in maps/ (F1: help)".into()),
        };
        let status = EditorStatus { message, level };
        let player = app.world().resource::<LocalPlayer>().0;
        let tools = EditorTools {
            slot: player,
            ..default()
        };
        app.insert_resource(EditorDoc::new(self.scenario.clone(), self.config.path.clone()))
            .insert_resource(tools)
            .insert_resource(status)
            .insert_resource(EditorFiles::new(self.config.maps_dir.clone()))
            .insert_resource(EditorProblems(problems))
            .insert_resource(EditorActions(self.config.script.iter().cloned().collect()))
            .insert_resource(ui::EDITOR_INSETS)
            .init_resource::<EditorSelection>()
            .init_resource::<EditorCursor>()
            .init_resource::<EditorUi>()
            .init_resource::<Gesture>()
            .init_resource::<PlayTestRequest>()
            .init_resource::<PendingConfirmation>()
            .init_resource::<playtest::PlayTestState>()
            .configure_sets(
                Update,
                (EditorSystems::Input, EditorSystems::Apply, EditorSystems::Sync)
                    .chain()
                    .before(ClientSystems::SyncSim),
            )
            .add_systems(
                Update,
                (
                    (input::editor_keys, input::editor_pointer).in_set(EditorSystems::Input),
                    (apply_editor_actions, playtest::switch_play_test)
                        .chain()
                        .in_set(EditorSystems::Apply),
                    (sync_preview, set_insets).in_set(EditorSystems::Sync),
                ),
            )
            .add_systems(Startup, log_starting_status)
            .add_plugins((overlay::EditorOverlayPlugin, ui::EditorUiPlugin));
    }
}

fn log_starting_status(status: Res<EditorStatus>) {
    status.log();
}

/// In the editor, the sim shows the document: whenever the document changes, a fresh [`Sim::preview`] of it
/// replaces the sim (which rebuilds the map mesh and the object visuals) and the problems are listed again.
fn sync_preview(
    mode: Res<ClientMode>,
    doc: Res<EditorDoc>,
    mut synced: Local<Option<u64>>,
    mut sim: ResMut<SimRes>,
    mut loaded: ResMut<LoadedScenario>,
    mut problems: ResMut<EditorProblems>,
) {
    if *mode != ClientMode::Editor || *synced == Some(doc.revision()) {
        return;
    }
    *synced = Some(doc.revision());
    let scenario = doc.scenario();
    let found = scenario.validate();
    if problems.0 != found {
        problems.0 = found;
    }
    if loaded.0 != *scenario {
        sim.replace(Sim::preview(scenario).0);
        loaded.0 = scenario.clone();
    }
}

/// The editor's panels frame the world view; a play-test has the game's HUD.
fn set_insets(mode: Res<ClientMode>, mut insets: ResMut<ViewportInsets>) {
    let wanted = match *mode {
        ClientMode::Editor => ui::EDITOR_INSETS,
        _ => HUD_INSETS,
    };
    insets.set_if_neq(wanted);
}
