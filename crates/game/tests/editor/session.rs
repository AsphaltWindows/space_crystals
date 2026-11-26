//! The editor session: what it shows, files, play-testing with its game setup, scripts, zoom, and the maps in the
//! repository.

use bevy::prelude::*;
use sc_sim::{Faction, PlayerId, Scenario, Sim, SlotId, TilePresetId, scenarios};
use space_crystals::ClientMode;
use space_crystals::camera::{CameraView, CameraZoom, ViewportInsets, viewport_rect, visible_extent_su};
use space_crystals::coords::{drawn_bounds, drawn_position, lift};
use space_crystals::editor::EditorConfig;
use space_crystals::editor::actions::EditorAction as A;
use space_crystals::editor::creation::NewMapDialog;
use space_crystals::editor::doc::EditorDoc;
use space_crystals::editor::files;
use space_crystals::editor::state::{EditorFiles, StatusLevel};
use space_crystals::editor::tools::Tool;
use space_crystals::editor::ui::{EDITOR_INSETS, ShownIn};
use space_crystals::editor::view::MIN_ZOOM;
use space_crystals::hud::{HUD_INSETS, HudRoot};
use space_crystals::input::{ActionState, CommandSlot, InputAction, InputContext};
use space_crystals::platform::SurfaceSize;
use space_crystals::sim_driver::{LoadedScenario, LocalPlayer, SimControl};

use super::{
    act, assert_warned, at, doc, editor, editor_with, patch, sim, status, tap, temp_maps, tile, tools, undo_depth,
    visual_count,
};

#[test]
fn the_editor_shows_the_map_without_the_game_hud() {
    let mut app = editor(scenarios::dev_skirmish_map(), &temp_maps("hud"));
    assert_eq!(*app.world().resource::<ClientMode>(), ClientMode::Editor);
    assert_eq!(*app.world().resource::<ViewportInsets>(), EDITOR_INSETS);
    let mut hud = app.world_mut().query_filtered::<&Visibility, With<HudRoot>>();
    assert!(hud.iter(app.world()).all(|v| *v == Visibility::Hidden));
    // The sim shows the document, paused at tick 0.
    app.update();
    assert_eq!(sim(&app).tick(), 0);
    assert_eq!(*sim(&app).map(), scenarios::dev_skirmish_map().terrain);
    // The pre-placed objects, and the starting base of each player of the default setup: its slot's GDO layout.
    let spawns = doc(&app).spawns.len();
    let bases: usize = doc(&app).slots.iter().map(|s| s.layouts.get(Faction::Gdo).len()).sum();
    assert_eq!((spawns, bases), (17, 8));
    assert_eq!(visual_count(&mut app), spawns + bases);
}

#[test]
fn saving_and_opening_round_trip_the_map() {
    let dir = temp_maps("files");
    let mut app = editor(scenarios::dev_skirmish_map(), &dir);
    act(&mut app, [A::SetDescription("Two bases, one plateau".into()), A::Save]);
    let path = dir.join("dev_skirmish.json");
    assert!(!app.world().resource::<EditorDoc>().is_dirty());
    assert_eq!(app.world().resource::<EditorDoc>().path(), Some(path.as_path()));
    let saved = doc(&app).clone();
    assert_eq!(files::load(&path), Ok(saved.clone()));
    assert_eq!(app.world().resource::<EditorFiles>().maps, vec![path.clone()]);
    // Review finding: saves are atomic (a temporary file, then a rename), and leave nothing else behind.
    assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);

    // A new map from the creation dialog; a second save goes to the same file.
    act(&mut app, [A::NewMap]);
    assert!(app.world().resource::<NewMapDialog>().open);
    act(&mut app, [A::CreateMap]);
    assert!(!app.world().resource::<NewMapDialog>().open);
    let terrain = &doc(&app).terrain;
    assert_eq!((terrain.width(), terrain.height()), files::NEW_MAP_SIZE);
    assert_eq!(app.world().resource::<EditorDoc>().path(), None);
    assert!(
        !app.world().resource::<EditorDoc>().can_undo(),
        "a new document has no history"
    );

    // Opening with unsaved changes needs a second request.
    act(&mut app, [A::SetName("scratch".into()), A::Open(path.clone())]);
    assert_eq!(doc(&app).name, "scratch");
    assert_warned(&app, "Unsaved changes");
    act(&mut app, [A::Open(path.clone())]);
    assert_eq!(*doc(&app), saved);
    assert_eq!(*sim(&app).map(), saved.terrain);
    act(&mut app, [A::SetDescription("Changed".into()), A::Save]);
    assert_eq!(files::load(&path).unwrap().description, "Changed");
    assert_eq!(app.world().resource::<EditorFiles>().maps.len(), 1);

    act(&mut app, [A::Open(dir.join("missing.json"))]);
    assert_eq!(status(&app).level, StatusLevel::Error);
    assert_eq!(doc(&app).description, "Changed", "a failed open keeps the document");
    std::fs::remove_dir_all(&dir).unwrap();
}

/// The play-test's game setup is no part of the map: each slot's faction, the slot you play and the seed are chosen
/// in the editor, and play-tests run with debug commands on.
#[test]
fn play_testing_runs_the_map_with_its_setup_and_returns_to_the_editor() {
    let mut map = scenarios::empty_map(32, 24);
    map.spawns.push(patch(4, 4));
    let mut app = editor(map, &temp_maps("play"));
    act(
        &mut app,
        [
            A::SetPreset(TilePresetId::WATER),
            A::Press(at(10, 10)),
            A::Release,
            A::Zoom { factor: 2.0, at: None },
        ],
    );
    let edited = doc(&app).clone();
    let (center, zoom) = (
        app.world().resource::<CameraView>().center,
        app.world().resource::<CameraZoom>().0,
    );
    assert_eq!(zoom, 2.0);

    act(
        &mut app,
        [
            A::SetPlayTestFaction(SlotId(2), Faction::Cults),
            A::SetPlayTestSlot(SlotId(2)),
            A::SetPlayTestSeed(7),
            A::PlayTest,
        ],
    );
    assert_eq!(*app.world().resource::<ClientMode>(), ClientMode::PlayTest);
    assert_eq!(*doc(&app), edited, "the setup is no part of the map");
    app.update();
    assert_eq!(*app.world().resource::<ViewportInsets>(), HUD_INSETS);
    assert_eq!(app.world().resource::<CameraZoom>().0, 1.0);
    let mut hud = app.world_mut().query_filtered::<&Visibility, With<HudRoot>>();
    assert!(hud.iter(app.world()).all(|v| *v == Visibility::Inherited));
    let mut roots = app.world_mut().query::<(&ShownIn, &Visibility)>();
    for (shown_in, visibility) in roots.iter(app.world()) {
        let expected = if shown_in.0 == ClientMode::PlayTest {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        assert_eq!(*visibility, expected);
    }
    let loaded = &app.world().resource::<LoadedScenario>().0;
    assert_eq!(loaded.map, edited);
    assert_eq!(loaded.setup.seed, 7);
    assert!(loaded.setup.config.debug, "play-tests run with debug commands");
    assert!(sim(&app).config().debug);
    assert_eq!(app.world().resource::<LocalPlayer>().0, PlayerId(2));
    let factions: Vec<Faction> = sim(&app).players().iter().map(|p| p.faction).collect();
    assert_eq!(factions, [Faction::Gdo, Faction::Cults]);
    let started = sim(&app).tick();
    for _ in 0..5 {
        app.update();
    }
    assert_eq!(sim(&app).tick(), started + 5, "the game runs");
    // Edits wait for the editor.
    act(&mut app, [A::Undo, A::SetName("later".into())]);
    assert_eq!(*doc(&app), edited);

    act(&mut app, [A::PlayTest]);
    app.update();
    assert_eq!(*app.world().resource::<ClientMode>(), ClientMode::Editor);
    assert_eq!(sim(&app).tick(), 0);
    assert!(app.world().resource::<SimControl>().paused);
    assert_eq!(*doc(&app), edited);
    assert_eq!(undo_depth(&app), 1, "the history is intact");
    assert_eq!(app.world().resource::<CameraView>().center, center);
    assert_eq!(app.world().resource::<CameraZoom>().0, zoom);
    assert_eq!(*app.world().resource::<ViewportInsets>(), EDITOR_INSETS);
    act(&mut app, [A::Undo]);
    assert_eq!(tile(&app, 10, 10).preset, TilePresetId::PLANE);

    // An invalid map can't be play-tested.
    app.world_mut()
        .resource_mut::<EditorDoc>()
        .edit(|m| m.spawns.push(patch(40, 40)));
    act(&mut app, [A::PlayTest]);
    assert_eq!(*app.world().resource::<ClientMode>(), ClientMode::Editor);
    assert_eq!(status(&app).level, StatusLevel::Error);
}

/// In the game Esc is Cancel (`systems/controls/interface.md`): only F5 ends a play-test. A play-test takes
/// the game's bindings, so the editor's keys do nothing there.
#[test]
fn only_f5_ends_a_play_test() {
    let mut app = editor(scenarios::empty_map(32, 24), &temp_maps("f5"));
    let context = |app: &App| app.world().resource::<ActionState>().context();
    assert_eq!(context(&app), InputContext::Editor);
    tap(&mut app, &[], KeyCode::F5);
    assert_eq!(*app.world().resource::<ClientMode>(), ClientMode::PlayTest);
    tap(&mut app, &[], KeyCode::Escape);
    assert_eq!(*app.world().resource::<ClientMode>(), ClientMode::PlayTest);
    assert_eq!(context(&app), InputContext::Game);
    let pressed = |app: &App| {
        app.world()
            .resource::<ActionState>()
            .pressed_actions()
            .collect::<Vec<_>>()
    };
    assert_eq!(pressed(&app), [InputAction::Cancel], "the game's Cancel");
    // B (the Paint tool) is nothing in the game, and V is the game's command slot, not the Select tool.
    tap(&mut app, &[], KeyCode::KeyB);
    tap(&mut app, &[], KeyCode::KeyH);
    assert_eq!(pressed(&app), [], "H is no binding of the game");
    tap(&mut app, &[], KeyCode::KeyV);
    assert_eq!(
        pressed(&app),
        [InputAction::CommandSlot(CommandSlot::new(2, 3).unwrap())]
    );
    tap(&mut app, &[], KeyCode::F5);
    assert_eq!(*app.world().resource::<ClientMode>(), ClientMode::Editor);
    assert_eq!(tools(&app).tool, Tool::Select, "the editor saw none of the game's keys");
    tap(&mut app, &[], KeyCode::KeyB);
    assert_eq!(context(&app), InputContext::Editor);
    assert_eq!(tools(&app).tool, Tool::Paint);
}

#[test]
fn a_script_runs_on_the_first_frame() {
    let script = vec![
        A::SetPreset(TilePresetId::CLIFF),
        A::Press(at(3, 3)),
        A::Release,
        A::FitMap,
    ];
    let app = editor_with(
        scenarios::empty(16, 12),
        EditorConfig {
            path: None,
            maps_dir: temp_maps("script"),
            script,
        },
    );
    assert_eq!(tile(&app, 3, 3).preset, TilePresetId::CLIFF);
    assert!(app.world().resource::<CameraZoom>().0 >= MIN_ZOOM);
}

#[test]
fn zoom_reaches_the_whole_map_and_stays_in_range() {
    let mut app = editor(scenarios::dev_skirmish_map(), &temp_maps("zoom"));
    act(&mut app, [A::FitMap]);
    let zoom = app.world().resource::<CameraZoom>().0;
    let size = *app.world().resource::<SurfaceSize>();
    let viewport = viewport_rect(size, EDITOR_INSETS).1;
    let visible = visible_extent_su(viewport, size.physical.x, zoom);
    // The whole map as drawn: its raised ground reaches above its northern edge.
    let drawn = drawn_bounds(&doc(&app).terrain);
    assert!(
        visible.x >= drawn.width() && visible.y >= drawn.height(),
        "{visible} shows all of {drawn:?}"
    );
    assert_eq!(app.world().resource::<CameraView>().center, drawn.center());

    act(
        &mut app,
        [A::Zoom {
            factor: 1000.0,
            at: None,
        }],
    );
    let max = app.world().resource::<CameraZoom>().0;
    assert!(max > zoom && max < zoom * 2.0, "{max}");
    act(
        &mut app,
        [A::Zoom {
            factor: 0.0001,
            at: None,
        }],
    );
    assert_eq!(app.world().resource::<CameraZoom>().0, MIN_ZOOM);
    // Centring on a ground point centres on where it is drawn; zooming at a point keeps where it is drawn in place
    // on screen. Both points lie on the central plateau, at level 3.
    let map = doc(&app).terrain.clone();
    let (target, point) = (Vec2::new(2000.0, 1500.0), Vec2::new(2100.0, 1500.0));
    assert_eq!((map.height_at(target), map.height_at(point)), (3.0, 3.0));
    act(&mut app, [A::CenterOn(target)]);
    assert_eq!(
        app.world().resource::<CameraView>().center,
        Vec2::new(2000.0, 1500.0 - lift(3.0))
    );
    assert_eq!(
        app.world().resource::<CameraView>().center,
        drawn_position(&map, target)
    );
    act(
        &mut app,
        [A::Zoom {
            factor: 2.0,
            at: Some(point),
        }],
    );
    assert_eq!(
        app.world().resource::<CameraView>().center,
        Vec2::new(1900.0, 1500.0 - lift(3.0))
    );
    let nan = f32::NAN;
    act(&mut app, [A::Zoom { factor: nan, at: None }, A::CenterOn(Vec2::NAN)]);
    assert!(app.world().resource::<CameraView>().center.is_finite());
    assert!(app.world().resource::<CameraZoom>().0.is_finite());
}

/// Every map in `maps/` validates and starts, and the editor saves it unchanged, byte for byte (slice M5-P3: the
/// playtest map is made with the editor's own tools, and a map edited in the editor shows only its edits in a diff).
#[test]
fn the_maps_in_the_repository_are_playable() {
    let maps = files::list_maps(&files::default_maps_dir());
    assert!(!maps.is_empty(), "maps/ has at least dev_skirmish.json");
    let dir = temp_maps("repository");
    for path in &maps {
        let map = files::load(path).unwrap();
        assert_eq!(map.validate(), vec![], "{}", path.display());
        assert!(Sim::try_new(&Scenario::new(map.clone())).is_ok());
        let saved = dir.join(path.file_name().unwrap());
        files::save(&saved, &map).unwrap();
        let (before, after) = (std::fs::read(path).unwrap(), std::fs::read(&saved).unwrap());
        assert!(before == after, "the editor doesn't save {} unchanged", path.display());
    }
    std::fs::remove_dir_all(&dir).unwrap();
    let skirmish = files::load(&files::default_maps_dir().join("dev_skirmish.json")).unwrap();
    assert_eq!(
        skirmish,
        scenarios::dev_skirmish_map(),
        "maps/dev_skirmish.json matches the built-in map"
    );
}
