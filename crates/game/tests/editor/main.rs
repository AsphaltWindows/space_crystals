//! Headless map editor tests. They drive the editor through its action layer (`EditorActions`), never raw input,
//! and check the document, the sim that shows it and the view (`ARCHITECTURE.md > Map editor`).

mod edits;
mod session;

use std::path::{Path, PathBuf};

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use sc_sim::{ObjectKind, Scenario, Sim, Spawn, SpawnAt, Tile, TilePos};
use space_crystals::editor::EditorConfig;
use space_crystals::editor::actions::{EditorAction, EditorActions};
use space_crystals::editor::doc::EditorDoc;
use space_crystals::editor::state::EditorStatus;
use space_crystals::editor::tools::EditorTools;
use space_crystals::object_render::ObjectVisual;
use space_crystals::platform::DisplayMode;
use space_crystals::sim_driver::SimRes;
use space_crystals::{AppConfig, build_app};

/// A fresh directory for one test's map files.
fn temp_maps(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sc_editor_test_{tag}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

/// A headless editor on `scenario`, after its first frame.
fn editor_with(scenario: Scenario, config: EditorConfig) -> App {
    let mut config = AppConfig::editor(scenario, config);
    config.display = DisplayMode::Headless;
    let mut app = build_app(config);
    app.insert_resource(TimeUpdateStrategy::FixedTimesteps(1));
    app.finish();
    app.cleanup();
    app.update();
    app
}

fn editor(scenario: Scenario, maps_dir: &Path) -> App {
    editor_with(
        scenario,
        EditorConfig {
            path: None,
            maps_dir: maps_dir.into(),
            script: Vec::new(),
        },
    )
}

/// Queues `actions` and runs one frame, which applies them.
fn act(app: &mut App, actions: impl IntoIterator<Item = EditorAction>) {
    let mut queue = app.world_mut().resource_mut::<EditorActions>();
    for action in actions {
        queue.push(action);
    }
    app.update();
}

fn doc(app: &App) -> &Scenario {
    app.world().resource::<EditorDoc>().scenario()
}

fn tools(app: &App) -> &EditorTools {
    app.world().resource::<EditorTools>()
}

fn status(app: &App) -> &EditorStatus {
    app.world().resource::<EditorStatus>()
}

fn undo_depth(app: &App) -> usize {
    app.world().resource::<EditorDoc>().undo_depth()
}

/// The center of tile (`x`, `y`) in sim space units.
fn at(x: i32, y: i32) -> Vec2 {
    TilePos::new(x, y).center()
}

fn tile(app: &App, x: i32, y: i32) -> Tile {
    doc(app).map.tile(TilePos::new(x, y)).unwrap()
}

fn sim(app: &App) -> &Sim {
    app.world().resource::<SimRes>().sim()
}

fn visual_count(app: &mut App) -> usize {
    let mut query = app.world_mut().query::<&ObjectVisual>();
    query.iter(app.world()).count()
}

fn patch(x: i32, y: i32) -> Spawn {
    Spawn {
        kind: ObjectKind::CrystalPatch,
        owner: None,
        at: SpawnAt::Tile(TilePos::new(x, y)),
    }
}
