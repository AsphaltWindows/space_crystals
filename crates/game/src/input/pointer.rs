//! The pointer over the map: where the cursor points, whether it is over the UI, and the presses, clicks and boxes
//! of the pointer actions (`ARCHITECTURE.md > Client architecture > Input`).
//!
//! The pointer follows a press of a primary pointer action that goes down on the map until its release
//! ([`Pointer::press`]): [`InputAction::Select`] in the game, [`EditorInput::UseTool`] in the editor. A
//! [`InputAction::Select`] press sends a [`MapClick`] when it is released, or a [`SelectionBox`] when the pointer
//! moved more than [`DRAG_THRESHOLD_PX`] while it was held; in the frame of its release the press, with how far it
//! moved, is [`Pointer::released`]. [`InputAction::ContextCommand`] and the editor's
//! [`EditorInput::Eyedropper`] send a [`MapClick`] when they are pressed. None of them starts over the UI.
//! Positions on the map are view points (`crate::coords`), which stay on the same spot of the map while the camera
//! pans, and the ground points drawn there (`coords::ground_under`), as the last frame showed them.

use bevy::ecs::system::SystemParam;
use bevy::picking::hover::HoverMap;
use bevy::picking::pointer::PointerId;
use bevy::prelude::*;

use super::{ActionState, EditorInput, InjectedInput, InputAction, Modifiers, RawInput};
use crate::camera::{CameraView, CameraZoom, ViewportInsets, cursor_offset, cursor_to_view, view_rect};
use crate::coords::ground_under;
use crate::platform::SurfaceSize;
use crate::sim_driver::SimRes;

pub use crate::tuning::input::{DOUBLE_CLICK_DISTANCE_PX, DOUBLE_CLICK_SECONDS, DRAG_THRESHOLD_PX};

/// The primary pointer actions, the game's and the editor's: the pointer follows a press of one on the map until
/// its release ([`Pointer::press`]).
const PRESS_ACTIONS: [InputAction; 2] = [InputAction::Select, InputAction::Editor(EditorInput::UseTool)];
/// The pointer actions that click on the map when pressed.
const CLICK_ON_PRESS_ACTIONS: [InputAction; 2] = [
    InputAction::ContextCommand,
    InputAction::Editor(EditorInput::Eyedropper),
];

/// Where the pointer is this frame ([`ActionState::pointer`]).
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Pointer {
    /// The cursor in logical pixels from the surface's top-left corner, while it is over the window or injected.
    pub screen: Option<Vec2>,
    /// The cursor is over the UI: over a UI node, or outside the world view (the HUD's bars, the editor's panels).
    /// No map press, click or box starts there. The one over-UI test of the game and the editor.
    pub over_ui: bool,
    /// The view point under the cursor, while it is over the world view and not over the UI.
    pub view: Option<Vec2>,
    /// The ground point drawn at [`Pointer::view`] (`coords::ground_under`): what the cursor points at on the map.
    pub ground: Option<Vec2>,
    /// The press of a primary pointer action that went down on the map, from that frame until the frame its button
    /// goes up (or its input context ends): the drag state. The editor's tools stroke and drag from it.
    pub press: Option<MapPress>,
    /// The press whose button went up this frame, in its input context, as it ended: how far it moved included
    /// ([`MapPress::moved`]). For a rule of its own about clicks and drags: the command flow's larger click allowance
    /// while a target is awaited (`crate::command_ui::TARGETING_CLICK_ALLOWANCE_PX`, design change 0073).
    pub released: Option<MapPress>,
    /// The box of a [`InputAction::Select`] drag in progress, in view space: from where it went down to the
    /// cursor, kept inside the world view.
    pub drag_box: Option<Rect>,
}

/// A press of a primary pointer action ([`InputAction::Select`], [`EditorInput::UseTool`]) that went down on the
/// map and is still held ([`Pointer::press`]).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MapPress {
    pub action: InputAction,
    /// Where it went down, in logical pixels.
    pub screen: Vec2,
    /// The view point there.
    pub view: Vec2,
    /// The ground point drawn there.
    pub ground: Vec2,
    /// The ground point drawn under the cursor now, while the cursor is over the world view: over a UI node there
    /// too, because no press starts over the UI but a drag goes on there. The editor's tools drag to it.
    pub drag_ground: Option<Vec2>,
    /// The farthest the pointer has been from where it went down, in logical pixels.
    pub moved: f32,
    /// The pointer moved further than [`DRAG_THRESHOLD_PX`] from there ([`MapPress::moved`]): a
    /// [`InputAction::Select`] press is a box now, not a click.
    pub dragging: bool,
}

/// A click on the map, never over the UI.
#[derive(Message, Copy, Clone, Debug, PartialEq)]
pub struct MapClick {
    /// [`InputAction::Select`], sent when released unless it became a drag, or [`InputAction::ContextCommand`] or
    /// the editor's [`EditorInput::Eyedropper`], sent when pressed.
    pub action: InputAction,
    /// Where the button went down, in logical pixels.
    pub screen: Vec2,
    /// The view point there.
    pub view: Vec2,
    /// The ground point drawn there: the click's target on the map.
    pub ground: Vec2,
    /// The modifiers held when the click was sent. What they mean comes from the bindings
    /// (`Bindings::holds(InputAction::Queue, modifiers)`), as in that frame's [`ActionState`].
    pub modifiers: Modifiers,
    /// The second click of a double-click (`systems/controls/selection.md > MapClickSelection`): the same
    /// action, within [`DOUBLE_CLICK_SECONDS`] and [`DOUBLE_CLICK_DISTANCE_PX`] of the previous click. A third click
    /// starts over.
    pub double: bool,
}

/// A box dragged with [`InputAction::Select`], sent when released (`systems/controls/selection.md >
/// BoxSelection`).
#[derive(Message, Copy, Clone, Debug, PartialEq)]
pub struct SelectionBox {
    /// In view space, from where the drag went down to where it was released, kept inside the world view. Objects
    /// are drawn at their terrain height, so box selection tests where they are drawn (`coords::drawn_position`).
    pub rect: Rect,
    /// The modifiers held when it was released.
    pub modifiers: Modifiers,
}

/// What the pointer needs to know about the view: the surface, the world view's place on it, and the map.
#[derive(SystemParam)]
pub(super) struct WorldView<'w> {
    size: Res<'w, SurfaceSize>,
    insets: Res<'w, ViewportInsets>,
    camera: Option<Res<'w, CameraView>>,
    zoom: Res<'w, CameraZoom>,
    sim: Res<'w, SimRes>,
}

impl WorldView<'_> {
    /// The view point under `screen`, if it is over the world view.
    fn view_at(&self, screen: Vec2) -> Option<Vec2> {
        let center = self.camera.as_ref()?.center;
        cursor_to_view(screen, *self.size, *self.insets, center, self.zoom.0)
    }

    /// The view point under `screen`, or the nearest one inside the world view when it is elsewhere.
    fn clamped_view_at(&self, screen: Vec2) -> Option<Vec2> {
        let center = self.camera.as_ref()?.center;
        let visible = view_rect(*self.size, *self.insets, center, self.zoom.0);
        cursor_offset(screen, *self.size, *self.insets, self.zoom.0).map(|o| visible.clamp_point(center + o))
    }
}

/// Whether the real mouse is over a UI node, by Bevy's UI picking.
#[derive(SystemParam)]
pub(super) struct UiHover<'w, 's> {
    hover: Option<Res<'w, HoverMap>>,
    ui_nodes: Query<'w, 's, (), With<Node>>,
}

impl UiHover<'_, '_> {
    fn over_ui_node(&self) -> bool {
        let hits = self.hover.as_ref().and_then(|hover| hover.get(&PointerId::Mouse));
        hits.is_some_and(|hits| hits.keys().any(|e| self.ui_nodes.contains(*e)))
    }
}

/// The pointer's memory between frames.
#[derive(Default, Debug)]
pub(super) struct PointerTracking {
    /// The press on the map, and when it went down (seconds of real time).
    press: Option<(MapPress, f64)>,
    /// The latest click, for double-clicks: its action, where and when (seconds) it went down.
    last_click: Option<(InputAction, Vec2, f64)>,
    /// Where the cursor was last seen, for drags that leave the window.
    last_screen: Option<Vec2>,
}

impl PointerTracking {
    /// Records a click and says whether it is the second of a double-click.
    fn click(&mut self, action: InputAction, screen: Vec2, time: f64) -> bool {
        let double = self.last_click.is_some_and(|(a, s, t)| {
            a == action && time - t <= DOUBLE_CLICK_SECONDS && s.distance(screen) <= DOUBLE_CLICK_DISTANCE_PX
        });
        self.last_click = if double { None } else { Some((action, screen, time)) };
        double
    }
}

/// Fills in [`ActionState::pointer`] and sends this frame's clicks and boxes, after the actions are read.
#[expect(clippy::too_many_arguments, reason = "a system's parameters")]
pub(super) fn track_pointer(
    raw: Res<RawInput>,
    injected: Res<InjectedInput>,
    world: WorldView,
    hover: UiHover,
    time: Res<Time<Real>>,
    mut state: ResMut<ActionState>,
    mut tracking: Local<PointerTracking>,
    mut messages: (MessageWriter<MapClick>, MessageWriter<SelectionBox>),
) {
    let (clicks, boxes) = &mut messages;
    let now = time.elapsed_secs_f64();
    let screen = state.cursor;
    if screen.is_some() {
        tracking.last_screen = screen;
    }
    // The hover map follows the real mouse, not an injected cursor.
    let over_node = if injected.cursor.is_some() {
        injected.over_ui
    } else {
        raw.enabled && hover.over_ui_node()
    };
    let in_view = screen.and_then(|s| world.view_at(s));
    let over_ui = screen.is_some() && (over_node || in_view.is_none());
    let view = in_view.filter(|_| !over_ui);
    let ground_at = |view| ground_under(world.sim.map(), view);
    let ground = view.map(ground_at);
    let modifiers = state.modifiers;

    if let Some(action) = PRESS_ACTIONS.into_iter().find(|a| state.just_pressed(*a)) {
        tracking.press = match (screen, view, ground) {
            (Some(screen), Some(view), Some(ground)) => Some((
                MapPress {
                    action,
                    screen,
                    view,
                    ground,
                    drag_ground: Some(ground),
                    moved: 0.0,
                    dragging: false,
                },
                now,
            )),
            _ => None,
        };
    }
    let last_screen = tracking.last_screen;
    if let Some((press, _)) = &mut tracking.press {
        press.drag_ground = in_view.map(ground_at);
        if let Some(cursor) = last_screen {
            press.moved = press.moved.max(cursor.distance(press.screen));
        }
        press.dragging = press.moved > DRAG_THRESHOLD_PX;
    }
    let drag_end = last_screen.and_then(|s| world.clamped_view_at(s));
    let mut released = None;
    if let Some((press, pressed_at)) = tracking.press
        && !state.held(press.action)
    {
        tracking.press = None;
        released = press.action.works_in(state.context).then_some(press);
        // A Select press ends in a click or a box, unless it ended because the input context changed.
        let selected = press.action == InputAction::Select && press.action.works_in(state.context);
        if selected && press.dragging {
            tracking.last_click = None;
            if let Some(end) = drag_end {
                boxes.write(SelectionBox {
                    rect: Rect::from_corners(press.view, end),
                    modifiers,
                });
            }
        } else if selected {
            let double = tracking.click(InputAction::Select, press.screen, pressed_at);
            clicks.write(MapClick {
                action: InputAction::Select,
                screen: press.screen,
                view: press.view,
                ground: press.ground,
                modifiers,
                double,
            });
        }
    }
    for action in CLICK_ON_PRESS_ACTIONS {
        if state.just_pressed(action)
            && let (Some(screen), Some(view), Some(ground)) = (screen, view, ground)
        {
            let double = tracking.click(action, screen, now);
            clicks.write(MapClick {
                action,
                screen,
                view,
                ground,
                modifiers,
                double,
            });
        }
    }

    let press = tracking.press.map(|(press, _)| press);
    let drag_box = press
        .filter(|p| p.action == InputAction::Select && p.dragging)
        .and_then(|p| drag_end.map(|end| Rect::from_corners(p.view, end)));
    state.pointer = Pointer {
        screen,
        over_ui,
        view,
        ground,
        press,
        released,
        drag_box,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn double_clicks_need_the_same_action_close_and_quick() {
        let mut t = PointerTracking::default();
        let at = Vec2::new(100.0, 100.0);
        let near = at + Vec2::new(DOUBLE_CLICK_DISTANCE_PX, 0.0);
        assert!(!t.click(InputAction::Select, at, 1.0));
        assert!(t.click(InputAction::Select, near, 1.0 + DOUBLE_CLICK_SECONDS - 0.01));
        // The third click starts over.
        assert!(!t.click(InputAction::Select, at, 1.5));
        // Too slow, too far, or another action.
        assert!(!t.click(InputAction::Select, at, 1.5 + DOUBLE_CLICK_SECONDS + 0.01));
        assert!(!t.click(InputAction::Select, at + Vec2::new(0.0, 7.0), 2.0));
        assert!(!t.click(InputAction::ContextCommand, at + Vec2::new(0.0, 7.0), 2.1));
        assert!(t.click(InputAction::ContextCommand, at + Vec2::new(0.0, 7.0), 2.2));
    }
}
