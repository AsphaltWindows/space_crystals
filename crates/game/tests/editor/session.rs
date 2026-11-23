//! The editor session: what it shows, files, play-testing, scripts, zoom, and the maps in the repository.

use bevy::prelude::*;
use sc_sim::{PlayerId, Sim, TilePresetId, scenarios};
use space_crystals::ClientMode;
use space_crystals::camera::{CameraView, CameraZoom, ViewportInsets, viewport_rect, visible_extent_su};
use space_crystals::coords::{drawn_bounds, drawn_position, lift};
use space_crystals::editor::EditorConfig;
use space_crystals::editor::actions::EditorAction as A;
use space_crystals::editor::doc::EditorDoc;
use space_crystals::editor::files;
use space_crystals::editor::state::{EditorFiles, StatusLevel};
use space_crystals::editor::ui::{EDITOR_INSETS, ShownIn};
use space_crystals::editor::view::MIN_ZOOM;
use space_crystals::hud::{HUD_INSETS, HudRoot};
use space_crystals::platform::SurfaceSize;
use space_crystals::sim_driver::SimControl;

use super::{act, at, doc, editor, editor_with, patch, sim, status, temp_maps, tile, undo_depth, visual_count};

#[test]
fn the_editor_shows_the_map_without_the_game_hud() {
    let mut app = editor(scenarios::dev_skirmish(), &temp_maps("hud"));
    assert_eq!(*app.world().resource::<ClientMode>(), ClientMode::Editor);
    assert_eq!(*app.world().resource::<ViewportInsets>(), EDITOR_INSETS);
    let mut hud = app.world_mut().query_filtered::<&Visibility, With<HudRoot>>();
    assert!(hud.iter(app.world()).all(|v| *v == Visibility::Hidden));
    // The sim shows the document, paused at tick 0.
    app.update();
    assert_eq!(sim(&app).tick(), 0);
    assert_eq!(*sim(&app).map(), scenarios::dev_skirmish().map);
    let spawns = doc(&app).spawns.len();
    assert_eq!(visual_count(&mut app), spawns);
}

#[test]
fn saving_and_opening_round_trip_the_scenario() {
    let dir = temp_maps("files");
    let mut app = editor(scenarios::dev_skirmish(), &dir);
    act(&mut app, [A::SetSeed(99), A::Save]);
    let path = dir.join("dev_skirmish.json");
    assert!(!app.world().resource::<EditorDoc>().is_dirty());
    assert_eq!(app.world().resource::<EditorDoc>().path(), Some(path.as_path()));
    let saved = doc(&app).clone();
    assert_eq!(files::load(&path), Ok(saved.clone()));
    assert_eq!(app.world().resource::<EditorFiles>().maps, vec![path.clone()]);

    // A new map; a second save goes to the same file.
    act(&mut app, [A::NewMap]);
    assert_eq!((doc(&app).map.width(), doc(&app).map.height()), files::NEW_MAP_SIZE);
    assert_eq!(app.world().resource::<EditorDoc>().path(), None);
    assert!(
        !app.world().resource::<EditorDoc>().can_undo(),
        "a new document has no history"
    );

    // Opening with unsaved changes needs a second request.
    act(&mut app, [A::SetName("scratch".into()), A::Open(path.clone())]);
    assert_eq!(doc(&app).name, "scratch");
    assert_eq!(status(&app).level, StatusLevel::Warning);
    act(&mut app, [A::Open(path.clone())]);
    assert_eq!(*doc(&app), saved);
    assert_eq!(*sim(&app).map(), saved.map);
    act(&mut app, [A::SetSeed(100), A::Save]);
    assert_eq!(files::load(&path).unwrap().seed, 100);
    assert_eq!(app.world().resource::<EditorFiles>().maps.len(), 1);

    act(&mut app, [A::Open(dir.join("missing.json"))]);
    assert_eq!(status(&app).level, StatusLevel::Error);
    assert_eq!(doc(&app).seed, 100, "a failed open keeps the document");
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn play_testing_runs_the_map_and_returns_to_the_editor() {
    let mut scenario = scenarios::empty(32, 24);
    scenario.spawns.push(patch(4, 4));
    let mut app = editor(scenario, &temp_maps("play"));
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

    act(&mut app, [A::SelectSlot(PlayerId(2)), A::PlayTest]);
    assert_eq!(*app.world().resource::<ClientMode>(), ClientMode::PlayTest);
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
    let started = sim(&app).tick();
    for _ in 0..5 {
        app.update();
    }
    assert_eq!(sim(&app).tick(), started + 5, "the game runs");
    assert_eq!(sim(&app).players().len(), 2);
    // Edits wait for the editor.
    act(&mut app, [A::Undo, A::SetSeed(3)]);
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
        .edit(|s| s.spawns.push(patch(40, 40)));
    act(&mut app, [A::PlayTest]);
    assert_eq!(*app.world().resource::<ClientMode>(), ClientMode::Editor);
    assert_eq!(status(&app).level, StatusLevel::Error);
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
    let mut app = editor(scenarios::dev_skirmish(), &temp_maps("zoom"));
    act(&mut app, [A::FitMap]);
    let zoom = app.world().resource::<CameraZoom>().0;
    let size = *app.world().resource::<SurfaceSize>();
    let viewport = viewport_rect(size, EDITOR_INSETS).1;
    let visible = visible_extent_su(viewport, size.physical.x, zoom);
    // The whole map as drawn: its raised ground reaches above its northern edge.
    let drawn = drawn_bounds(&doc(&app).map);
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
    let map = doc(&app).map.clone();
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

#[test]
fn the_maps_in_the_repository_are_playable() {
    let maps = files::list_maps(&files::default_maps_dir());
    assert!(!maps.is_empty(), "maps/ has at least dev_skirmish.json");
    for path in &maps {
        let scenario = files::load(path).unwrap();
        assert_eq!(scenario.validate(), vec![], "{}", path.display());
        assert!(Sim::try_new(&scenario).is_ok());
    }
    let skirmish = files::load(&files::default_maps_dir().join("dev_skirmish.json")).unwrap();
    assert_eq!(
        skirmish,
        scenarios::dev_skirmish(),
        "maps/dev_skirmish.json matches the built-in scenario"
    );
}
