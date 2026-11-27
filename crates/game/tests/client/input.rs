//! The input layer (`ARCHITECTURE.md > Client architecture > Input`), driven through the virtual keyboard and
//! mouse: control group keys, hotkeys and modifiers, rebinding, panning, map clicks, the selection box, and
//! clicks over the UI.

use bevy::prelude::*;

use sc_sim::{TilePos, scenarios};
use space_crystals::camera::{CameraView, GrabPan, ViewportInsets, view_rect};
use space_crystals::coords::{drawn_position, ground_under};
use space_crystals::input::{
    Binding, Bindings, Chord, CommandSlot, ControlGroup, InputAction, MapClick, Modifiers, SelectionBox,
};
use space_crystals::map_render::GridOverlay;
use space_crystals::platform::SurfaceSize;
use space_crystals::sim_driver::SimRes;

use super::{
    actions, camera_center, centered_scenario, drain, headless_app, input, plateau_scenario, set_cursor,
    small_scenario, updates, view_at, view_middle,
};

fn group(n: u8) -> ControlGroup {
    ControlGroup::new(n).unwrap()
}

fn slot(row: u8, column: u8) -> InputAction {
    InputAction::CommandSlot(CommandSlot::new(row, column).unwrap())
}

/// Taps `key` while the modifier keys `modifiers` are down, for one frame, and returns the actions the tap fired
/// (without the modifier actions, which the modifiers hold).
fn tap(app: &mut App, modifiers: &[KeyCode], key: KeyCode) -> Vec<InputAction> {
    for &m in modifiers {
        input(app).keys.press(m);
    }
    input(app).tap_key(key);
    app.update();
    for &m in modifiers {
        input(app).keys.release(m);
    }
    let roles = [
        InputAction::Queue,
        InputAction::ToggleSelection,
        InputAction::SelectSameType,
        InputAction::CenterOnObject,
    ];
    actions(app).pressed_actions().filter(|a| !roles.contains(a)).collect()
}

/// `systems/controls/selection.md > ControlGroups > Default keys`: a number recalls its group, Ctrl assigns,
/// Shift adds, and pressing the number twice quickly recalls and centers.
#[test]
fn control_group_keys_resolve_by_their_modifiers_and_double_taps() {
    use KeyCode::{ControlRight, Digit1, Digit2, ShiftLeft};
    let mut app = headless_app(small_scenario());
    updates(&mut app, 2);
    assert_eq!(tap(&mut app, &[], Digit1), [InputAction::RecallGroup(group(1))]);
    // The next press of 1, a frame (1/16 s) later, completes a double tap; a third one starts over.
    assert_eq!(
        tap(&mut app, &[], Digit1),
        [InputAction::RecallAndCenterGroup(group(1))]
    );
    assert_eq!(tap(&mut app, &[], Digit1), [InputAction::RecallGroup(group(1))]);
    // Too slow (over half a second later), or another key in between: single presses.
    updates(&mut app, 8);
    assert_eq!(tap(&mut app, &[], Digit1), [InputAction::RecallGroup(group(1))]);
    assert_eq!(tap(&mut app, &[], Digit2), [InputAction::RecallGroup(group(2))]);
    assert_eq!(tap(&mut app, &[], Digit1), [InputAction::RecallGroup(group(1))]);
    // Either Ctrl assigns and either Shift adds, also twice in a row: a chord with more modifiers beats a double tap.
    assert_eq!(
        tap(&mut app, &[ControlRight], Digit2),
        [InputAction::AssignGroup(group(2))]
    );
    assert_eq!(tap(&mut app, &[ShiftLeft], Digit2), [InputAction::AddToGroup(group(2))]);
    assert_eq!(tap(&mut app, &[ShiftLeft], Digit2), [InputAction::AddToGroup(group(2))]);
}

/// The command panel's hotkeys (`systems/controls/interface.md > CommandPanel > Grid Layout`,
/// `systems/controls/interface.md > Standard Slot Assignments`), group cycling and cancel. Shift + a hotkey
/// still presses the slot, with the queue modifier held (`systems/controls/commands.md > Unit Command`).
#[test]
fn hotkeys_fire_their_actions_and_modifiers_hold_their_roles() {
    use KeyCode::*;
    let mut app = headless_app(small_scenario());
    updates(&mut app, 2);
    assert_eq!(tap(&mut app, &[], KeyQ), [slot(0, 0)]);
    assert_eq!(tap(&mut app, &[], KeyF), [slot(1, 3)]);
    assert_eq!(tap(&mut app, &[], KeyZ), [InputAction::CommandSlot(CommandSlot::BACK)]);
    assert_eq!(
        tap(&mut app, &[], KeyX),
        [InputAction::CommandSlot(CommandSlot::CANCEL_PRODUCTION)]
    );
    assert_eq!(
        tap(&mut app, &[], KeyC),
        [InputAction::CommandSlot(CommandSlot::RALLY_POINT)]
    );
    assert_eq!(tap(&mut app, &[], Tab), [InputAction::NextGroup]);
    assert_eq!(tap(&mut app, &[ShiftRight], Tab), [InputAction::PreviousGroup]);
    assert_eq!(tap(&mut app, &[], Escape), [InputAction::Cancel]);
    assert_eq!(tap(&mut app, &[], KeyP), [InputAction::TogglePaths]);
    assert!(tap(&mut app, &[], KeyO).is_empty(), "O is not bound");

    // Shift + W: slot W, queued. The slot is held while its key is down, the roles while their modifier is.
    input(&mut app).keys.press(ShiftLeft);
    input(&mut app).keys.press(KeyW);
    app.update();
    let state = actions(&app);
    assert!(state.just_pressed(slot(0, 1)));
    assert!(state.held(InputAction::Queue) && state.held(InputAction::ToggleSelection));
    assert!(!state.held(InputAction::SelectSameType));
    assert_eq!(state.modifiers(), Modifiers::SHIFT);
    input(&mut app).keys.release(ShiftLeft);
    app.update();
    let state = actions(&app);
    assert!(state.held(slot(0, 1)) && !state.just_pressed(slot(0, 1)));
    assert!(state.just_released(InputAction::Queue) && !state.held(InputAction::Queue));
    input(&mut app).keys.release(KeyW);
    app.update();
    assert!(actions(&app).just_released(slot(0, 1)));
}

/// Gameplay asks for actions, never for keys: rebinding one changes the table and nothing else.
#[test]
fn a_rebound_action_follows_its_new_binding() {
    let mut app = headless_app(small_scenario());
    updates(&mut app, 2);
    {
        let mut bindings = app.world_mut().resource_mut::<Bindings>();
        bindings.unbind(InputAction::ToggleGrid);
        bindings.bind(
            InputAction::ToggleGrid,
            Binding::Press(Chord::key(KeyCode::KeyH).with(Modifiers::ALT)),
        );
    }
    let grid_shown = |app: &mut App| {
        let mut query = app.world_mut().query_filtered::<&Visibility, With<GridOverlay>>();
        *query.single(app.world()).unwrap() != Visibility::Hidden
    };
    assert!(tap(&mut app, &[], KeyCode::KeyG).is_empty());
    assert!(tap(&mut app, &[], KeyCode::KeyH).is_empty(), "H needs Alt now");
    assert!(!grid_shown(&mut app));
    assert_eq!(
        tap(&mut app, &[KeyCode::AltLeft], KeyCode::KeyH),
        [InputAction::ToggleGrid]
    );
    assert!(grid_shown(&mut app));
}

/// `core/camera.md > Panning`: the arrow keys and the middle button pan through their bindings; a modifier pressed
/// meanwhile doesn't stop them.
#[test]
fn arrow_keys_and_the_middle_button_pan_through_their_bindings() {
    let mut app = headless_app(centered_scenario());
    updates(&mut app, 2);
    let start = camera_center(&app);
    input(&mut app).keys.press(KeyCode::ArrowLeft);
    updates(&mut app, 2);
    let moved = camera_center(&app);
    assert!(
        moved.x < start.x && moved.y == start.y,
        "{moved} should be west of {start}"
    );
    input(&mut app).keys.press(KeyCode::ControlLeft);
    app.update();
    assert!(camera_center(&app).x < moved.x);
    input(&mut app).keys.release(KeyCode::ArrowLeft);
    input(&mut app).keys.release(KeyCode::ControlLeft);
    app.update();
    let stopped = camera_center(&app);
    updates(&mut app, 2);
    assert_eq!(camera_center(&app), stopped);

    let grab_at = view_middle(&app);
    set_cursor(&mut app, grab_at);
    input(&mut app).mouse.press(MouseButton::Middle);
    app.update();
    let grabbed = view_at(&app, grab_at);
    assert_eq!(app.world().resource::<GrabPan>().grabbed, Some(grabbed));
    let moved_to = grab_at + Vec2::new(-90.0, 40.0);
    set_cursor(&mut app, moved_to);
    app.update();
    assert_ne!(camera_center(&app), stopped);
    assert!((view_at(&app, moved_to) - grabbed).length() < 1e-3);
    input(&mut app).mouse.release(MouseButton::Middle);
    app.update();
    assert_eq!(app.world().resource::<GrabPan>().grabbed, None);
}

/// `systems/controls/selection.md > MapClickSelection`: a left click on the map is sent when released, with the
/// view point and the ground drawn there (they differ on raised ground), the modifiers, and whether it completes a
/// double-click. A right click (the context command) is sent when pressed.
#[test]
fn clicks_on_the_map_carry_the_ground_point_the_modifiers_and_double_clicks() {
    let mut app = headless_app(plateau_scenario());
    updates(&mut app, 2);
    // Look at the middle of the plateau.
    let map = app.world().resource::<SimRes>().map().clone();
    let patch = TilePos::new(16, 12).center();
    app.world_mut().resource_mut::<CameraView>().center = drawn_position(&map, patch);
    let at = view_middle(&app);
    set_cursor(&mut app, at);
    app.update();
    let view = view_at(&app, at);
    let pointer = *actions(&app).pointer();
    assert_eq!(pointer.screen, Some(at));
    assert!(!pointer.over_ui);
    assert_eq!(pointer.view, Some(view));
    assert_eq!(pointer.ground, Some(ground_under(&map, view)));
    let ground = pointer.ground.unwrap();
    assert!(
        (ground - patch).length() < 0.01,
        "{ground} should be the plateau's {patch}"
    );
    assert_ne!(ground, view);

    input(&mut app).mouse.press(MouseButton::Left);
    app.update();
    assert!(
        drain::<MapClick>(&mut app).is_empty(),
        "a left click is sent when released"
    );
    input(&mut app).mouse.release(MouseButton::Left);
    app.update();
    let click = MapClick {
        action: InputAction::Select,
        screen: at,
        view,
        ground,
        modifiers: Modifiers::NONE,
        double: false,
    };
    assert_eq!(drain::<MapClick>(&mut app), [click]);

    // Another click a little to the side, quickly, with Shift held: a double-click, whose Shift means "toggle".
    let near = at + Vec2::new(3.0, -2.0);
    set_cursor(&mut app, near);
    input(&mut app).keys.press(KeyCode::ShiftLeft);
    input(&mut app).mouse.press(MouseButton::Left);
    input(&mut app).mouse.release(MouseButton::Left);
    updates(&mut app, 2);
    let clicks = drain::<MapClick>(&mut app);
    assert_eq!(clicks.len(), 1);
    assert_eq!((clicks[0].screen, clicks[0].modifiers), (near, Modifiers::SHIFT));
    assert!(clicks[0].double);
    let bindings = app.world().resource::<Bindings>();
    assert!(bindings.holds(InputAction::ToggleSelection, clicks[0].modifiers));
    assert!(!bindings.holds(InputAction::SelectSameType, clicks[0].modifiers));
    input(&mut app).keys.release(KeyCode::ShiftLeft);

    input(&mut app).mouse.press(MouseButton::Right);
    app.update();
    let clicks = drain::<MapClick>(&mut app);
    assert_eq!(clicks.len(), 1);
    assert_eq!(clicks[0].action, InputAction::ContextCommand);
    assert_eq!(clicks[0].ground, ground_under(&map, view_at(&app, near)));
    assert!(!clicks[0].double, "the previous click was another action's");
    input(&mut app).mouse.release(MouseButton::Right);
    app.update();
    assert!(drain::<MapClick>(&mut app).is_empty());
}

/// `systems/controls/selection.md > BoxSelection`: a left press that moves further than the drag threshold is a
/// box in view space, shown while dragging and sent when released, kept inside the world view.
#[test]
fn dragging_the_select_button_makes_a_box() {
    let mut app = headless_app(scenarios::empty(64, 48));
    updates(&mut app, 2);
    let start = view_middle(&app);
    set_cursor(&mut app, start);
    input(&mut app).mouse.press(MouseButton::Left);
    app.update();
    set_cursor(
        &mut app,
        start + Vec2::new(space_crystals::tuning::input::BUILTIN.drag_threshold_px, 0.0),
    );
    app.update();
    assert_eq!(
        actions(&app).pointer().drag_box,
        None,
        "not further than the threshold yet"
    );
    let end = start + Vec2::new(-120.0, 80.0);
    set_cursor(&mut app, end);
    app.update();
    let rect = Rect::from_corners(view_at(&app, start), view_at(&app, end));
    assert_eq!(actions(&app).pointer().drag_box, Some(rect));
    input(&mut app).mouse.release(MouseButton::Left);
    app.update();
    let dragged = SelectionBox {
        rect,
        modifiers: Modifiers::NONE,
    };
    assert_eq!(drain::<SelectionBox>(&mut app), [dragged]);
    assert!(drain::<MapClick>(&mut app).is_empty(), "a drag is no click");
    assert_eq!(actions(&app).pointer().drag_box, None);

    // A drag into the HUD's bottom bar ends at the world view's bottom edge.
    set_cursor(&mut app, start);
    input(&mut app).mouse.press(MouseButton::Left);
    app.update();
    let size = *app.world().resource::<SurfaceSize>();
    let insets = *app.world().resource::<ViewportInsets>();
    set_cursor(&mut app, Vec2::new(start.x + 50.0, size.logical().y - 2.0));
    input(&mut app).keys.press(KeyCode::ControlLeft);
    app.update();
    assert!(actions(&app).pointer().over_ui);
    input(&mut app).mouse.release(MouseButton::Left);
    app.update();
    let boxes = drain::<SelectionBox>(&mut app);
    let visible = view_rect(size, insets, camera_center(&app), 1.0);
    assert_eq!(boxes.len(), 1);
    assert_eq!(boxes[0].rect.min, view_at(&app, start));
    assert_eq!(boxes[0].rect.max.y, visible.max.y);
    assert_eq!(boxes[0].modifiers, Modifiers::CTRL);
}

/// No map click or box starts over the UI: over a UI node (injected: headless apps have no UI picking), or over the
/// HUD's bars, outside the world view.
#[test]
fn clicks_over_the_ui_never_reach_the_map() {
    let mut app = headless_app(small_scenario());
    updates(&mut app, 2);
    let middle = view_middle(&app);
    let click_both = |app: &mut App| {
        for button in [MouseButton::Left, MouseButton::Right] {
            input(app).mouse.press(button);
            input(app).mouse.release(button);
        }
        updates(app, 2);
        drain::<MapClick>(app).len()
    };
    set_cursor(&mut app, middle);
    input(&mut app).over_ui = true;
    assert_eq!(click_both(&mut app), 0);
    let pointer = *actions(&app).pointer();
    assert_eq!(pointer.screen, Some(middle));
    assert!(pointer.over_ui && pointer.view.is_none() && pointer.ground.is_none());

    input(&mut app).over_ui = false;
    set_cursor(&mut app, Vec2::new(middle.x, 2.0)); // the top bar
    assert_eq!(click_both(&mut app), 0);
    assert!(actions(&app).pointer().over_ui);

    set_cursor(&mut app, middle);
    assert_eq!(click_both(&mut app), 2);
    assert!(!actions(&app).pointer().over_ui);
}
