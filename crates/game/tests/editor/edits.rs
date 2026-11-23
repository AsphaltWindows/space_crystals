//! Edits: terrain, heights, objects and their values, symmetry, undo and redo, resizing, player slots and
//! validation problems.

use bevy::prelude::*;
use sc_sim::{
    CornerPos, Faction, ObjectKind, PlayerId, ResourceState, ScenarioProblem, Spawn, SpawnAt, SpawnProblem,
    SpawnSettings, Symmetry, TICKS_PER_SECOND, Tile, TilePos, TilePreset, TilePresetId, scenarios,
};
use space_crystals::camera::CameraView;
use space_crystals::editor::actions::EditorAction as A;
use space_crystals::editor::doc::EditorDoc;
use space_crystals::editor::files;
use space_crystals::editor::overlay::Ghost;
use space_crystals::editor::state::{EditorProblems, EditorSelection, Selection, StatusLevel};
use space_crystals::editor::tools::{ElevationMode, Tool};
use space_crystals::editor::ui::fields::TextField;
use space_crystals::map_render::MapView;

use super::{act, at, doc, editor, patch, sim, status, temp_maps, tile, tools, undo_depth, visual_count};

fn corner(app: &App, x: i32, y: i32) -> u8 {
    doc(app).map.corner(CornerPos::new(x, y)).unwrap()
}

fn selection(app: &App) -> Option<Selection> {
    app.world().resource::<EditorSelection>().0
}

fn problems(app: &App) -> Vec<ScenarioProblem> {
    app.world().resource::<EditorProblems>().0.clone()
}

/// The ground point that snaps a 2 × 2 footprint's top-left tile to (`x`, `y`).
fn station_at(x: i32, y: i32) -> Vec2 {
    TilePos::new(x + 1, y + 1).corner()
}

#[test]
fn painting_heights_and_fill_edit_the_terrain() {
    let mut app = editor(scenarios::empty(16, 12), &temp_maps("paint"));
    let builds = app.world().resource::<MapView>().builds;

    // One stroke of a 3×3 brush from tile (5, 5) to (8, 5) is one undo step.
    act(
        &mut app,
        [
            A::SetPreset(TilePresetId::WATER),
            A::SetBrush(3),
            A::Press(at(5, 5)),
            A::Drag(at(8, 5)),
            A::Release,
        ],
    );
    assert_eq!(tools(&app).tool, Tool::Paint);
    for x in 4..=9 {
        for y in 4..=6 {
            assert_eq!(tile(&app, x, y).preset, TilePresetId::WATER, "({x}, {y})");
        }
    }
    assert_eq!(tile(&app, 3, 5).preset, TilePresetId::PLANE);
    assert_eq!(tile(&app, 6, 7).preset, TilePresetId::PLANE);
    assert_eq!(undo_depth(&app), 1);
    // The sim and the map mesh caught up.
    assert_eq!(*sim(&app).map(), doc(&app).map);
    assert!(app.world().resource::<MapView>().builds > builds);

    // Setting a level moves the brush's corners; the slopes around them follow, one level per tile.
    act(
        &mut app,
        [A::SetElevation(3), A::SetBrush(1), A::Press(at(1, 1)), A::Release],
    );
    assert_eq!(tools(&app).tool, Tool::Elevation);
    assert_eq!(doc(&app).map.tile_level(TilePos::new(1, 1)), Some(3));
    assert_eq!((corner(&app, 0, 0), corner(&app, 4, 1), corner(&app, 5, 1)), (2, 1, 0));
    assert_eq!(doc(&app).map.too_steep_tiles(), vec![]);
    assert_eq!(tile(&app, 1, 1).preset, TilePresetId::PLANE, "heights keep the presets");

    // A raise stroke raises each corner once, however often the brush passes over it.
    act(
        &mut app,
        [
            A::SetElevationMode(ElevationMode::Raise),
            A::Press(at(1, 1)),
            A::Drag(at(2, 1)),
            A::Drag(at(1, 1)),
            A::Release,
        ],
    );
    assert_eq!(
        (corner(&app, 1, 1), corner(&app, 2, 1), corner(&app, 3, 1)),
        (4, 4, 4),
        "tile (2, 1)'s eastern corners rose from the slope's 3"
    );
    act(
        &mut app,
        [
            A::SetElevationMode(ElevationMode::Lower),
            A::Press(at(2, 1)),
            A::Release,
        ],
    );
    assert_eq!(corner(&app, 2, 1), 3);
    assert_eq!(
        corner(&app, 1, 1),
        4,
        "corners outside the brush only move when a slope needs it"
    );
    assert_eq!(doc(&app).map.too_steep_tiles(), vec![]);
    act(&mut app, [A::StepElevation(99)]);
    assert_eq!(tools(&app).elevation, 16, "16 is the top level");

    // Fill: the connected plane becomes rugged, hill included; the water stays, and so do the heights.
    act(
        &mut app,
        [
            A::SetTool(Tool::Fill),
            A::SetPreset(TilePresetId::RUGGED),
            A::Press(at(0, 11)),
            A::Release,
        ],
    );
    assert_eq!(tools(&app).tool, Tool::Fill, "choosing a preset keeps the Fill tool");
    assert_eq!(tile(&app, 15, 0), Tile::new(TilePresetId::RUGGED));
    assert_eq!(tile(&app, 1, 1), Tile::new(TilePresetId::RUGGED));
    assert_eq!(tile(&app, 6, 5).preset, TilePresetId::WATER);
    assert_eq!(corner(&app, 1, 1), 4);

    // Symmetry repeats strokes at the images.
    act(
        &mut app,
        [
            A::SetTool(Tool::Paint),
            A::SetSymmetry(Symmetry::MirrorVerticalAxis),
            A::SetPreset(TilePresetId::MOUNTAIN),
            A::Press(at(0, 0)),
            A::Release,
        ],
    );
    assert_eq!(tile(&app, 0, 0).preset, TilePresetId::MOUNTAIN);
    assert_eq!(tile(&app, 15, 0).preset, TilePresetId::MOUNTAIN);
    assert_eq!(tile(&app, 1, 0).preset, TilePresetId::RUGGED);
    act(
        &mut app,
        [A::SetSymmetry(Symmetry::Rotate180), A::Press(at(2, 2)), A::Release],
    );
    assert_eq!(tile(&app, 13, 9).preset, TilePresetId::MOUNTAIN);

    // The eyedropper picks a tile's preset and the height at the point, rounded.
    act(&mut app, [A::Pick(at(6, 5))]);
    assert_eq!((tools(&app).preset, tools(&app).elevation), (TilePresetId::WATER, 0));
    act(&mut app, [A::Pick(Vec2::new(64.0, 64.0))]);
    assert_eq!((tools(&app).preset, tools(&app).elevation), (TilePresetId::RUGGED, 4));
}

/// `core/map.md > Elevation > Maximum slope`: slopes stop at non-Traversible tiles, and a Traversible preset is
/// refused, with a warning, on tiles steeper than 1 level.
#[test]
fn slopes_stop_at_cliffs_and_traversible_presets_need_gentle_ground() {
    let mut scenario = scenarios::empty(16, 12);
    for y in 0..12 {
        scenario
            .map
            .set_tile(TilePos::new(6, y), Tile::new(TilePresetId::CLIFF));
    }
    let mut app = editor(scenario, &temp_maps("cliffs"));
    act(
        &mut app,
        [A::SetElevation(5), A::SetBrush(1), A::Press(at(5, 5)), A::Release],
    );
    // Tile (5, 5) stands at 5; the slope runs down westwards, while the cliff column takes the whole drop to the
    // east, where the plane stays flat.
    assert_eq!(doc(&app).map.tile_level(TilePos::new(5, 5)), Some(5));
    assert_eq!(
        (0..=5).map(|x| corner(&app, x, 5)).collect::<Vec<_>>(),
        [0, 1, 2, 3, 4, 5]
    );
    assert!((7..=16).all(|x| corner(&app, x, 5) == 0));
    assert_eq!(doc(&app).map.tile_slope(TilePos::new(6, 5)), Some(5));
    assert_eq!(problems(&app), vec![], "a cliff may be as steep as it likes");

    // Painting Plane down the cliff column: the tiles at most 1 level steep take it, the others are left alone.
    act(
        &mut app,
        [
            A::SetPreset(TilePresetId::PLANE),
            A::Press(at(6, 0)),
            A::Drag(at(6, 11)),
            A::Release,
        ],
    );
    let column: Vec<TilePresetId> = (0..12).map(|y| tile(&app, 6, y).preset).collect();
    let (p, c) = (TilePresetId::PLANE, TilePresetId::CLIFF);
    assert_eq!(column, [p, c, c, c, c, c, c, c, c, c, p, p]);
    assert_eq!(status(&app).level, StatusLevel::Warning);
    assert!(
        status(&app).message.contains("left 9 tile(s)"),
        "{}",
        status(&app).message
    );
    assert_eq!(doc(&app).map.too_steep_tiles(), vec![]);
}

/// Design 0013: crystal patches and Supply Delivery Stations need flat ground. Placing one on a slope is refused
/// (its ghost turns red); a height edit that slopes the ground under one is allowed, and flagged.
#[test]
fn resources_need_flat_ground() {
    let mut app = editor(scenarios::empty(16, 12), &temp_maps("flat"));
    let station = ObjectKind::SupplyDeliveryStation;
    act(
        &mut app,
        [
            A::SetKind(ObjectKind::CrystalPatch),
            A::Press(at(10, 8)),
            A::Release,
            A::SetElevation(2),
            A::SetBrush(1),
            A::Press(at(3, 3)),
            A::Release,
        ],
    );
    assert_eq!(doc(&app).spawns.len(), 1);
    // Tile (3, 3) is flat at 2, its neighbours slope.
    act(&mut app, [A::SetKind(station), A::Hover(Some(station_at(2, 3)))]);
    let mut ghosts = app
        .world_mut()
        .query::<(&Ghost, &MeshMaterial2d<ColorMaterial>, &Visibility)>();
    let (_, material, visibility) = ghosts.iter(app.world()).find(|(g, ..)| g.image == 0).unwrap();
    assert_eq!(*visibility, Visibility::Inherited);
    let color = app
        .world()
        .resource::<Assets<ColorMaterial>>()
        .get(&material.0)
        .unwrap()
        .color
        .to_srgba();
    assert!(color.red > 2.0 * color.green, "the ghost is red: {color:?}");
    act(&mut app, [A::Press(station_at(2, 3)), A::Release]);
    assert_eq!(doc(&app).spawns.len(), 1, "refused");
    assert_eq!(status(&app).level, StatusLevel::Warning);
    assert!(status(&app).message.contains("flat ground"), "{}", status(&app).message);
    // A flat plateau takes it.
    act(
        &mut app,
        [
            A::SetElevation(2),
            A::SetBrush(3),
            A::Press(at(12, 3)),
            A::Release,
            A::SetKind(station),
            A::Press(station_at(12, 3)),
            A::Release,
        ],
    );
    assert_eq!(doc(&app).spawns.len(), 2);

    // Raising the tile east of the patch slopes the patch's tile: allowed, and listed as a problem.
    act(
        &mut app,
        [
            A::SetElevationMode(ElevationMode::Raise),
            A::SetBrush(1),
            A::Press(at(11, 8)),
            A::Release,
        ],
    );
    assert_eq!(corner(&app, 11, 8), 1);
    assert!(problems(&app).contains(&ScenarioProblem::InvalidSpawn {
        spawn: 0,
        kind: ObjectKind::CrystalPatch,
        problem: SpawnProblem::NotFlat(TilePos::new(10, 8)),
    }));
    assert_eq!(
        visual_count(&mut app),
        2,
        "the editor still shows it, so it can be fixed"
    );
}

#[test]
fn objects_are_placed_moved_and_deleted() {
    let mut app = editor(scenarios::empty(16, 12), &temp_maps("objects"));
    let station = ObjectKind::SupplyDeliveryStation;

    // The ghost follows the cursor; the station's 2×2 footprint centres on the nearest grid point.
    act(&mut app, [A::SetKind(station), A::Hover(Some(Vec2::new(320.0, 320.0)))]);
    assert_eq!(tools(&app).tool, Tool::Place);
    let mut ghosts = app.world_mut().query::<(&Ghost, &Visibility, &Transform)>();
    let (_, visibility, transform) = ghosts.iter(app.world()).find(|(g, ..)| g.image == 0).unwrap();
    assert_eq!(*visibility, Visibility::Inherited);
    assert_eq!(transform.translation.truncate(), Vec2::new(320.0, -320.0));

    act(&mut app, [A::Press(Vec2::new(320.0, 320.0)), A::Release]);
    assert_eq!(
        doc(&app).spawns,
        vec![Spawn::new(station, None, SpawnAt::Tile(TilePos::new(4, 4)))]
    );

    // Overlapping and off-map spots are refused with a warning.
    act(&mut app, [A::Press(Vec2::new(384.0, 384.0)), A::Release]);
    assert_eq!(doc(&app).spawns.len(), 1);
    assert_eq!(status(&app).level, StatusLevel::Warning);
    act(&mut app, [A::Press(Vec2::new(16.0 * 64.0 - 10.0, 300.0)), A::Release]);
    assert_eq!(doc(&app).spawns.len(), 1);

    // Resources are always neutral; a drag places a row of patches as one undo step.
    let depth = undo_depth(&app);
    act(
        &mut app,
        [
            A::SetOwner(Some(PlayerId(1))),
            A::SetKind(ObjectKind::CrystalPatch),
            A::Press(at(1, 1)),
            A::Drag(at(2, 1)),
            A::Drag(at(3, 1)),
            A::Release,
        ],
    );
    let patches: Vec<&Spawn> = doc(&app)
        .spawns
        .iter()
        .filter(|s| s.kind == ObjectKind::CrystalPatch)
        .collect();
    assert_eq!(patches.len(), 3);
    assert!(patches.iter().all(|s| s.owner.is_none()));
    assert_eq!(undo_depth(&app), depth + 1);
    assert_eq!(visual_count(&mut app), 4);

    // Select the station and drag it elsewhere.
    act(
        &mut app,
        [
            A::SetTool(Tool::Select),
            A::Press(Vec2::new(330.0, 310.0)),
            A::Drag(Vec2::new(650.0, 438.0)),
            A::Release,
        ],
    );
    assert_eq!(selection(&app), Some(Selection::Spawn(0)));
    assert_eq!(doc(&app).spawns[0].at, SpawnAt::Tile(TilePos::new(9, 6)));
    let station_object = sim(&app).objects().find(|o| o.kind == station).unwrap().pos;
    assert_eq!(station_object, Vec2::new(640.0, 448.0), "the sim shows the move");

    act(&mut app, [A::DeleteSelected]);
    assert!(doc(&app).spawns.iter().all(|s| s.kind != station));
    assert_eq!(selection(&app), None);
    assert_eq!(visual_count(&mut app), 3);

    // Mirrored placement.
    act(
        &mut app,
        [
            A::SetSymmetry(Symmetry::MirrorHorizontalAxis),
            A::SetKind(ObjectKind::CrystalPatch),
            A::Press(at(7, 0)),
            A::Release,
        ],
    );
    let spots: Vec<SpawnAt> = doc(&app).spawns.iter().map(|s| s.at).collect();
    assert!(spots.contains(&SpawnAt::Tile(TilePos::new(7, 0))) && spots.contains(&SpawnAt::Tile(TilePos::new(7, 11))));
}

/// `core/map_editor.md > Per-object Resource Values`: new objects get the defaults; the values are edited as one
/// undo step each, checked, shown by the sim, and saved.
#[test]
fn per_object_values_are_edited_undone_and_saved() {
    let dir = temp_maps("values");
    let mut app = editor(scenarios::empty(16, 12), &dir);
    let station = ObjectKind::SupplyDeliveryStation;
    act(&mut app, [A::SetKind(station), A::Press(station_at(4, 4)), A::Release]);
    assert_eq!(doc(&app).spawns[0].settings, SpawnSettings::defaults(station));

    // Selecting it shows its three values.
    act(
        &mut app,
        [A::SetTool(Tool::Select), A::Press(station_at(4, 4)), A::Release],
    );
    app.update();
    let mut fields = app.world_mut().query::<&TextField>();
    let shown = fields
        .iter(app.world())
        .filter(|f| matches!(f, TextField::SpawnValue { spawn: 0, .. }))
        .count();
    assert_eq!(shown, 3);

    let values = SpawnSettings::SupplyDeliveryStation {
        starting_supplies: 12,
        delivery_size: 6,
        delivery_interval: 30 * TICKS_PER_SECOND,
    };
    let depth = undo_depth(&app);
    act(
        &mut app,
        [A::SetSpawnSettings {
            spawn: 0,
            settings: values,
        }],
    );
    assert_eq!(doc(&app).spawns[0].settings, Some(values));
    assert_eq!(undo_depth(&app), depth + 1);
    let resource = sim(&app).objects().next().unwrap().resource.clone();
    assert!(
        matches!(
            resource,
            Some(ResourceState::SupplyStation {
                current_supplies: 12,
                delivery_size: 6,
                ..
            })
        ),
        "{resource:?}"
    );

    // Values that make no sense, or that belong to another kind, are refused.
    let empty_deliveries = SpawnSettings::SupplyDeliveryStation {
        starting_supplies: 12,
        delivery_size: 0,
        delivery_interval: 30 * TICKS_PER_SECOND,
    };
    for (spawn, settings) in [
        (0, empty_deliveries),
        (0, SpawnSettings::CrystalPatch { starting_amount: 5 }),
        (7, values),
    ] {
        act(&mut app, [A::SetSpawnSettings { spawn, settings }]);
        assert_eq!(status(&app).level, StatusLevel::Warning);
        assert_eq!(doc(&app).spawns[0].settings, Some(values));
    }

    act(&mut app, [A::Undo]);
    assert_eq!(doc(&app).spawns[0].settings, SpawnSettings::defaults(station));
    act(&mut app, [A::Redo]);
    assert_eq!(doc(&app).spawns[0].settings, Some(values));

    // They survive saving and opening.
    act(&mut app, [A::Save]);
    let path = app.world().resource::<EditorDoc>().path().unwrap().to_path_buf();
    assert_eq!(files::load(&path).unwrap().spawns[0].settings, Some(values));
    act(&mut app, [A::NewMap, A::Open(path)]);
    assert_eq!(doc(&app).spawns[0].settings, Some(values));
    std::fs::remove_dir_all(&dir).unwrap();
}

/// `core/map_editor.md > Symmetry`: strokes and placements repeat at every image of the mode; 90° rotation needs a
/// square map.
#[test]
fn symmetry_repeats_edits_at_every_image() {
    let mut app = editor(scenarios::empty(12, 12), &temp_maps("symmetry"));
    let water = TilePresetId::WATER;
    act(
        &mut app,
        [
            A::SetSymmetry(Symmetry::Rotate90),
            A::SetPreset(water),
            A::SetBrush(1),
            A::Press(at(1, 2)),
            A::Release,
        ],
    );
    assert_eq!(tools(&app).symmetry, Symmetry::Rotate90);
    for (x, y) in [(1, 2), (9, 1), (10, 9), (2, 10)] {
        assert_eq!(tile(&app, x, y).preset, water, "({x}, {y})");
    }
    let depth = undo_depth(&app);
    act(
        &mut app,
        [
            A::SetKind(ObjectKind::SupplyDeliveryStation),
            A::Press(station_at(3, 4)),
            A::Release,
        ],
    );
    let spots: Vec<SpawnAt> = doc(&app).spawns.iter().map(|s| s.at).collect();
    let expected: Vec<SpawnAt> = [(3, 4), (6, 3), (7, 6), (4, 7)]
        .map(|(x, y)| SpawnAt::Tile(TilePos::new(x, y)))
        .into();
    assert_eq!(spots, expected);
    assert!(
        doc(&app).spawns.iter().all(|s| s.owner.is_none()),
        "resources stay neutral"
    );
    act(&mut app, [A::Undo]);
    assert!(doc(&app).spawns.is_empty(), "the images are part of the same undo step");
    assert_eq!(undo_depth(&app), depth);

    // The other modes, each with its own preset.
    let cases = [
        (
            Symmetry::MirrorBothAxes,
            TilePresetId::MOUNTAIN,
            (0, 5),
            vec![(11, 5), (0, 6), (11, 6)],
        ),
        (Symmetry::Rotate180, TilePresetId::CLIFF, (5, 0), vec![(6, 11)]),
        (
            Symmetry::MirrorHorizontalAxis,
            TilePresetId::RUGGED,
            (3, 0),
            vec![(3, 11)],
        ),
        (
            Symmetry::MirrorVerticalAxis,
            TilePresetId::RUGGED,
            (0, 3),
            vec![(11, 3)],
        ),
        (Symmetry::None, TilePresetId::MOUNTAIN, (6, 6), vec![]),
    ];
    for (symmetry, preset, (x, y), images) in cases {
        act(
            &mut app,
            [
                A::SetSymmetry(symmetry),
                A::SetPreset(preset),
                A::Press(at(x, y)),
                A::Release,
            ],
        );
        for (ix, iy) in std::iter::once((x, y)).chain(images) {
            assert_eq!(tile(&app, ix, iy).preset, preset, "{symmetry:?}: ({ix}, {iy})");
        }
    }
    assert_eq!(tile(&app, 5, 5).preset, TilePresetId::PLANE, "nothing else changed");

    // 90° rotation is refused on a map that isn't square, and switched off when the map stops being square.
    act(
        &mut app,
        [A::SetSymmetry(Symmetry::Rotate90), A::Resize { width: 12, height: 10 }],
    );
    assert_eq!(tools(&app).symmetry, Symmetry::None);
    assert_eq!(status(&app).level, StatusLevel::Warning);
    act(&mut app, [A::SetSymmetry(Symmetry::Rotate90)]);
    assert_eq!(tools(&app).symmetry, Symmetry::None);
    assert!(status(&app).message.contains("square"), "{}", status(&app).message);
}

#[test]
fn undo_and_redo_step_through_whole_actions() {
    let mut app = editor(scenarios::empty(16, 12), &temp_maps("undo"));
    let original = doc(&app).clone();
    act(
        &mut app,
        [
            A::SetPreset(TilePresetId::WATER),
            A::Press(at(3, 3)),
            A::Drag(at(6, 3)),
            A::Release,
        ],
    );
    let painted = doc(&app).clone();
    act(
        &mut app,
        [A::SetKind(ObjectKind::CrystalPatch), A::Press(at(10, 10)), A::Release],
    );
    let placed = doc(&app).clone();
    act(&mut app, [A::SetName("Undo test".into()), A::SetSeed(77)]);
    assert_eq!(undo_depth(&app), 4);

    act(&mut app, [A::Undo, A::Undo]);
    assert_eq!(*doc(&app), placed);
    act(&mut app, [A::Undo]);
    assert_eq!(*doc(&app), painted);
    assert_eq!(sim(&app).objects().count(), 0, "the sim follows the undo");
    act(&mut app, [A::Undo, A::Undo]);
    assert_eq!(*doc(&app), original);
    assert!(!app.world().resource::<EditorDoc>().is_dirty());
    assert_eq!(*sim(&app).map(), original.map);

    act(&mut app, [A::Redo, A::Redo]);
    assert_eq!(*doc(&app), placed);
    // A new edit drops what could be redone.
    act(&mut app, [A::SetSeed(5), A::Redo]);
    assert_eq!(doc(&app).seed, 5);
    assert_eq!(doc(&app).name, placed.name);
    // Cancelling a stroke restores the map without an undo step.
    let depth = undo_depth(&app);
    act(
        &mut app,
        [A::SetTool(Tool::Paint), A::Press(at(12, 3)), A::Cancel, A::Release],
    );
    assert_eq!(doc(&app).map, placed.map);
    assert_eq!(undo_depth(&app), depth);
}

/// Review finding: a selection is a spawn index, which an undo, a redo or a resize may shift onto another spawn.
/// They drop the selection, so Delete can't remove the wrong object.
#[test]
fn undo_redo_and_resize_drop_the_selection() {
    let mut scenario = scenarios::empty(16, 12);
    scenario.spawns = vec![patch(1, 1), patch(3, 1), patch(5, 1)];
    let mut app = editor(scenario, &temp_maps("stale"));
    act(
        &mut app,
        [
            A::SetTool(Tool::Select),
            A::Press(at(1, 1)),
            A::Release,
            A::DeleteSelected,
        ],
    );
    assert_eq!(doc(&app).spawns, vec![patch(3, 1), patch(5, 1)]);
    act(&mut app, [A::Press(at(5, 1)), A::Release]);
    assert_eq!(selection(&app), Some(Selection::Spawn(1)));
    // The undo brings the first patch back as #0, which makes #1 the middle one.
    act(&mut app, [A::Undo]);
    assert_eq!(selection(&app), None);
    act(&mut app, [A::DeleteSelected]);
    assert_eq!(doc(&app).spawns.len(), 3, "nothing was deleted");

    act(&mut app, [A::Press(at(5, 1)), A::Release, A::Redo]);
    assert_eq!(selection(&app), None);
    act(&mut app, [A::Press(at(5, 1)), A::Release]);
    assert_eq!(selection(&app), Some(Selection::Spawn(1)));
    act(&mut app, [A::Resize { width: 15, height: 12 }]);
    assert_eq!(selection(&app), None);
}

/// Review finding: a damaged file's spawns at the ends of the `i32` range must not overflow anywhere: loading,
/// validation, placing next to them, selecting, focusing their problems and drawing the overlay.
#[test]
fn spawns_at_the_ends_of_the_coordinate_range_do_no_harm() {
    let station = ObjectKind::SupplyDeliveryStation;
    let mut scenario = scenarios::empty(16, 12);
    scenario.spawns = vec![
        Spawn::new(station, None, SpawnAt::Tile(TilePos::new(i32::MAX, i32::MAX))),
        Spawn::new(station, None, SpawnAt::Tile(TilePos::new(i32::MIN, 3))),
        patch(i32::MAX, 0),
    ];
    let mut app = editor(scenario, &temp_maps("overflow"));
    assert_eq!(problems(&app).len(), 3, "{:?}", problems(&app));
    act(
        &mut app,
        [
            A::SetKind(station),
            A::Hover(Some(station_at(4, 4))),
            A::Press(station_at(4, 4)),
            A::Release,
        ],
    );
    assert_eq!(doc(&app).spawns.len(), 4);
    for i in 0..3 {
        act(&mut app, [A::FocusProblem(i)]);
        assert!(app.world().resource::<CameraView>().center.is_finite());
    }
    act(
        &mut app,
        [
            A::SetSymmetry(Symmetry::MirrorBothAxes),
            A::SetTool(Tool::Select),
            A::Press(at(8, 8)),
            A::Release,
            A::SetTool(Tool::Place),
            A::Hover(Some(at(15, 11))),
        ],
    );
}

/// Review finding: the tools may hold a custom preset of the map before; a map without it (opened or new) resets
/// the choice instead of panicking on the status line.
#[test]
fn a_custom_preset_choice_is_reset_by_a_map_without_it() {
    let dir = temp_maps("custom_preset");
    let mut custom_map = scenarios::empty(16, 12);
    let ash = custom_map.map.add_preset(TilePreset {
        name: "Ash".into(),
        buildable: true,
        traversible: true,
        rugged: false,
        drillable: false,
        recruitable: false,
    });
    let (custom, plain) = (dir.join("custom.json"), dir.join("plain.json"));
    files::save(&custom, &custom_map).unwrap();
    files::save(&plain, &scenarios::empty(8, 8)).unwrap();
    let mut app = editor(custom_map, &dir);
    act(&mut app, [A::SetPreset(ash), A::Hover(Some(at(2, 2)))]);
    assert_eq!(tools(&app).preset, ash);

    act(&mut app, [A::Open(plain), A::Hover(Some(at(3, 3)))]);
    assert_eq!(tools(&app).preset, TilePresetId::PLANE);
    app.update();

    act(
        &mut app,
        [A::Open(custom), A::SetPreset(ash), A::NewMap, A::Hover(Some(at(2, 2)))],
    );
    assert_eq!(tools(&app).preset, TilePresetId::PLANE);
    app.update();
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn resizing_reports_what_falls_outside_and_undo_restores_it() {
    let mut scenario = scenarios::empty(16, 12);
    scenario.spawns = vec![patch(2, 2), patch(14, 10)];
    let mut app = editor(scenario.clone(), &temp_maps("resize"));
    act(&mut app, [A::Resize { width: 10, height: 8 }]);
    assert_eq!((doc(&app).map.width(), doc(&app).map.height()), (10, 8));
    assert_eq!(doc(&app).spawns, vec![patch(2, 2)]);
    assert_eq!(
        doc(&app).players[1].start_location,
        None,
        "P2's start location (11, 8) fell outside"
    );
    assert_eq!(status(&app).level, StatusLevel::Warning);
    assert!(status(&app).message.contains("removed #1"), "{}", status(&app).message);
    assert!(
        status(&app).message.contains("P2's start location"),
        "{}",
        status(&app).message
    );
    assert_eq!(sim(&app).map().width(), 10);

    act(&mut app, [A::Undo]);
    assert_eq!(*doc(&app), scenario);
    act(&mut app, [A::Resize { width: 0, height: 8 }]);
    assert_eq!(*doc(&app), scenario, "sizes start at 1");
    act(&mut app, [A::Resize { width: 300, height: 8 }]);
    assert_eq!(*doc(&app), scenario, "sides end at the sim's largest");
    act(&mut app, [A::Resize { width: 20, height: 14 }]);
    assert_eq!(
        doc(&app).map.tile(TilePos::new(19, 13)),
        Some(Tile::new(TilePresetId::PLANE))
    );
    assert_eq!(doc(&app).map.corner(CornerPos::new(20, 14)), Some(0));
    assert_eq!(doc(&app).spawns, scenario.spawns);
}

#[test]
fn player_slots_stay_numbered_from_one() {
    let mut app = editor(scenarios::empty(16, 12), &temp_maps("slots"));
    let ids = |app: &App| doc(app).players.iter().map(|p| p.id.0).collect::<Vec<_>>();
    act(&mut app, [A::AddSlot, A::AddSlot]);
    assert_eq!(ids(&app), vec![1, 2, 3, 4]);
    assert_eq!(tools(&app).slot, PlayerId(4));
    act(
        &mut app,
        [
            A::SetSlotName(PlayerId(3), "Third".into()),
            A::SetSlotFaction(PlayerId(3), Faction::Cults),
        ],
    );

    act(&mut app, [A::RemoveSlot(PlayerId(2))]);
    assert_eq!(ids(&app), vec![1, 2, 3]);
    assert_eq!(
        (doc(&app).players[1].name.as_str(), doc(&app).players[1].faction),
        ("Third", Faction::Cults)
    );
    assert_eq!(
        tools(&app).slot,
        PlayerId(3),
        "the chosen slot moved down with its number"
    );
    act(
        &mut app,
        [
            A::RemoveSlot(PlayerId(3)),
            A::RemoveSlot(PlayerId(2)),
            A::RemoveSlot(PlayerId(1)),
        ],
    );
    assert_eq!(ids(&app), vec![1], "a map keeps one slot");

    // Markers: place the start location and the camera start, then select and clear the camera start.
    act(
        &mut app,
        [
            A::SelectSlot(PlayerId(1)),
            A::SetTool(Tool::StartLocation),
            A::Press(at(3, 3)),
            A::Drag(at(5, 4)),
            A::Release,
        ],
    );
    assert_eq!(doc(&app).players[0].start_location, Some(TilePos::new(5, 4)));
    act(
        &mut app,
        [
            A::SetTool(Tool::CameraStart),
            A::Press(Vec2::new(500.0, 300.0)),
            A::Release,
        ],
    );
    assert_eq!(doc(&app).players[0].camera_start, Some(Vec2::new(500.0, 300.0)));
    act(
        &mut app,
        [A::SetTool(Tool::Select), A::Press(Vec2::new(510.0, 290.0)), A::Release],
    );
    assert_eq!(selection(&app), Some(Selection::CameraStart(PlayerId(1))));
    act(&mut app, [A::DeleteSelected]);
    assert_eq!(doc(&app).players[0].camera_start, None);
    act(&mut app, [A::SetCameraStart(PlayerId(1), Some(Vec2::new(-5.0, 0.0)))]);
    assert_eq!(doc(&app).players[0].camera_start, None, "camera starts stay on the map");
}

#[test]
fn validation_problems_are_listed_and_lead_to_their_spot() {
    let mut scenario = scenarios::empty(64, 48);
    scenario.spawns = vec![patch(2, 2), patch(20, 15), patch(20, 15), patch(70, 3)];
    let dir = temp_maps("problems");
    let mut app = editor(scenario, &dir);
    assert_eq!(problems(&app).len(), 2, "{:?}", problems(&app));
    // The preview shows every spawn that has a place, overlaps included, so it can be fixed.
    assert_eq!(sim(&app).objects().count(), 3);

    act(&mut app, [A::FocusProblem(0)]);
    assert_eq!(app.world().resource::<CameraView>().center, at(20, 15));
    assert_eq!(selection(&app), Some(Selection::Spawn(2)));
    act(&mut app, [A::DeleteSelected]);
    assert_eq!(problems(&app).len(), 1);

    // Saving an invalid map works, with a warning.
    act(&mut app, [A::Save]);
    assert_eq!(status(&app).level, StatusLevel::Warning);
    act(&mut app, [A::FocusProblem(0), A::DeleteSelected]);
    assert!(problems(&app).is_empty());
    std::fs::remove_dir_all(&dir).unwrap();
}
