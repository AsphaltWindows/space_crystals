//! New maps from the creation dialog and its placeholder limits (`core/map_editor.md > Map Creation`), the map
//! types (`> Map Types`), and custom tile presets (`> Custom Tile Presets`).

use bevy::prelude::*;
use sc_sim::{
    MapSettings, MapType, ObjectKind, SlotId, Spawn, SpawnAt, Symmetry, TileLook, TilePos, TilePresetId, scenarios,
};
use space_crystals::editor::actions::EditorAction as A;
use space_crystals::editor::creation::{NewMapDialog, NewMapField as F};
use space_crystals::editor::doc::EditorDoc;
use space_crystals::editor::files;
use space_crystals::editor::objects::{PlaceProblem, check_placement};
use space_crystals::editor::presets::PresetFlag;
use space_crystals::editor::tools::Tool;
use space_crystals::editor::ui::texts::PanelText;

use super::{act, assert_warned, at, created, doc, editor, status, temp_maps, tile, tools, undo_depth};

fn dialog(app: &App) -> &NewMapDialog {
    app.world().resource::<NewMapDialog>()
}

fn panel_text(app: &mut App, which: PanelText) -> String {
    let mut texts = app.world_mut().query::<(&PanelText, &Text)>();
    texts
        .iter(app.world())
        .find(|(p, _)| **p == which)
        .map(|(_, t)| t.0.clone())
        .unwrap()
}

#[test]
fn the_creation_dialog_makes_maps_within_the_placeholder_limits() {
    let mut app = editor(scenarios::empty_map(32, 24), &temp_maps("create"));
    act(&mut app, [A::NewMap]);
    assert!(dialog(&app).open);
    assert_eq!(
        dialog(&app).settings,
        MapSettings::default(),
        "Skirmish Default, 64 x 48, 180 degrees, 2 slots"
    );
    // While it is open, the document is left alone.
    let before = doc(&app).clone();
    act(
        &mut app,
        [
            A::SetTool(Tool::Paint),
            A::Press(at(3, 3)),
            A::Release,
            A::SetName("x".into()),
        ],
    );
    assert_eq!(*doc(&app), before);
    assert_warned(&app, "Create or cancel");

    // A Skirmish Default map needs a symmetry; 90 degree rotation makes the map square and needs whole sets of 4.
    act(&mut app, [A::EditNewMap(F::Symmetry(Symmetry::None)), A::CreateMap]);
    assert!(dialog(&app).open);
    assert_warned(&app, "needs a symmetry");
    act(&mut app, [A::EditNewMap(F::Symmetry(Symmetry::Rotate90))]);
    let settings = &dialog(&app).settings;
    assert_eq!((settings.width, settings.height, settings.slots), (64, 64, 4));
    act(&mut app, [A::EditNewMap(F::Height(40))]);
    assert_eq!((dialog(&app).settings.width, dialog(&app).settings.height), (40, 40));

    // The limits: sides from 16 to 256 tiles, at most 8 slots in whole sets, and a name.
    for (field, problem) in [
        (F::Width(15), "sides go from 16 to 256"),
        (F::Width(257), "sides go from 16 to 256"),
        (F::Slots(6), "with this symmetry a map has 4, 8"),
        (F::Slots(12), "with this symmetry a map has 4, 8"),
        (F::Name("  ".into()), "needs a name"),
    ] {
        act(&mut app, [A::EditNewMap(field.clone()), A::CreateMap]);
        assert!(dialog(&app).open, "{field:?}");
        assert_warned(&app, problem);
        act(&mut app, [A::EditNewMap(F::Width(40)), A::EditNewMap(F::Slots(4))]);
        act(&mut app, [A::EditNewMap(F::Name("Square".into()))]);
    }
    assert_eq!(
        panel_text(&mut app, PanelText::NewMapVerdict),
        "Ready: Create makes the map"
    );

    // A Custom map without symmetry, of 3 slots.
    act(
        &mut app,
        [
            A::EditNewMap(F::Type(MapType::Custom)),
            A::EditNewMap(F::Symmetry(Symmetry::None)),
            A::EditNewMap(F::Slots(3)),
            A::EditNewMap(F::Width(20)),
            A::EditNewMap(F::Height(16)),
            A::EditNewMap(F::Name("Three ways".into())),
            A::EditNewMap(F::Description("A test map".into())),
            A::CreateMap,
        ],
    );
    assert!(!dialog(&app).open);
    let map = doc(&app);
    assert_eq!(
        (
            map.map_type,
            map.symmetry,
            map.slots.len(),
            map.terrain.width(),
            map.terrain.height()
        ),
        (MapType::Custom, Symmetry::None, 3, 20, 16)
    );
    assert_eq!(
        (map.name.as_str(), map.description.as_str()),
        ("Three ways", "A test map")
    );
    assert_eq!(map.validate(), vec![]);
    assert_eq!(app.world().resource::<EditorDoc>().path(), None);
    assert_eq!(undo_depth(&app), 0);

    // Choosing Skirmish Default puts a symmetry back, and fits the slots to it.
    act(&mut app, [A::NewMap, A::EditNewMap(F::Type(MapType::SkirmishDefault))]);
    let settings = &dialog(&app).settings;
    assert_eq!((settings.symmetry, settings.slots), (Symmetry::Rotate180, 4));
    // Cancel (or Esc) closes the dialog and keeps the document.
    let kept = doc(&app).clone();
    act(&mut app, [A::Cancel]);
    assert!(!dialog(&app).open);
    act(&mut app, [A::NewMap, A::CancelNewMap]);
    assert!(!dialog(&app).open);
    assert_eq!(*doc(&app), kept);

    // With unsaved changes, New asks twice.
    act(&mut app, [A::SetName("Edited".into()), A::NewMap]);
    assert!(!dialog(&app).open);
    assert_warned(&app, "Unsaved changes");
    act(&mut app, [A::NewMap]);
    assert!(dialog(&app).open);
}

/// The map type and the symmetry are chosen at creation and shown, fixed, in the properties panel.
#[test]
fn the_type_and_the_symmetry_are_fixed_after_creation() {
    let mut app = editor(created(Symmetry::Rotate90, 32, 32, 4), &temp_maps("fixed"));
    app.update();
    assert_eq!(panel_text(&mut app, PanelText::MapType), "Custom (fixed)");
    assert_eq!(panel_text(&mut app, PanelText::Symmetry), "Rotate 90, 4 parts (fixed)");
    // A map with 90 degree rotation stays square.
    act(&mut app, [A::Resize { width: 40, height: 32 }]);
    assert_eq!(doc(&app).terrain.width(), 32);
    assert_warned(&app, "stays square");
    act(&mut app, [A::Resize { width: 40, height: 40 }]);
    assert_eq!((doc(&app).terrain.width(), doc(&app).terrain.height()), (40, 40));
    assert_eq!(doc(&app).first_asymmetry(), None);

    // The resize is an unsaved change: New asks twice.
    act(&mut app, [A::NewMap, A::NewMap, A::CreateMap]);
    app.update();
    assert_eq!(panel_text(&mut app, PanelText::MapType), "Skirmish Default (fixed)");
    assert_eq!(panel_text(&mut app, PanelText::Symmetry), "Rotate 180, 2 parts (fixed)");
}

/// `core/map_editor.md > Map Types > Skirmish Default`: no player-owned objects outside the Starting Layouts.
#[test]
fn skirmish_maps_take_no_owned_objects_outside_the_layouts() {
    let mut app = editor(scenarios::dev_skirmish_map(), &temp_maps("skirmish"));
    act(&mut app, [A::SetOwner(Some(SlotId(1)))]);
    assert_eq!(tools(&app).owner, None);
    assert_warned(&app, "neutral objects only");
    let owned = Spawn::new(
        ObjectKind::CrystalPatch,
        Some(SlotId(1)),
        SpawnAt::Tile(TilePos::new(10, 20)),
    );
    assert_eq!(
        check_placement(doc(&app), &owned, None),
        Err(PlaceProblem::OwnedOnSkirmishMap)
    );
    let custom = created(Symmetry::None, 32, 24, 2);
    assert_eq!(check_placement(&custom, &owned, None), Ok(()));
    // Switching to a Skirmish map drops an owner chosen before.
    let mut app = editor(custom, &temp_maps("skirmish_owner"));
    act(&mut app, [A::SetOwner(Some(SlotId(2)))]);
    assert_eq!(tools(&app).owner, Some(SlotId(2)));
    act(&mut app, [A::NewMap, A::CreateMap]);
    assert_eq!(tools(&app).owner, None);
}

/// `core/map_editor.md > Custom Tile Presets`: custom presets are added, named, given properties and a look, painted,
/// saved, and removed once no tile uses them. The default presets are fixed.
#[test]
fn custom_presets_are_added_edited_painted_saved_and_removed() {
    let dir = temp_maps("presets");
    let mut app = editor(scenarios::empty_map(16, 12), &dir);
    act(&mut app, [A::AddPreset]);
    let ash = TilePresetId(5);
    assert_eq!(tools(&app).preset, ash, "the new preset is chosen");
    let preset = doc(&app).terrain.presets()[5].clone();
    assert_eq!((preset.name.as_str(), preset.look), ("Custom 1", TileLook::Plane));
    act(
        &mut app,
        [
            A::SetPresetName(ash, "Ash".into()),
            A::TogglePresetFlag(ash, PresetFlag::Buildable),
            A::TogglePresetFlag(ash, PresetFlag::Drillable),
            A::SetPresetLook(ash, TileLook::Rugged),
        ],
    );
    let preset = doc(&app).terrain.presets()[5].clone();
    assert_eq!(preset.name, "Ash");
    // Plane's properties, with Buildable and Drillable switched off.
    assert!(!preset.buildable && !preset.drillable && preset.traversible && preset.recruitable);
    assert_eq!(preset.look, TileLook::Rugged);
    assert_eq!(undo_depth(&app), 5);

    // Painted with, and shown by the sim.
    act(
        &mut app,
        [A::SetBrush(1), A::Press(at(2, 2)), A::Drag(at(4, 2)), A::Release],
    );
    assert_eq!(tools(&app).tool, Tool::Paint);
    assert_eq!(tile(&app, 3, 2).preset, ash);
    assert_eq!(super::sim(&app).map().presets()[5].name, "Ash");

    // Saved and opened again.
    act(&mut app, [A::Save]);
    let path = app.world().resource::<EditorDoc>().path().unwrap().to_path_buf();
    assert_eq!(files::load(&path).unwrap().terrain.presets()[5], preset);

    // The default presets are fixed.
    for action in [
        A::SetPresetName(TilePresetId::PLANE, "Grass".into()),
        A::TogglePresetFlag(TilePresetId::WATER, PresetFlag::Traversible),
        A::SetPresetLook(TilePresetId::CLIFF, TileLook::Water),
        A::RemovePreset(TilePresetId::MOUNTAIN),
    ] {
        act(&mut app, [action]);
        assert_warned(&app, "default preset");
    }
    assert_eq!(doc(&app).terrain.presets().len(), 6);

    // A preset in use can't be removed; once its tiles are painted over it can.
    act(&mut app, [A::RemovePreset(ash)]);
    assert_warned(&app, "still used by 3 tile(s)");
    act(&mut app, [A::Undo, A::RemovePreset(ash)]);
    assert_eq!(doc(&app).terrain.presets().len(), 5);
    assert_eq!(tools(&app).preset, TilePresetId::PLANE, "the chosen preset is gone");
    act(&mut app, [A::Undo]);
    assert_eq!(doc(&app).terrain.presets()[5], preset);

    // Removing a preset moves the later ones down, with their tiles and the chosen one.
    act(
        &mut app,
        [
            A::AddPreset,
            A::SetPresetName(TilePresetId(6), "Basalt".into()),
            A::Press(at(8, 8)),
            A::Release,
            A::RemovePreset(ash),
        ],
    );
    assert_eq!(doc(&app).terrain.presets()[5].name, "Basalt");
    assert_eq!(tile(&app, 8, 8).preset, TilePresetId(5));
    assert_eq!(tools(&app).preset, TilePresetId(5));
    assert_eq!(status(&app).level, space_crystals::editor::state::StatusLevel::Info);
    std::fs::remove_dir_all(&dir).unwrap();
}

/// After Open and New, the tools work on the new document: no stale preset, slot, owner, selection or gesture.
#[test]
fn the_tools_work_after_open_and_new() {
    let dir = temp_maps("after_open");
    let mut custom = scenarios::empty_map(16, 12);
    custom.terrain.add_preset(sc_sim::TilePreset {
        name: "Ash".into(),
        look: TileLook::Rugged,
        ..custom.terrain.presets()[0].clone()
    });
    let symmetric = created(Symmetry::Rotate180, 24, 24, 4);
    let (custom_path, symmetric_path) = (dir.join("custom.json"), dir.join("symmetric.json"));
    files::save(&custom_path, &custom).unwrap();
    files::save(&symmetric_path, &symmetric).unwrap();

    let mut app = editor(custom, &dir);
    act(
        &mut app,
        [
            A::SelectSlot(SlotId(2)),
            A::SetOwner(Some(SlotId(2))),
            A::SetPreset(TilePresetId(5)),
            A::SetTool(Tool::Paint),
            A::Press(at(2, 2)),
        ],
    );
    // Opening mid-stroke (asked twice: the stroke is an unsaved change) drops the stroke. The symmetric map has no
    // preset 5, and slot 2 is generated there.
    act(
        &mut app,
        [
            A::Open(symmetric_path.clone()),
            A::Open(symmetric_path),
            A::Release,
            A::Hover(Some(at(3, 3))),
        ],
    );
    assert_eq!(*doc(&app), symmetric);
    assert_eq!(tools(&app).preset, TilePresetId::PLANE);
    assert_eq!(tools(&app).slot, SlotId(1), "slot 2 is the image of slot 1");
    assert_eq!(tools(&app).owner, Some(SlotId(2)), "a Custom map keeps the owner");
    act(&mut app, [A::Press(at(3, 3)), A::Release]);
    assert_eq!(tile(&app, 3, 3).preset, TilePresetId::PLANE);
    act(
        &mut app,
        [A::SetPreset(TilePresetId::WATER), A::Press(at(3, 3)), A::Release],
    );
    assert_eq!(tile(&app, 20, 20).preset, TilePresetId::WATER);
    act(&mut app, [A::SetTool(Tool::Start), A::Press(at(5, 5)), A::Release]);
    assert_eq!(doc(&app).slots[0].start, TilePos::new(5, 5));

    // A new map: Skirmish Default, so neutral objects only.
    act(&mut app, [A::Save, A::NewMap, A::CreateMap]);
    assert_eq!(tools(&app).owner, None);
    act(
        &mut app,
        [A::SetKind(ObjectKind::CrystalPatch), A::Press(at(4, 4)), A::Release],
    );
    assert_eq!(doc(&app).spawns.len(), 2, "the patch and its image");
    std::fs::remove_dir_all(&dir).unwrap();
}
