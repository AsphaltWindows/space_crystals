//! Generated symmetry (`core/map_editor.md > Symmetry > Editing a symmetric map`): only the editable part of a
//! symmetric map is edited; after every action the rest is generated from it: tiles, heights, objects with their
//! values, slots with their markers and Starting Layouts. Edits of the locked part are refused.

use bevy::prelude::*;
use sc_sim::scenario::Asymmetry;
use sc_sim::{
    Faction, LayoutObject, ObjectKind, ScenarioProblem, SlotId, Spawn, SpawnAt, SpawnSettings, Symmetry, Tile, TilePos,
    TilePresetId,
};
use space_crystals::editor::actions::EditorAction as A;
use space_crystals::editor::doc::EditorDoc;
use space_crystals::editor::state::Selection;
use space_crystals::editor::tools::Tool;

use super::{
    act, assert_warned, at, created, doc, editor, patch, problems, selection, station_at, temp_maps, tile, tools,
    undo_depth,
};

/// Every symmetry, with a slot count it allows, on a square map that fits them all.
const MODES: [(Symmetry, usize); 5] = [
    (Symmetry::Rotate180, 2),
    (Symmetry::Rotate90, 4),
    (Symmetry::MirrorVerticalAxis, 2),
    (Symmetry::MirrorHorizontalAxis, 2),
    (Symmetry::MirrorBothAxes, 4),
];

#[test]
fn every_symmetry_generates_the_locked_part_after_each_edit() {
    for (symmetry, slots) in MODES {
        let mut app = editor(created(symmetry, 24, 24, slots), &temp_maps("generate"));
        let amount = SpawnSettings::CrystalPatch { starting_amount: 900 };
        act(
            &mut app,
            [
                A::SetPreset(TilePresetId::WATER),
                A::SetBrush(1),
                A::Press(at(1, 2)),
                A::Release,
                A::SetElevation(2),
                A::Press(at(4, 6)),
                A::Release,
                A::SetKind(ObjectKind::CrystalPatch),
                A::Press(at(3, 9)),
                A::Release,
                A::SetSpawnSettings {
                    spawn: 0,
                    settings: amount,
                },
                A::SetStart(SlotId(1), TilePos::new(5, 5)),
                A::SetCameraStart(SlotId(1), Some(at(6, 6))),
            ],
        );
        let map = doc(&app).clone();
        let images = map.transforms();
        assert_eq!(images.len(), slots, "{symmetry:?}");
        assert_eq!(map.first_asymmetry(), None, "{symmetry:?}");
        assert_eq!(problems(&app), vec![], "{symmetry:?}");
        assert_eq!(
            undo_depth(&app),
            6,
            "{symmetry:?}: each edit with its images is one step"
        );
        for (j, m) in images.iter().enumerate() {
            assert_eq!(
                tile(&app, m.tile(TilePos::new(1, 2)).x, m.tile(TilePos::new(1, 2)).y).preset,
                TilePresetId::WATER,
                "{symmetry:?} image {j}"
            );
            let level = m.tile(TilePos::new(4, 6));
            assert_eq!(map.terrain.tile_level(level), Some(2), "{symmetry:?} image {j}");
            let slot = map.image_slot(SlotId(1), j);
            assert_eq!(map.slot(slot).unwrap().start, m.tile(TilePos::new(5, 5)));
            assert_eq!(map.slot(slot).unwrap().camera_start, Some(m.point(at(6, 6))));
        }
        assert_eq!(map.spawns.len(), slots, "{symmetry:?}: the patch and its images");
        assert!(
            map.spawns.iter().all(|s| s.settings == Some(amount)),
            "{symmetry:?}: the values follow"
        );
        // Undo and redo bring back whole, symmetric maps.
        act(&mut app, [A::Undo]);
        assert_eq!(doc(&app).first_asymmetry(), None);
        assert_eq!(doc(&app).slots[1].camera_start, None, "{symmetry:?}");
        act(&mut app, [A::Redo]);
        assert_eq!(*doc(&app), map);
    }
}

/// 90° rotation: an edit appears four times, turned about the center.
#[test]
fn rotation_by_90_degrees_repeats_an_edit_four_ways() {
    let mut app = editor(created(Symmetry::Rotate90, 24, 24, 4), &temp_maps("rotate90"));
    act(
        &mut app,
        [
            A::SetPreset(TilePresetId::MOUNTAIN),
            A::SetBrush(1),
            A::Press(at(1, 2)),
            A::Release,
            A::SetKind(ObjectKind::SupplyDeliveryStation),
            A::Press(station_at(3, 4)),
            A::Release,
        ],
    );
    for (x, y) in [(1, 2), (21, 1), (22, 21), (2, 22)] {
        assert_eq!(tile(&app, x, y).preset, TilePresetId::MOUNTAIN, "({x}, {y})");
    }
    assert_eq!(tile(&app, 2, 1).preset, TilePresetId::PLANE);
    let spots: Vec<SpawnAt> = doc(&app).spawns.iter().map(|s| s.at).collect();
    let expected: Vec<SpawnAt> = [(3, 4), (18, 3), (19, 18), (4, 19)]
        .map(|(x, y)| SpawnAt::Tile(TilePos::new(x, y)))
        .into();
    assert_eq!(spots, expected);
    assert_eq!(doc(&app).first_asymmetry(), None);
}

#[test]
fn edits_of_the_locked_part_are_refused() {
    let mut map = created(Symmetry::Rotate180, 24, 24, 2);
    map.spawns = vec![patch(3, 3)];
    map.regenerate();
    let mut app = editor(map.clone(), &temp_maps("locked"));
    let locked = at(20, 5);
    for tool in [Tool::Paint, Tool::Elevation, Tool::Fill, Tool::Start] {
        act(
            &mut app,
            [A::SetTool(tool), A::Press(locked), A::Drag(at(21, 6)), A::Release],
        );
        assert_eq!(*doc(&app), map, "{tool:?}");
        assert_warned(&app, "Locked");
    }
    act(&mut app, [A::SetTool(Tool::CameraStart), A::Press(locked), A::Release]);
    assert_warned(&app, "Locked");
    act(
        &mut app,
        [A::SetKind(ObjectKind::CrystalPatch), A::Press(locked), A::Release],
    );
    assert_warned(&app, "Locked");
    // The generated slot's markers, the image of the patch and its values.
    act(&mut app, [A::SelectSlot(SlotId(2))]);
    assert_eq!(tools(&app).slot, SlotId(1));
    assert_warned(&app, "slot 2 (the image of slot 1)");
    act(&mut app, [A::SetStart(SlotId(2), TilePos::new(5, 5))]);
    assert_warned(&app, "Locked");
    act(&mut app, [A::SetCameraStart(SlotId(2), Some(at(5, 5)))]);
    assert_warned(&app, "Locked");
    act(&mut app, [A::SetStart(SlotId(1), TilePos::new(20, 5))]);
    assert_warned(&app, "Locked");
    act(&mut app, [A::SetTool(Tool::Select), A::Press(at(20, 20)), A::Release]);
    assert_eq!(selection(&app), None, "the image of the patch can't be selected");
    assert_warned(&app, "Locked");
    act(
        &mut app,
        [A::SetSpawnSettings {
            spawn: 1,
            settings: SpawnSettings::CrystalPatch { starting_amount: 5 },
        }],
    );
    assert_warned(&app, "Locked");
    assert_eq!(*doc(&app), map);
    assert_eq!(undo_depth(&app), 0);

    // Reading the locked part is fine.
    act(&mut app, [A::SetPreset(TilePresetId::WATER), A::Pick(locked)]);
    assert_eq!(tools(&app).preset, TilePresetId::PLANE);

    // A stroke that runs into the locked part edits the editable tiles; their images cover the rest.
    act(
        &mut app,
        [
            A::SetPreset(TilePresetId::RUGGED),
            A::SetBrush(3),
            A::Press(at(11, 8)),
            A::Drag(at(13, 8)),
            A::Release,
        ],
    );
    for (x, y) in [(10, 7), (11, 8), (11, 9), (12, 14), (13, 15), (12, 16)] {
        assert_eq!(tile(&app, x, y).preset, TilePresetId::RUGGED, "({x}, {y})");
    }
    for (x, y) in [(12, 8), (13, 8), (9, 8)] {
        assert_eq!(tile(&app, x, y).preset, TilePresetId::PLANE, "({x}, {y})");
    }
    assert_eq!(doc(&app).first_asymmetry(), None);
}

/// The placeholder rule for objects on an axis or at the center (`core/map_editor.md > Symmetry`'s TBD): an object
/// that is its own image stays one object; one that would overlap its own image is refused.
#[test]
fn objects_on_an_axis_or_the_center_follow_the_placeholder_rule() {
    let station = ObjectKind::SupplyDeliveryStation;
    let mut app = editor(created(Symmetry::Rotate180, 24, 24, 2), &temp_maps("center"));
    act(
        &mut app,
        [A::SetKind(station), A::Press(station_at(11, 11)), A::Release],
    );
    assert_eq!(
        doc(&app).spawns,
        vec![Spawn::new(station, None, SpawnAt::Tile(TilePos::new(11, 11)))],
        "a station at the center is its own image"
    );

    // An odd width puts the mirror axis through the middle column, x = 12.
    let mut app = editor(created(Symmetry::MirrorVerticalAxis, 25, 24, 2), &temp_maps("axis"));
    act(
        &mut app,
        [A::SetKind(ObjectKind::CrystalPatch), A::Press(at(12, 5)), A::Release],
    );
    assert_eq!(
        doc(&app).spawns,
        vec![patch(12, 5)],
        "a patch on the axis is its own image"
    );
    act(&mut app, [A::SetKind(station), A::Press(station_at(11, 8)), A::Release]);
    assert_eq!(doc(&app).spawns.len(), 1);
    assert_warned(&app, "its own mirror image");
    assert_eq!(doc(&app).first_asymmetry(), None);
}

#[test]
fn slots_come_in_sets_and_their_markers_are_generated() {
    let mut app = editor(created(Symmetry::Rotate180, 24, 24, 2), &temp_maps("slot_sets"));
    act(&mut app, [A::AddSlots]);
    assert_eq!(doc(&app).slots.len(), 4);
    assert_eq!(tools(&app).slot, SlotId(3));
    act(
        &mut app,
        [
            A::SetTool(Tool::Start),
            A::Press(at(4, 20)),
            A::Release,
            A::SetTool(Tool::CameraStart),
            A::Press(at(5, 18)),
            A::Release,
        ],
    );
    let map = doc(&app).clone();
    assert_eq!(map.slots[2].start, TilePos::new(4, 20));
    assert_eq!(map.slots[3].start, TilePos::new(19, 3));
    assert_eq!(map.slots[3].camera_start, Some(at(18, 5)));
    assert_eq!(map.first_asymmetry(), None);

    // Starting Layouts are mirrored with their slots (the editor has no tool for them until M2).
    let object = |x, y| LayoutObject {
        kind: ObjectKind::CrystalPatch,
        at: SpawnAt::Tile(TilePos::new(x, y)),
        orientation: default(),
    };
    app.world_mut()
        .resource_mut::<EditorDoc>()
        .edit(|m| m.slots[0].layouts.get_mut(Faction::Syndicate).push(object(6, 7)));
    act(&mut app, [A::SetName("layouts".into())]);
    assert_eq!(doc(&app).slots[1].layouts.get(Faction::Syndicate), [object(17, 16)]);
    assert!(doc(&app).slots[1].layouts.get(Faction::Gdo).is_empty());

    // Selecting and clearing S3's camera start clears its image's.
    act(&mut app, [A::SetTool(Tool::Select), A::Press(at(5, 18)), A::Release]);
    assert_eq!(selection(&app), Some(Selection::CameraStart(SlotId(3))));
    act(&mut app, [A::DeleteSelected]);
    assert_eq!(doc(&app).slots[3].camera_start, None);

    // Removing a slot removes its whole set.
    act(&mut app, [A::RemoveSlots(SlotId(4))]);
    let starts: Vec<TilePos> = doc(&app).slots.iter().map(|s| s.start).collect();
    assert_eq!(starts, [map.slots[0].start, map.slots[1].start]);
}

/// A file whose locked part differs from what its editable part generates is flagged; the first edit regenerates it.
#[test]
fn an_asymmetric_file_is_flagged_until_the_first_edit() {
    let mut map = created(Symmetry::Rotate180, 24, 24, 2);
    map.terrain
        .set_tile(TilePos::new(20, 5), Tile::new(TilePresetId::WATER));
    // A locked spawn listed before an editable one: the selection follows the editable one.
    map.spawns = vec![patch(20, 20), patch(3, 3)];
    let mut app = editor(map, &temp_maps("asymmetric"));
    assert!(
        problems(&app).contains(&ScenarioProblem::NotSymmetric(Asymmetry::Tile(TilePos::new(20, 5)))),
        "{:?}",
        problems(&app)
    );
    act(&mut app, [A::SetTool(Tool::Select), A::Press(at(3, 3)), A::Release]);
    assert_eq!(selection(&app), Some(Selection::Spawn(1)));
    act(&mut app, [A::SetDescription("regenerated".into())]);
    assert_eq!(problems(&app), vec![]);
    assert_eq!(tile(&app, 20, 5).preset, TilePresetId::PLANE);
    assert_eq!(doc(&app).spawns, vec![patch(3, 3), patch(20, 20)]);
    assert_eq!(
        selection(&app),
        Some(Selection::Spawn(0)),
        "the selection kept its spawn"
    );
    act(&mut app, [A::DeleteSelected]);
    assert_eq!(doc(&app).spawns, vec![], "with its image");
}
