//! Edits: terrain, objects, undo and redo, resizing, player slots and validation problems.

use bevy::prelude::*;
use sc_sim::{Faction, ObjectKind, PlayerId, Spawn, SpawnAt, Tile, TilePos, TilePresetId, scenarios};
use space_crystals::camera::CameraView;
use space_crystals::editor::actions::EditorAction as A;
use space_crystals::editor::doc::EditorDoc;
use space_crystals::editor::overlay::Ghost;
use space_crystals::editor::state::{EditorProblems, EditorSelection, Selection, StatusLevel};
use space_crystals::editor::tools::{ElevationMode, Symmetry, Tool};
use space_crystals::map_render::MapView;

use super::{act, at, doc, editor, patch, sim, status, temp_maps, tile, tools, undo_depth, visual_count};

#[test]
fn painting_elevation_and_fill_edit_the_terrain() {
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

    // Set elevation, then raise: a stroke raises each tile once, however often it passes.
    act(
        &mut app,
        [A::SetElevation(3), A::SetBrush(1), A::Press(at(1, 1)), A::Release],
    );
    assert_eq!(tools(&app).tool, Tool::Elevation);
    assert_eq!(tile(&app, 1, 1), Tile::new(TilePresetId::PLANE, 3));
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
    assert_eq!(tile(&app, 1, 1).elevation, 4);
    assert_eq!(tile(&app, 2, 1).elevation, 1);
    act(
        &mut app,
        [
            A::SetElevationMode(ElevationMode::Lower),
            A::Press(at(2, 1)),
            A::Release,
        ],
    );
    assert_eq!(tile(&app, 2, 1).elevation, 0);
    act(&mut app, [A::StepElevation(99)]);
    assert_eq!(tools(&app).elevation, 16, "16 is the top level");

    // Fill: the connected flat plane becomes rugged; the water and the raised tile stay.
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
    assert_eq!(tile(&app, 15, 0), Tile::new(TilePresetId::RUGGED, 0));
    assert_eq!(tile(&app, 6, 5).preset, TilePresetId::WATER);
    assert_eq!(tile(&app, 1, 1), Tile::new(TilePresetId::PLANE, 4));

    // Symmetry repeats strokes at the mirror image.
    act(
        &mut app,
        [
            A::SetSymmetry(Symmetry::MirrorX),
            A::SetPreset(TilePresetId::MOUNTAIN),
            A::Press(at(0, 0)),
            A::Release,
        ],
    );
    assert_eq!(tile(&app, 0, 0).preset, TilePresetId::MOUNTAIN);
    assert_eq!(tile(&app, 15, 0).preset, TilePresetId::MOUNTAIN);
    act(
        &mut app,
        [A::SetSymmetry(Symmetry::Point), A::Press(at(2, 2)), A::Release],
    );
    assert_eq!(tile(&app, 13, 9).preset, TilePresetId::MOUNTAIN);

    // The eyedropper picks a tile's preset and elevation.
    act(&mut app, [A::Pick(at(6, 5))]);
    assert_eq!((tools(&app).preset, tools(&app).elevation), (TilePresetId::WATER, 0));
}

#[test]
fn objects_are_placed_moved_and_deleted() {
    let mut app = editor(scenarios::empty(16, 12), &temp_maps("objects"));
    let station = ObjectKind::SupplyDeliveryStation;

    // The ghost follows the cursor; the station's 2×2 footprint centres on the nearest grid point.
    act(&mut app, [A::SetKind(station), A::Hover(Some(Vec2::new(320.0, 320.0)))]);
    assert_eq!(tools(&app).tool, Tool::Place);
    let mut ghosts = app.world_mut().query::<(&Ghost, &Visibility, &Transform)>();
    let (_, visibility, transform) = ghosts.iter(app.world()).find(|(g, ..)| !g.mirror).unwrap();
    assert_eq!(*visibility, Visibility::Inherited);
    assert_eq!(transform.translation.truncate(), Vec2::new(320.0, -320.0));

    act(&mut app, [A::Press(Vec2::new(320.0, 320.0)), A::Release]);
    assert_eq!(
        doc(&app).spawns,
        vec![Spawn {
            kind: station,
            owner: None,
            at: SpawnAt::Tile(TilePos::new(4, 4))
        }]
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
    assert_eq!(app.world().resource::<EditorSelection>().0, Some(Selection::Spawn(0)));
    assert_eq!(doc(&app).spawns[0].at, SpawnAt::Tile(TilePos::new(9, 6)));
    let station_object = sim(&app).objects().find(|o| o.kind == station).unwrap().pos;
    assert_eq!(station_object, Vec2::new(640.0, 448.0), "the sim shows the move");

    act(&mut app, [A::DeleteSelected]);
    assert!(doc(&app).spawns.iter().all(|s| s.kind != station));
    assert_eq!(app.world().resource::<EditorSelection>().0, None);
    assert_eq!(visual_count(&mut app), 3);

    // Mirrored placement.
    act(
        &mut app,
        [
            A::SetSymmetry(Symmetry::MirrorY),
            A::SetKind(ObjectKind::CrystalPatch),
            A::Press(at(7, 0)),
            A::Release,
        ],
    );
    let spots: Vec<SpawnAt> = doc(&app).spawns.iter().map(|s| s.at).collect();
    assert!(spots.contains(&SpawnAt::Tile(TilePos::new(7, 0))) && spots.contains(&SpawnAt::Tile(TilePos::new(7, 11))));
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
    act(&mut app, [A::Resize { width: 20, height: 14 }]);
    assert_eq!(
        doc(&app).map.tile(TilePos::new(19, 13)),
        Some(Tile::new(TilePresetId::PLANE, 0))
    );
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
    assert_eq!(
        app.world().resource::<EditorSelection>().0,
        Some(Selection::CameraStart(PlayerId(1)))
    );
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
    let problems = app.world().resource::<EditorProblems>().0.clone();
    assert_eq!(problems.len(), 2, "{problems:?}");
    // The preview shows every spawn that has a place, overlaps included, so it can be fixed.
    assert_eq!(sim(&app).objects().count(), 3);

    act(&mut app, [A::FocusProblem(0)]);
    assert_eq!(app.world().resource::<CameraView>().center, at(20, 15));
    assert_eq!(app.world().resource::<EditorSelection>().0, Some(Selection::Spawn(2)));
    act(&mut app, [A::DeleteSelected]);
    assert_eq!(app.world().resource::<EditorProblems>().0.len(), 1);

    // Saving an invalid map works, with a warning.
    act(&mut app, [A::Save]);
    assert_eq!(status(&app).level, StatusLevel::Warning);
    act(&mut app, [A::FocusProblem(0), A::DeleteSelected]);
    assert!(app.world().resource::<EditorProblems>().0.is_empty());
    std::fs::remove_dir_all(&dir).unwrap();
}
