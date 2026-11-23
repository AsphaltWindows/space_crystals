//! Headless map editor tests. They drive the editor through its action layer (`EditorActions`), and its input
//! through the shared input layer's virtual keyboard and mouse (`InjectedInput`), never raw input (except a text
//! field's own keys and the window's close button), and check the document, the sim that shows it and the view
//! (`ARCHITECTURE.md > Map editor`).

mod creation;
mod edits;
mod input;
mod review;
mod session;
mod symmetry;
mod units;

use std::path::{Path, PathBuf};

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use sc_sim::{MapDef, MapSettings, MapType, ObjectKind, Scenario, Sim, Spawn, SpawnAt, Symmetry, Tile, TilePos};
use space_crystals::editor::EditorConfig;
use space_crystals::editor::actions::{EditorAction, EditorActions};
use space_crystals::editor::doc::EditorDoc;
use space_crystals::editor::state::{EditorProblems, EditorSelection, EditorStatus, Selection};
use space_crystals::editor::tools::EditorTools;
use space_crystals::editor::ui::fields::TextField;
use space_crystals::input::InjectedInput;
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

/// A headless editor on `scenario`'s map, with its setup for play-tests, after its first frame.
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

/// A headless editor on a new (unsaved) document `map`, listing `maps_dir`.
fn editor(map: MapDef, maps_dir: &Path) -> App {
    editor_with(
        Scenario::new(map),
        EditorConfig {
            path: None,
            maps_dir: maps_dir.into(),
            script: Vec::new(),
        },
    )
}

/// A flat Custom map of `slots` slots with `symmetry`, as the creation dialog makes it.
fn created(symmetry: Symmetry, width: u32, height: u32, slots: usize) -> MapDef {
    MapSettings {
        map_type: MapType::Custom,
        width,
        height,
        slots,
        symmetry,
        name: "test".into(),
        description: String::new(),
    }
    .create()
    .unwrap()
}

/// Queues `actions` and runs one frame, which applies them.
fn act(app: &mut App, actions: impl IntoIterator<Item = EditorAction>) {
    let mut queue = app.world_mut().resource_mut::<EditorActions>();
    for action in actions {
        queue.push(action);
    }
    app.update();
}

fn doc(app: &App) -> &MapDef {
    app.world().resource::<EditorDoc>().map()
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

fn selection(app: &App) -> Option<Selection> {
    app.world().resource::<EditorSelection>().0
}

fn problems(app: &App) -> Vec<sc_sim::ScenarioProblem> {
    app.world().resource::<EditorProblems>().0.clone()
}

/// The center of tile (`x`, `y`) in sim space units.
fn at(x: i32, y: i32) -> Vec2 {
    TilePos::new(x, y).center()
}

/// The ground point that snaps a 2 × 2 footprint's top-left tile to (`x`, `y`).
fn station_at(x: i32, y: i32) -> Vec2 {
    TilePos::new(x + 1, y + 1).corner()
}

fn tile(app: &App, x: i32, y: i32) -> Tile {
    doc(app).terrain.tile(TilePos::new(x, y)).unwrap()
}

fn corner(app: &App, x: i32, y: i32) -> u8 {
    doc(app).terrain.corner(sc_sim::CornerPos::new(x, y)).unwrap()
}

fn sim(app: &App) -> &Sim {
    app.world().resource::<SimRes>().sim()
}

fn visual_count(app: &mut App) -> usize {
    let mut query = app.world_mut().query::<&ObjectVisual>();
    query.iter(app.world()).count()
}

fn patch(x: i32, y: i32) -> Spawn {
    Spawn::new(ObjectKind::CrystalPatch, None, SpawnAt::Tile(TilePos::new(x, y)))
}

/// Whether the last status message is a warning that contains `text`.
#[track_caller]
fn assert_warned(app: &App, text: &str) {
    let status = status(app);
    assert_eq!(
        status.level,
        space_crystals::editor::state::StatusLevel::Warning,
        "{}",
        status.message
    );
    assert!(status.message.contains(text), "{:?} lacks {text:?}", status.message);
}

/// The input layer's virtual keyboard and mouse.
fn input(app: &mut App) -> Mut<'_, InjectedInput> {
    app.world_mut().resource_mut::<InjectedInput>()
}

/// Taps `key` on the input layer's virtual keyboard while the keys `modifiers` are held, in one frame.
fn tap(app: &mut App, modifiers: &[KeyCode], key: KeyCode) {
    for &m in modifiers {
        input(app).keys.press(m);
    }
    input(app).tap_key(key);
    app.update();
    for &m in modifiers {
        input(app).keys.release(m);
    }
}

/// The entity of the text field `which`.
fn field(app: &mut App, which: TextField) -> Entity {
    let mut fields = app.world_mut().query::<(Entity, &TextField)>();
    let found = fields.iter(app.world()).find(|(_, f)| **f == which).map(|(e, _)| e);
    found.unwrap_or_else(|| panic!("no {which:?} field"))
}
