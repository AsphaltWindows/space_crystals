//! Edits of a map without symmetry: terrain, heights, objects and their values, undo and redo, resizing, player
//! slots and their markers, and validation problems.

use bevy::prelude::*;
use sc_sim::{
    ObjectKind, ResourceState, ScenarioProblem, SlotId, Spawn, SpawnAt, SpawnProblem, SpawnSettings, TICKS_PER_SECOND,
    Tile, TilePos, TilePresetId, scenarios,
};
use space_crystals::camera::CameraView;
use space_crystals::editor::actions::EditorAction as A;
use space_crystals::editor::doc::EditorDoc;
use space_crystals::editor::files;
use space_crystals::editor::overlay::Ghost;
use space_crystals::editor::state::{Selection, StatusLevel};
use space_crystals::editor::tools::{ElevationMode, Tool};
use space_crystals::editor::ui::fields::TextField;
use space_crystals::map_render::MapView;

use super::{
    act, assert_warned, at, corner, doc, editor, patch, problems, selection, sim, station_at, status, temp_maps, tile,
    tools, undo_depth, visual_count,
};

#[test]
fn painting_heights_and_fill_edit_the_terrain() {
    let mut app = editor(scenarios::empty_map(16, 12), &temp_maps("paint"));
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
    assert_eq!(*sim(&app).map(), doc(&app).terrain);
    assert!(app.world().resource::<MapView>().builds > builds);

    // Setting a level moves the brush's corners; the slopes around them follow, one level per tile.
    act(
        &mut app,
        [A::SetElevation(3), A::SetBrush(1), A::Press(at(1, 1)), A::Release],
    );
    assert_eq!(tools(&app).tool, Tool::Elevation);
    assert_eq!(doc(&app).terrain.tile_level(TilePos::new(1, 1)), Some(3));
    assert_eq!((corner(&app, 0, 0), corner(&app, 4, 1), corner(&app, 5, 1)), (2, 1, 0));
    assert_eq!(doc(&app).terrain.too_steep_tiles(), vec![]);
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
    assert_eq!(doc(&app).terrain.too_steep_tiles(), vec![]);
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
    let mut map = scenarios::empty_map(16, 12);
    for y in 0..12 {
        map.terrain.set_tile(TilePos::new(6, y), Tile::new(TilePresetId::CLIFF));
    }
    let mut app = editor(map, &temp_maps("cliffs"));
    act(
        &mut app,
        [A::SetElevation(5), A::SetBrush(1), A::Press(at(5, 5)), A::Release],
    );
    // Tile (5, 5) stands at 5; the slope runs down westwards, while the cliff column takes the whole drop to the
    // east, where the plane stays flat.
    assert_eq!(doc(&app).terrain.tile_level(TilePos::new(5, 5)), Some(5));
    assert_eq!(
        (0..=5).map(|x| corner(&app, x, 5)).collect::<Vec<_>>(),
        [0, 1, 2, 3, 4, 5]
    );
    assert!((7..=16).all(|x| corner(&app, x, 5) == 0));
    assert_eq!(doc(&app).terrain.tile_slope(TilePos::new(6, 5)), Some(5));
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
    assert_warned(&app, "left 9 tile(s)");
    assert_eq!(doc(&app).terrain.too_steep_tiles(), vec![]);
}

/// Design 0013: crystal patches and Supply Delivery Stations need flat ground. Placing one on a slope is refused
/// (its ghost turns red); a height edit that slopes the ground under one is allowed, and flagged.
#[test]
fn resources_need_flat_ground() {
    let mut app = editor(scenarios::empty_map(16, 12), &temp_maps("flat"));
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
    assert_warned(&app, "flat ground");
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
    let mut app = editor(scenarios::empty_map(16, 12), &temp_maps("objects"));
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
            A::SetOwner(Some(SlotId(1))),
            A::SetKind(ObjectKind::CrystalPatch),
            A::Press(at(1, 1)),
            A::Drag(at(2, 1)),
            A::Drag(at(3, 1)),
            A::Release,
        ],
    );
    assert_eq!(tools(&app).owner, Some(SlotId(1)), "a Custom map takes owners");
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
}

/// `core/map_editor.md > Per-object Resource Values`: new objects get the defaults; the values are edited as one
/// undo step each, checked, shown by the sim, and saved.
#[test]
fn per_object_values_are_edited_undone_and_saved() {
    let dir = temp_maps("values");
    let mut app = editor(scenarios::empty_map(16, 12), &dir);
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
    act(&mut app, [A::NewMap, A::CreateMap, A::Open(path)]);
    assert_eq!(doc(&app).spawns[0].settings, Some(values));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn undo_and_redo_step_through_whole_actions() {
    let mut app = editor(scenarios::empty_map(16, 12), &temp_maps("undo"));
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
    act(
        &mut app,
        [A::SetName("Undo test".into()), A::SetDescription("Steps".into())],
    );
    assert_eq!(undo_depth(&app), 4);

    act(&mut app, [A::Undo, A::Undo]);
    assert_eq!(*doc(&app), placed);
    act(&mut app, [A::Undo]);
    assert_eq!(*doc(&app), painted);
    assert_eq!(sim(&app).objects().count(), 0, "the sim follows the undo");
    act(&mut app, [A::Undo, A::Undo]);
    assert_eq!(*doc(&app), original);
    assert!(!app.world().resource::<EditorDoc>().is_dirty());
    assert_eq!(*sim(&app).map(), original.terrain);

    act(&mut app, [A::Redo, A::Redo]);
    assert_eq!(*doc(&app), placed);
    // A new edit drops what could be redone.
    act(&mut app, [A::SetDescription("Other".into()), A::Redo]);
    assert_eq!(doc(&app).description, "Other");
    assert_eq!(doc(&app).name, placed.name);
    // Cancelling a stroke restores the map without an undo step.
    let depth = undo_depth(&app);
    act(
        &mut app,
        [A::SetTool(Tool::Paint), A::Press(at(12, 3)), A::Cancel, A::Release],
    );
    assert_eq!(doc(&app).terrain, placed.terrain);
    assert_eq!(undo_depth(&app), depth);
}

/// Review finding: a selection is a spawn index, which an undo, a redo or a resize may shift onto another spawn.
/// They drop the selection, so Delete can't remove the wrong object.
#[test]
fn undo_redo_and_resize_drop_the_selection() {
    let mut map = scenarios::empty_map(32, 24);
    map.spawns = vec![patch(1, 1), patch(3, 1), patch(5, 1)];
    let mut app = editor(map, &temp_maps("stale"));
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
    act(&mut app, [A::Resize { width: 31, height: 24 }]);
    assert_eq!(selection(&app), None);
}

/// Review finding: a damaged file's spawns at the ends of the `i32` range must not overflow anywhere: loading,
/// validation, placing next to them, selecting, focusing their problems and drawing the overlay.
#[test]
fn spawns_at_the_ends_of_the_coordinate_range_do_no_harm() {
    let station = ObjectKind::SupplyDeliveryStation;
    let extremes = vec![
        Spawn::new(station, None, SpawnAt::Tile(TilePos::new(i32::MAX, i32::MAX))),
        Spawn::new(station, None, SpawnAt::Tile(TilePos::new(i32::MIN, 3))),
        patch(i32::MAX, 0),
    ];
    let mut map = scenarios::empty_map(16, 12);
    map.spawns = extremes.clone();
    let mut app = editor(map, &temp_maps("overflow"));
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
    // The same on a map with four images, whose locked part is generated from such spawns.
    let mut map = super::created(sc_sim::Symmetry::MirrorBothAxes, 16, 16, 4);
    map.spawns = extremes;
    let mut app = editor(map, &temp_maps("overflow_sym"));
    act(
        &mut app,
        [
            A::SetTool(Tool::Select),
            A::Press(at(3, 3)),
            A::Release,
            A::SetKind(station),
            A::Hover(Some(station_at(2, 2))),
            A::Press(station_at(2, 2)),
            A::Release,
            A::Hover(Some(at(15, 11))),
        ],
    );
    for i in 0..problems(&app).len() {
        act(&mut app, [A::FocusProblem(i)]);
        assert!(app.world().resource::<CameraView>().center.is_finite());
    }
}

#[test]
fn resizing_reports_what_falls_outside_and_undo_restores_it() {
    let mut map = scenarios::empty_map(32, 24);
    map.spawns = vec![patch(2, 2), patch(30, 20)];
    map.slots[1].camera_start = Some(at(28, 20));
    let mut app = editor(map.clone(), &temp_maps("resize"));
    act(&mut app, [A::Resize { width: 20, height: 16 }]);
    let terrain = &doc(&app).terrain;
    assert_eq!((terrain.width(), terrain.height()), (20, 16));
    assert_eq!(doc(&app).spawns, vec![patch(2, 2)]);
    assert_eq!(
        doc(&app).slots[1].start,
        TilePos::new(19, 15),
        "S2's start (23, 17) moved onto the map"
    );
    assert_eq!(doc(&app).slots[1].camera_start, None);
    assert_warned(&app, "removed #1");
    assert_warned(&app, "moved S2's start");
    assert_warned(&app, "cleared S2's camera start");
    assert_eq!(sim(&app).map().width(), 20);

    act(&mut app, [A::Undo]);
    assert_eq!(*doc(&app), map);
    // The placeholder limits: sides from 16 to 256 tiles.
    for (width, height) in [(15, 24), (0, 24), (32, 257), (300, 8)] {
        act(&mut app, [A::Resize { width, height }]);
        assert_eq!(*doc(&app), map, "{width} x {height}");
        assert_warned(&app, "from 16 to 256");
    }
    act(&mut app, [A::Resize { width: 40, height: 30 }]);
    assert_eq!(
        doc(&app).terrain.tile(TilePos::new(39, 29)),
        Some(Tile::new(TilePresetId::PLANE))
    );
    assert_eq!(corner(&app, 40, 30), 0);
    assert_eq!(doc(&app).spawns, map.spawns);
}

#[test]
fn player_slots_and_their_markers() {
    let mut app = editor(scenarios::empty_map(32, 24), &temp_maps("slots"));
    let starts = |app: &App| doc(app).slots.iter().map(|s| s.start).collect::<Vec<_>>();
    act(&mut app, [A::AddSlots, A::AddSlots]);
    assert_eq!(doc(&app).slots.len(), 4);
    assert_eq!(tools(&app).slot, SlotId(4), "the new slot is chosen");
    let before = starts(&app);
    assert!(
        before.iter().enumerate().all(|(i, s)| !before[..i].contains(s)),
        "new slots start at free spots: {before:?}"
    );

    // Removing a slot moves the later ones down; the chosen slot stays a slot of the map.
    act(&mut app, [A::RemoveSlots(SlotId(2))]);
    assert_eq!(starts(&app), [before[0], before[2], before[3]]);
    assert!(doc(&app).slot(tools(&app).slot).is_some());
    act(
        &mut app,
        [
            A::RemoveSlots(SlotId(3)),
            A::RemoveSlots(SlotId(2)),
            A::RemoveSlots(SlotId(1)),
        ],
    );
    assert_eq!(doc(&app).slots.len(), 1, "a map keeps one slot");
    assert_warned(&app, "at least one");
    for _ in 0..8 {
        act(&mut app, [A::AddSlots]);
    }
    assert_eq!(doc(&app).slots.len(), 8, "at most 8 slots");
    assert_warned(&app, "At most 8");

    // Markers: move the start, place the camera start, then select and clear the camera start.
    act(
        &mut app,
        [
            A::SelectSlot(SlotId(1)),
            A::SetTool(Tool::Start),
            A::Press(at(3, 3)),
            A::Drag(at(5, 4)),
            A::Release,
        ],
    );
    assert_eq!(doc(&app).slots[0].start, TilePos::new(5, 4));
    act(
        &mut app,
        [
            A::SetTool(Tool::CameraStart),
            A::Press(Vec2::new(500.0, 300.0)),
            A::Release,
        ],
    );
    assert_eq!(doc(&app).slots[0].camera_start, Some(Vec2::new(500.0, 300.0)));
    act(
        &mut app,
        [A::SetTool(Tool::Select), A::Press(Vec2::new(510.0, 290.0)), A::Release],
    );
    assert_eq!(selection(&app), Some(Selection::CameraStart(SlotId(1))));
    act(&mut app, [A::DeleteSelected]);
    assert_eq!(doc(&app).slots[0].camera_start, None);
    act(&mut app, [A::SetCameraStart(SlotId(1), Some(Vec2::new(-5.0, 0.0)))]);
    assert_eq!(doc(&app).slots[0].camera_start, None, "camera starts stay on the map");
    // A start can't be deleted: every slot is a start position.
    act(&mut app, [A::Press(at(5, 4)), A::Release, A::DeleteSelected]);
    assert_eq!(selection(&app), Some(Selection::Start(SlotId(1))));
    assert_eq!(doc(&app).slots[0].start, TilePos::new(5, 4));
    act(&mut app, [A::SetStart(SlotId(1), TilePos::new(40, 3))]);
    assert_eq!(doc(&app).slots[0].start, TilePos::new(5, 4), "starts stay on the map");
}

#[test]
fn validation_problems_are_listed_and_lead_to_their_spot() {
    let mut map = scenarios::empty_map(64, 48);
    map.spawns = vec![patch(2, 2), patch(20, 15), patch(20, 15), patch(70, 3)];
    let dir = temp_maps("problems");
    let mut app = editor(map, &dir);
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
