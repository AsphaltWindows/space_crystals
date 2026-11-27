//! The map editor (`cargo run -- --editor [path.json]`, `ARCHITECTURE.md > Map editor`).
//!
//! It edits a map (`sc_sim::MapDef`, the document: [`doc::EditorDoc`]) and draws it with the game's own renderer,
//! camera and object visuals: the sim shows [`sc_sim::Sim::preview`] of the map, paused at tick 0, and is replaced
//! whenever the document changes, so the map mesh and the object visuals catch up through their usual paths.
//!
//! Everything the editor does is an [`actions::EditorAction`]: the shared input layer's actions and pointer
//! ([`input`]), UI buttons and fields ([`ui`]), `--editor-script` files and tests queue them, and
//! [`actions::apply_editor_actions`] applies them. On a symmetric map only the editable part is edited; after every
//! action the locked part is generated again from it (`core/map_editor.md > Symmetry`). New maps come from the
//! creation dialog ([`creation`]); maps may define custom tile presets ([`presets`]). Each slot's Starting Layouts
//! are edited with the Layout tool ([`layouts`]), and the structures the tools place are turned with R and F
//! ([`orientation`]). F5 play-tests the map in game mode with the play-test's game setup ([`playtest`]) and comes
//! back with the document and its undo history intact.
//!
//! Map design is `core/map.md` (tiles, presets, corner heights 0–16, resources on flat ground) and
//! `core/map_editor.md` (map types, symmetry, player slots and their markers).

pub mod actions;
pub mod creation;
pub mod doc;
pub mod document;
pub mod files;
pub mod gesture;
pub mod input;
pub mod layouts;
pub mod objects;
pub mod orientation;
pub mod overlay;
pub mod playtest;
pub mod presets;
pub mod state;
pub mod terrain;
pub mod tools;
pub mod ui;
pub mod view;

use std::path::PathBuf;

use bevy::prelude::*;
use sc_sim::{Faction, MapDef, Scenario, Sim, SlotId};

use self::actions::{EditorAction, EditorActions, PendingConfirmation, PlayTestRequest, apply_editor_actions};
use self::creation::NewMapDialog;
use self::doc::EditorDoc;
use self::gesture::Gesture;
use self::layouts::check::{LayoutPreview, update_layout_preview};
use self::playtest::PlayTestSetup;
use self::state::{EditorCursor, EditorFiles, EditorProblems, EditorSelection, EditorStatus, EditorUi, StatusLevel};
use self::tools::EditorTools;
use crate::camera::ViewportInsets;
use crate::hud::HUD_INSETS;
use crate::platform::ConfirmClose;
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
    /// The input layer's actions and the text fields → [`EditorAction`]s.
    Input,
    /// Apply the queued actions; start or end a play-test.
    Apply,
    /// Bring the sim, the validation problems and the viewport insets up to date with the document and the mode.
    Sync,
}

pub struct EditorPlugin {
    pub config: EditorConfig,
    /// The map as it starts, and the game setup whose players, seed and game options its play-tests start with.
    pub scenario: Scenario,
}

/// The play-test setup that `scenario`'s setup describes, played from `slot` (the first slot if the map has no
/// such slot).
fn play_test_setup(scenario: &Scenario, slot: SlotId) -> PlayTestSetup {
    let map = &scenario.map;
    let mut factions = vec![Faction::Gdo; map.slots.len()];
    for player in &scenario.setup.players {
        if let Some(faction) = player.slot.index().and_then(|i| factions.get_mut(i)) {
            *faction = player.faction;
        }
    }
    PlayTestSetup {
        factions,
        slot: if map.slot(slot).is_some() { slot } else { SlotId(1) },
        seed: scenario.setup.seed,
        config: scenario.setup.config,
    }
}

/// The status line the editor starts with.
fn starting_status(map: &MapDef, path: Option<&std::path::Path>, problems: usize) -> EditorStatus {
    let file = path.map(|p| (files::display_path(p), p.exists()));
    let (level, message) = match (&file, problems) {
        (Some((path, false)), _) => (
            StatusLevel::Info,
            format!("New map: Ctrl+S saves it as {path} (F1: help)"),
        ),
        (Some((path, true)), 0) => (StatusLevel::Info, format!("Editing {path} (F1: help)")),
        (Some((path, true)), n) => (
            StatusLevel::Warning,
            format!("Editing {path}: {n} problem(s), see Problems (F1: help)"),
        ),
        (None, _) => (
            StatusLevel::Info,
            format!("New map {}: Ctrl+S saves it in maps/ (F1: help)", map.name),
        ),
    };
    EditorStatus { message, level }
}

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::tuning::ClientTuning>();
        let map = &self.scenario.map;
        let player = app.world().resource::<LocalPlayer>().0;
        let playtest = play_test_setup(&self.scenario, SlotId(player.0));
        let problems = playtest.problems(map);
        // Logged at startup (`log_starting_status`): a headless run's logger arrives after the plugins are built.
        let status = starting_status(map, self.config.path.as_deref(), problems.len());
        let tools = EditorTools {
            slot: map.source_slot(playtest.slot),
            ..default()
        };
        app.insert_resource(EditorDoc::new(map.clone(), self.config.path.clone()))
            .insert_resource(tools)
            .insert_resource(status)
            .insert_resource(playtest)
            .insert_resource(EditorFiles::new(self.config.maps_dir.clone()))
            .insert_resource(EditorProblems(problems))
            .insert_resource(EditorActions(self.config.script.iter().cloned().collect()))
            .insert_resource(ui::EDITOR_INSETS)
            // The editor confirms closing the window when there are unsaved changes.
            .init_resource::<ConfirmClose>()
            .init_resource::<NewMapDialog>()
            .init_resource::<EditorSelection>()
            .init_resource::<EditorCursor>()
            .init_resource::<EditorUi>()
            .init_resource::<Gesture>()
            .init_resource::<PlayTestRequest>()
            .init_resource::<PendingConfirmation>()
            .init_resource::<playtest::PlayTestState>()
            .init_resource::<LayoutPreview>()
            .configure_sets(
                Update,
                (EditorSystems::Input, EditorSystems::Apply, EditorSystems::Sync)
                    .chain()
                    .before(ClientSystems::SyncSim),
            )
            .add_systems(
                Update,
                (
                    (input::editor_keys, input::editor_pointer, input::close_requests)
                        .chain()
                        .in_set(EditorSystems::Input),
                    (apply_editor_actions, playtest::switch_play_test)
                        .chain()
                        .in_set(EditorSystems::Apply),
                    (
                        sync_preview,
                        sync_problems,
                        set_insets,
                        update_layout_preview,
                        input::report_placing,
                    )
                        .in_set(EditorSystems::Sync),
                ),
            )
            .add_systems(Startup, log_starting_status)
            .add_plugins((overlay::EditorOverlayPlugin, ui::EditorUiPlugin));
    }
}

fn log_starting_status(status: Res<EditorStatus>) {
    status.log();
}

/// What the editor's preview was last brought up to date with: the document's revision, and the faction whose
/// layouts it shows instead of the GDO's ([`layouts::previewed_faction`]).
type PreviewKey = (u64, Option<Faction>);

/// In the editor, the sim shows the document: whenever the document changes, a fresh [`Sim::preview`] of its map
/// replaces the sim (which rebuilds the map mesh and the object visuals). Its players are those of the map's default
/// setup, or, while the Layout tool edits a faction's layouts, all of that faction ([`layouts::preview_scenario`]).
/// A play-test replaces the sim; when it ends, the preview is checked again.
fn sync_preview(
    mode: Res<ClientMode>,
    doc: Res<EditorDoc>,
    tools: Res<EditorTools>,
    mut synced: Local<Option<PreviewKey>>,
    mut sim: ResMut<SimRes>,
    mut loaded: ResMut<LoadedScenario>,
) {
    if *mode != ClientMode::Editor {
        *synced = None;
        return;
    }
    let faction = layouts::previewed_faction(&tools);
    let key = (doc.revision(), faction);
    if *synced == Some(key) {
        return;
    }
    let before = synced.replace(key);
    let map = doc.map();
    // The setup counts only while a faction is or was previewed: until the first edit, the editor shows the setup
    // it was started with.
    let faction_shown = faction.is_some() || before.is_some_and(|(_, f)| f.is_some());
    let scenario = layouts::preview_scenario(map, faction);
    if loaded.0.map != *map || (faction_shown && loaded.0.setup != scenario.setup) {
        sim.replace(Sim::preview(&scenario).0);
        loaded.0 = scenario;
    }
}

/// The Problems list follows the document and the play-test's setup: whenever either changes in the editor, and when
/// a play-test ends, the problems are listed again ([`PlayTestSetup::problems`]: what keeps a play-test from
/// starting, the layouts of different slots in each other's way included).
fn sync_problems(
    mode: Res<ClientMode>,
    doc: Res<EditorDoc>,
    setup: Res<PlayTestSetup>,
    mut listed: Local<Option<u64>>,
    mut problems: ResMut<EditorProblems>,
) {
    if *mode != ClientMode::Editor {
        *listed = None;
        return;
    }
    if *listed == Some(doc.revision()) && !setup.is_changed() {
        return;
    }
    *listed = Some(doc.revision());
    let found = setup.problems(doc.map());
    if problems.0 != found {
        problems.0 = found;
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
