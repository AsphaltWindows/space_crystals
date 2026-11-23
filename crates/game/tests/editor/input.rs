//! The editor's input through the shared input layer (`ARCHITECTURE.md > Client architecture > Input`): its keys are
//! bindings of the one table, its tools follow the layer's pointer, nothing reaches the map over the UI, and no key
//! fires while a text field has the keyboard. Driven through the layer's virtual keyboard and mouse.

use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input_focus::{FocusCause, InputFocus};
use bevy::prelude::*;
use sc_sim::{ObjectKind, TilePos, TilePresetId, scenarios};
use space_crystals::camera::{CameraView, CameraZoom, ViewportInsets, viewport_rect};
use space_crystals::editor::actions::EditorAction as A;
use space_crystals::editor::state::{EditorCursor, EditorUi, Selection};
use space_crystals::editor::tools::Tool;
use space_crystals::editor::ui::fields::TextField;
use space_crystals::input::{ActionState, Binding, Bindings, Chord, EditorInput, InputAction, Pointer};
use space_crystals::map_render::MapView;
use space_crystals::platform::SurfaceSize;

use super::{act, at, doc, editor, field, input, selection, status, tap, temp_maps, tile, tools, undo_depth};

fn pointer(app: &App) -> Pointer {
    *app.world().resource::<ActionState>().pointer()
}

fn hovered(app: &App) -> Option<Vec2> {
    app.world().resource::<EditorCursor>().0
}

fn zoom(app: &App) -> f32 {
    app.world().resource::<CameraZoom>().0
}

/// The middle of the editor's world view, in logical pixels.
fn view_middle(app: &App) -> Vec2 {
    let size = *app.world().resource::<SurfaceSize>();
    let insets = *app.world().resource::<ViewportInsets>();
    let (position, viewport) = viewport_rect(size, insets);
    (position.as_vec2() + viewport.as_vec2() / 2.0) / size.scale_factor
}

/// Puts the virtual mouse's cursor at `screen` and runs a frame.
fn move_to(app: &mut App, screen: Vec2) {
    input(app).cursor = Some(screen);
    app.update();
}

fn click(app: &mut App, button: MouseButton) {
    input(app).mouse.press(button);
    input(app).mouse.release(button);
    app.update();
}

#[test]
fn the_keys_reach_the_editor_through_its_bindings() {
    let mut app = editor(scenarios::empty_map(32, 24), &temp_maps("keys"));
    tap(&mut app, &[], KeyCode::KeyB);
    assert_eq!(tools(&app).tool, Tool::Paint);
    tap(&mut app, &[], KeyCode::BracketRight);
    assert_eq!(tools(&app).brush, 5);
    tap(&mut app, &[], KeyCode::KeyH);
    tap(&mut app, &[], KeyCode::Equal);
    assert_eq!((tools(&app).tool, tools(&app).elevation), (Tool::Elevation, 2));
    tap(&mut app, &[], KeyCode::KeyK);
    assert!(app.world().resource::<EditorUi>().slopes);
    // G is the grid toggle the editor shares with the game.
    let grid = |app: &App| app.world().resource::<MapView>().grid_visible;
    let shown = grid(&app);
    tap(&mut app, &[], KeyCode::KeyG);
    assert_eq!(grid(&app), !shown);
    // Ctrl + key is a command of its own.
    tap(&mut app, &[KeyCode::ControlLeft], KeyCode::KeyB);
    assert_eq!(tools(&app).tool, Tool::Elevation, "Ctrl + B is no Paint tool");

    // Undo and redo, with Ctrl and Ctrl + Shift; Z alone (a command slot in the game) is nothing here.
    act(
        &mut app,
        [A::SetPreset(TilePresetId::WATER), A::Press(at(3, 3)), A::Release],
    );
    assert_eq!(undo_depth(&app), 1);
    tap(&mut app, &[], KeyCode::KeyZ);
    assert_eq!(tile(&app, 3, 3).preset, TilePresetId::WATER);
    tap(&mut app, &[KeyCode::ControlLeft], KeyCode::KeyZ);
    assert_eq!(tile(&app, 3, 3).preset, TilePresetId::PLANE);
    tap(&mut app, &[KeyCode::ControlRight, KeyCode::ShiftLeft], KeyCode::KeyZ);
    assert_eq!(tile(&app, 3, 3).preset, TilePresetId::WATER);

    // F1 shows the help, Esc (the shared Cancel) hides it.
    tap(&mut app, &[], KeyCode::F1);
    assert!(app.world().resource::<EditorUi>().help);
    tap(&mut app, &[], KeyCode::Escape);
    assert!(!app.world().resource::<EditorUi>().help);

    // The editor's keys are entries of the one bindings table, rebound like the game's.
    {
        let mut bindings = app.world_mut().resource_mut::<Bindings>();
        let paint = InputAction::Editor(EditorInput::PaintTool);
        bindings.unbind(paint);
        bindings.bind(paint, Binding::Press(Chord::key(KeyCode::KeyP)));
    }
    tap(&mut app, &[], KeyCode::KeyV);
    tap(&mut app, &[], KeyCode::KeyB);
    assert_eq!(tools(&app).tool, Tool::Select, "B is unbound now");
    tap(&mut app, &[], KeyCode::KeyP);
    assert_eq!(tools(&app).tool, Tool::Paint);
}

/// The tools take their positions from the layer's pointer: the ground under the cursor, the press on the map and
/// its drag. Over the UI (injected: headless apps have no UI picking) nothing reaches the map.
#[test]
fn the_pointer_uses_the_tool_on_the_map_and_never_over_the_ui() {
    let mut app = editor(scenarios::empty_map(32, 24), &temp_maps("pointer"));
    let middle = view_middle(&app);
    move_to(&mut app, middle);
    let ground = pointer(&app).ground.expect("the middle of the view shows the map");
    assert_eq!(hovered(&app), Some(ground), "the brush follows the cursor");

    // A stroke: the press paints, the drag paints on (over a UI node on the view too: no press starts there, but a
    // drag goes on), the release makes it one undo step.
    act(&mut app, [A::SetPreset(TilePresetId::WATER)]);
    input(&mut app).mouse.press(MouseButton::Left);
    app.update();
    let start = TilePos::containing(ground);
    assert_eq!(tile(&app, start.x, start.y).preset, TilePresetId::WATER);
    assert_eq!(pointer(&app).press.map(|p| p.ground), Some(ground));
    input(&mut app).over_ui = true;
    move_to(&mut app, middle + Vec2::new(200.0, 0.0));
    assert_eq!(pointer(&app).ground, None, "over the UI");
    let end = TilePos::containing(pointer(&app).press.and_then(|p| p.drag_ground).unwrap());
    assert!(end.x > start.x + 3, "{end:?} lies east of {start:?}");
    assert_eq!(tile(&app, end.x, end.y).preset, TilePresetId::WATER);
    assert_eq!(undo_depth(&app), 0, "the stroke is still open");
    input(&mut app).over_ui = false;
    input(&mut app).mouse.release(MouseButton::Left);
    app.update();
    assert_eq!(pointer(&app).press, None);
    assert_eq!(undo_depth(&app), 1, "the stroke is one undo step");

    // The eyedropper (right button) picks the preset under the cursor.
    act(&mut app, [A::SetPreset(TilePresetId::CLIFF)]);
    move_to(&mut app, middle);
    click(&mut app, MouseButton::Right);
    assert_eq!(tools(&app).preset, TilePresetId::WATER, "{}", status(&app).message);

    // The wheel zooms at the cursor: the ground under it stays under it.
    input(&mut app).wheel = 1.0;
    app.update();
    assert_eq!(zoom(&app), 0.85);
    app.update();
    let after = pointer(&app).ground.unwrap();
    assert!(after.distance(ground) < 1.0, "{after} moved away from {ground}");

    // Over the UI the cursor is off the map: no hover, no press, no pick, no zoom.
    input(&mut app).over_ui = true;
    app.update();
    assert_eq!(hovered(&app), None);
    act(&mut app, [A::SetPreset(TilePresetId::CLIFF)]);
    click(&mut app, MouseButton::Left);
    click(&mut app, MouseButton::Right);
    input(&mut app).wheel = 1.0;
    app.update();
    assert_eq!(undo_depth(&app), 1, "no stroke");
    assert_eq!(tools(&app).preset, TilePresetId::CLIFF, "no pick");
    assert_eq!(zoom(&app), 0.85, "the wheel scrolls the panels instead");
    input(&mut app).over_ui = false;
    app.update();
    assert_eq!(hovered(&app), Some(after));

    // The middle button grabs the map, as in the game.
    let center = app.world().resource::<CameraView>().center;
    input(&mut app).mouse.press(MouseButton::Middle);
    app.update();
    move_to(&mut app, middle - Vec2::new(100.0, 0.0));
    input(&mut app).mouse.release(MouseButton::Middle);
    app.update();
    assert!(
        app.world().resource::<CameraView>().center.x > center.x,
        "the map follows the cursor west"
    );
    assert_eq!(undo_depth(&app), 1);
}

/// While a text field has the keyboard, keys type text and fire no binding, from the real keyboard or the virtual
/// one; nor does the Esc that gives the field up (Bevy's text input clears the focus on it).
#[test]
fn typed_keys_fire_no_binding() {
    let mut app = editor(scenarios::empty_map(32, 24), &temp_maps("typing"));
    app.init_resource::<InputFocus>();
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
    assert_eq!(selection(&app), Some(Selection::Spawn(0)));
    let name = field(&mut app, TextField::MapName);
    app.world_mut()
        .resource_mut::<InputFocus>()
        .set(name, FocusCause::Pressed);
    app.update();

    tap(&mut app, &[], KeyCode::KeyB);
    tap(&mut app, &[], KeyCode::Delete);
    for state in [ButtonState::Pressed, ButtonState::Released] {
        real_key(&mut app, KeyCode::F1, Key::F1, state);
        app.update();
    }
    assert_eq!(tools(&app).tool, Tool::Select);
    assert_eq!(doc(&app).spawns.len(), 1);
    assert!(!app.world().resource::<EditorUi>().help);

    real_key(&mut app, KeyCode::Escape, Key::Escape, ButtonState::Pressed);
    app.world_mut().resource_mut::<InputFocus>().clear();
    app.update();
    assert_eq!(
        selection(&app),
        Some(Selection::Spawn(0)),
        "the Esc gave up the field, it cancels nothing"
    );
    real_key(&mut app, KeyCode::Escape, Key::Escape, ButtonState::Released);
    app.update();

    // Without a field the keys work again.
    tap(&mut app, &[], KeyCode::Escape);
    assert_eq!(selection(&app), None);
}

/// A key of the real keyboard (Bevy's keyboard input).
fn real_key(app: &mut App, key_code: KeyCode, logical_key: Key, state: ButtonState) {
    app.world_mut().write_message(KeyboardInput {
        key_code,
        logical_key,
        state,
        text: None,
        repeat: false,
        window: Entity::PLACEHOLDER,
    });
}
