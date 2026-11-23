//! The review findings on the first editor: the window's close button, text fields that lose their text, UI work on
//! idle frames, and files that can't be played.

use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input_focus::{FocusCause, InputFocus};
use bevy::prelude::*;
use bevy::text::EditableText;
use bevy::window::WindowCloseRequested;
use sc_sim::{ObjectKind, ScenarioProblem, SlotId, TilePos, TilePresetId, scenarios};
use space_crystals::ClientMode;
use space_crystals::editor::EditorConfig;
use space_crystals::editor::actions::EditorAction as A;
use space_crystals::editor::doc::EditorDoc;
use space_crystals::editor::files;
use space_crystals::editor::presets::PresetFlag;
use space_crystals::editor::state::StatusLevel;
use space_crystals::editor::tools::Tool;
use space_crystals::editor::ui::UiAction;
use space_crystals::editor::ui::fields::{SpawnValue, TextField};

use super::{act, assert_warned, at, doc, editor, editor_with, problems, status, tap, temp_maps};

fn request_close(app: &mut App) {
    app.world_mut().write_message(WindowCloseRequested {
        window: Entity::PLACEHOLDER,
    });
    app.update();
}

/// Finding 7: the window's close button went around the unsaved-changes check.
#[test]
fn closing_the_window_asks_about_unsaved_changes() {
    let mut app = editor(scenarios::empty_map(16, 12), &temp_maps("close_clean"));
    request_close(&mut app);
    assert!(app.should_exit().is_some(), "nothing to lose: the editor closes");

    let mut app = editor(scenarios::empty_map(16, 12), &temp_maps("close_dirty"));
    act(&mut app, [A::SetName("edited".into())]);
    request_close(&mut app);
    assert!(app.should_exit().is_none());
    assert_warned(&app, "close the window again to discard them");
    request_close(&mut app);
    assert!(app.should_exit().is_some());

    // Also during a play-test, where every other editor action waits.
    let mut app = editor(scenarios::empty_map(16, 12), &temp_maps("close_playing"));
    act(&mut app, [A::SetName("edited".into()), A::PlayTest]);
    app.update();
    assert_eq!(*app.world().resource::<ClientMode>(), ClientMode::PlayTest);
    request_close(&mut app);
    assert!(app.should_exit().is_none());
    assert_warned(&app, "Unsaved changes");
    request_close(&mut app);
    assert!(app.should_exit().is_some());
}

/// The entity of the text field `which`.
fn field(app: &mut App, which: TextField) -> Entity {
    let mut fields = app.world_mut().query::<(Entity, &TextField)>();
    let found = fields.iter(app.world()).find(|(_, f)| **f == which).map(|(e, _)| e);
    found.unwrap_or_else(|| panic!("no {which:?} field"))
}

fn field_text(app: &App, entity: Entity) -> String {
    app.world().get::<EditableText>(entity).unwrap().value().to_string()
}

/// Gives `entity` the keyboard, as a click on it does, and types `text` over its contents.
fn type_into(app: &mut App, entity: Entity, text: &str) {
    app.world_mut()
        .resource_mut::<InputFocus>()
        .set(entity, FocusCause::Pressed);
    app.update();
    app.world_mut()
        .get_mut::<EditableText>(entity)
        .unwrap()
        .editor
        .set_text(text);
}

fn key(app: &mut App, key_code: KeyCode, logical_key: Key, state: ButtonState) {
    app.world_mut().write_message(KeyboardInput {
        key_code,
        logical_key,
        state,
        text: None,
        repeat: false,
        window: Entity::PLACEHOLDER,
    });
}

/// Finding 2: text typed into a field was lost when the next click did something else (here: saved the map), or when
/// the panel was rebuilt.
#[test]
fn text_fields_commit_before_the_next_action_and_survive_rebuilds() {
    let dir = temp_maps("fields");
    let mut app = editor(scenarios::empty_map(16, 12), &dir);
    app.init_resource::<InputFocus>();
    let name = field(&mut app, TextField::MapName);
    assert_eq!(field_text(&app, name), "empty");

    // A click on Save takes the keyboard from the field: the name commits first, then the map saves.
    type_into(&mut app, name, "Renamed");
    app.world_mut().resource_mut::<InputFocus>().clear();
    act(&mut app, [A::Save]);
    assert_eq!(doc(&app).name, "Renamed");
    assert_eq!(files::load(&dir.join("renamed.json")).unwrap().name, "Renamed");
    assert!(!app.world().resource::<EditorDoc>().is_dirty());

    // Enter commits too; Escape gives up the text.
    let description = field(&mut app, TextField::Description);
    type_into(&mut app, description, "With Enter");
    key(&mut app, KeyCode::Enter, Key::Enter, ButtonState::Pressed);
    app.update();
    key(&mut app, KeyCode::Enter, Key::Enter, ButtonState::Released);
    app.update();
    assert_eq!(doc(&app).description, "With Enter");
    assert_eq!(app.world().resource::<InputFocus>().get(), None);
    type_into(&mut app, description, "Given up");
    key(&mut app, KeyCode::Escape, Key::Escape, ButtonState::Pressed);
    app.world_mut().resource_mut::<InputFocus>().clear();
    app.update();
    assert_eq!(doc(&app).description, "With Enter");
    assert_eq!(
        field_text(&app, description),
        "With Enter",
        "the field shows the map's value again"
    );
    key(&mut app, KeyCode::Escape, Key::Escape, ButtonState::Released);
    app.update();

    // A bad value is refused with a message, and the field shows the map's value again.
    let width = field(&mut app, TextField::Width);
    type_into(&mut app, width, "wide");
    app.world_mut().resource_mut::<InputFocus>().clear();
    app.update();
    assert_eq!(status(&app).level, StatusLevel::Warning);
    assert_eq!(field_text(&app, width), "16");

    // Marker moves, painting, and preset edits rebuild no field: one being typed into keeps its text.
    act(&mut app, [A::AddPreset]);
    let preset = TilePresetId(5);
    let preset_name = field(&mut app, TextField::PresetName(preset));
    act(
        &mut app,
        [
            A::SetKind(ObjectKind::CrystalPatch),
            A::Press(at(3, 3)),
            A::Release,
            A::SetTool(Tool::Select),
            A::Press(at(3, 3)),
            A::Release,
        ],
    );
    let amount = field(
        &mut app,
        TextField::SpawnValue {
            spawn: 0,
            value: SpawnValue::StartingAmount,
        },
    );
    type_into(&mut app, preset_name, "Half typed");
    act(
        &mut app,
        [
            A::SetStart(SlotId(1), TilePos::new(8, 8)),
            A::SetCameraStart(SlotId(2), Some(at(9, 9))),
            A::TogglePresetFlag(preset, PresetFlag::Buildable),
            A::SetPresetLook(preset, sc_sim::TileLook::Water),
            A::Press(at(3, 3)),
            A::Drag(at(5, 6)),
            A::Release,
        ],
    );
    assert_eq!(field(&mut app, TextField::MapName), name);
    assert_eq!(field(&mut app, TextField::PresetName(preset)), preset_name);
    let moved = field(
        &mut app,
        TextField::SpawnValue {
            spawn: 0,
            value: SpawnValue::StartingAmount,
        },
    );
    assert_eq!(moved, amount, "moving the selected object keeps its fields");
    assert_eq!(field_text(&app, preset_name), "Half typed");

    // The race of the finding: the click that takes the keyboard from the field also rebuilds its list (choosing
    // another preset replaces the preset editor). The text commits to the preset it was typed for first.
    app.world_mut().resource_mut::<InputFocus>().clear();
    act(&mut app, [A::SetPreset(TilePresetId::WATER)]);
    assert_eq!(doc(&app).terrain.presets()[5].name, "Half typed");
    assert!(
        app.world().get_entity(preset_name).is_err(),
        "the preset editor was rebuilt for Water"
    );
    std::fs::remove_dir_all(&dir).unwrap();
}

/// Changes to the editor's UI seen by the frame's end: UI entities changed or spawned.
#[derive(Resource, Default)]
struct UiChanges(usize);

/// A text, a colour, a field, a visibility changed, or a button was spawned.
type UiChanged = Or<(
    Changed<Text>,
    Changed<TextColor>,
    Changed<BackgroundColor>,
    Changed<BorderColor>,
    Changed<EditableText>,
    Changed<Visibility>,
    Added<UiAction>,
)>;

fn count_ui_changes(changed: Query<(), UiChanged>, mut changes: ResMut<UiChanges>) {
    changes.0 += changed.iter().count();
}

/// Finding 10: the panels were rebuilt and restyled on every frame.
#[test]
fn idle_frames_leave_the_ui_alone() {
    let mut map = scenarios::dev_skirmish_map();
    map.terrain.add_preset(sc_sim::TilePreset {
        name: "Ash".into(),
        ..map.terrain.presets()[0].clone()
    });
    let mut app = editor(map, &temp_maps("idle"));
    app.init_resource::<UiChanges>().add_systems(Last, count_ui_changes);
    act(&mut app, [A::SetTool(Tool::Select), A::Press(at(10, 10)), A::Release]);
    app.update();
    app.world_mut().resource_mut::<UiChanges>().0 = 0;
    for _ in 0..5 {
        app.update();
    }
    assert_eq!(app.world().resource::<UiChanges>().0, 0, "idle frames");
    // The counter does see changes.
    act(&mut app, [A::ToggleGrid]);
    assert!(app.world().resource::<UiChanges>().0 > 0);
}

/// Finding 5: the editor opens a file the game can't play, to fix it, whatever `--player` says (also see `cli.rs`).
#[test]
fn the_editor_opens_files_with_problems() {
    let dir = temp_maps("invalid");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("off_map.json");
    let mut map = scenarios::empty_map(16, 12);
    map.slots[1].start = TilePos::new(40, 3);
    files::save(&path, &map).unwrap();
    let mut app = editor_with(
        sc_sim::Scenario::new(scenarios::empty_map(16, 12)),
        EditorConfig {
            path: None,
            maps_dir: dir.clone(),
            script: Vec::new(),
        },
    );
    act(&mut app, [A::Open(path.clone())]);
    assert_eq!(*doc(&app), map);
    assert_warned(&app, "1 problem(s)");
    assert_eq!(
        problems(&app),
        vec![ScenarioProblem::StartOffMap {
            slot: SlotId(2),
            tile: TilePos::new(40, 3)
        }]
    );
    // A file over the largest map side is refused before anything is allocated, and the document stays.
    let huge = dir.join("huge.json");
    let mut json = serde_json::to_value(scenarios::empty_map(5, 4)).unwrap();
    json["terrain"] = serde_json::json!({ "tiles": [".".repeat(files::MAX_MAP_SIDE as usize + 1)] });
    std::fs::write(&huge, json.to_string()).unwrap();
    act(&mut app, [A::Open(huge)]);
    assert_eq!(status(&app).level, StatusLevel::Error);
    assert!(status(&app).message.contains("too big"), "{}", status(&app).message);
    assert_eq!(*doc(&app), map);
    // It can't be play-tested until it is fixed.
    act(&mut app, [A::PlayTest]);
    app.update();
    assert_eq!(*app.world().resource::<ClientMode>(), ClientMode::Editor);
    act(&mut app, [A::SetStart(SlotId(2), TilePos::new(12, 8))]);
    assert_eq!(problems(&app), vec![]);
    act(&mut app, [A::PlayTest]);
    app.update();
    assert_eq!(*app.world().resource::<ClientMode>(), ClientMode::PlayTest);
    // F5 alone ends it (Esc is the game's Cancel).
    tap(&mut app, KeyCode::Escape, Key::Escape);
    assert_eq!(*app.world().resource::<ClientMode>(), ClientMode::PlayTest);
    tap(&mut app, KeyCode::F5, Key::F5);
    assert_eq!(*app.world().resource::<ClientMode>(), ClientMode::Editor);
    std::fs::remove_dir_all(&dir).unwrap();
}
