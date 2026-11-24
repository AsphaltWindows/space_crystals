//! Edits of the map as a whole, without symmetry: undo and redo, resizing, player slots and their markers, and
//! validation problems.

use bevy::prelude::*;
use sc_sim::{ObjectKind, SlotId, Tile, TilePos, TilePresetId, scenarios};
use space_crystals::camera::CameraView;
use space_crystals::editor::actions::EditorAction as A;
use space_crystals::editor::doc::EditorDoc;
use space_crystals::editor::state::{Selection, StatusLevel};
use space_crystals::editor::tools::Tool;

use super::{
    act, assert_warned, at, corner, doc, editor, patch, problems, selection, sim, status, temp_maps, tools, undo_depth,
};

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
