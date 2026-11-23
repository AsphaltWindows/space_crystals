//! The pointer over the map: where the cursor points, whether it is over the UI, and the clicks and boxes of the
//! pointer actions (`ARCHITECTURE.md > Client architecture > Input`).
//!
//! [`InputAction::Select`] sends a [`MapClick`] when it is released, or a [`SelectionBox`] when the pointer moved
//! more than [`DRAG_THRESHOLD_PX`] while it was held. [`InputAction::ContextCommand`] sends a [`MapClick`] when it
//! is pressed. Neither starts over the UI. Positions on the map are view points (`crate::coords`), which stay on the
//! same spot of the map while the camera pans, and the ground points drawn there (`coords::ground_under`), as the
//! last frame showed them.

use bevy::ecs::system::SystemParam;
use bevy::picking::hover::HoverMap;
use bevy::picking::pointer::PointerId;
use bevy::prelude::*;

use super::{ActionState, InjectedInput, InputAction, Modifiers, RawInput};
use crate::camera::{CameraView, CameraZoom, ViewportInsets, cursor_offset, cursor_to_view, view_rect};
use crate::coords::ground_under;
use crate::platform::SurfaceSize;
use crate::sim_driver::SimRes;

/// A press that moves further than this from where it went down (logical pixels) is a drag, which makes
/// [`InputAction::Select`] a box instead of a click. Placeholder: unspecified, a few pixels like desktop drags.
pub const DRAG_THRESHOLD_PX: f32 = 4.0;
/// Two clicks of the same action within this many seconds (from press to press) are a double-click
/// (`systems/controls/selection_and_hud.md > MapClickSelection`). Placeholder: unspecified.
pub const DOUBLE_CLICK_SECONDS: f64 = 0.4;
/// ... and when the second is at most this far from the first (logical pixels). Placeholder: unspecified.
pub const DOUBLE_CLICK_DISTANCE_PX: f32 = 6.0;

/// Where the pointer is this frame ([`ActionState::pointer`]).
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Pointer {
    /// The cursor in logical pixels from the surface's top-left corner, while it is over the window or injected.
    pub screen: Option<Vec2>,
    /// The cursor is over the UI: over a UI node, or outside the world view (the HUD's bars, the editor's panels).
    /// No map click or box starts there.
    pub over_ui: bool,
    /// The view point under the cursor, while it is over the world view and not over the UI.
    pub view: Option<Vec2>,
    /// The ground point drawn at [`Pointer::view`] (`coords::ground_under`): what the cursor points at on the map.
    pub ground: Option<Vec2>,
    /// The box of a [`InputAction::Select`] drag in progress, in view space: from where it went down to the
    /// cursor, kept inside the world view.
    pub drag_box: Option<Rect>,
}

/// A click on the map, never over the UI.
#[derive(Message, Copy, Clone, Debug, PartialEq)]
pub struct MapClick {
    /// [`InputAction::Select`], sent when released unless it became a drag, or [`InputAction::ContextCommand`],
    /// sent when pressed.
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
    /// The second click of a double-click (`> MapClickSelection`): the same action, within
    /// [`DOUBLE_CLICK_SECONDS`] and [`DOUBLE_CLICK_DISTANCE_PX`] of the previous click. A third click starts over.
    pub double: bool,
}

/// A box dragged with [`InputAction::Select`], sent when released (`> BoxSelection`).
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

/// An [`InputAction::Select`] press that went down on the map.
#[derive(Copy, Clone, Debug)]
struct Press {
    screen: Vec2,
    view: Vec2,
    ground: Vec2,
    /// Seconds of real time.
    time: f64,
    dragging: bool,
}

/// The pointer's memory between frames.
#[derive(Default, Debug)]
pub(super) struct PointerTracking {
    press: Option<Press>,
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
    let over_ui = screen.is_some_and(|s| over_node || world.view_at(s).is_none());
    let view = screen.filter(|_| !over_ui).and_then(|s| world.view_at(s));
    let ground = view.map(|v| ground_under(world.sim.map(), v));
    let modifiers = state.modifiers;

    if state.just_pressed(InputAction::Select) {
        tracking.press = match (screen, view, ground) {
            (Some(screen), Some(view), Some(ground)) => Some(Press {
                screen,
                view,
                ground,
                time: now,
                dragging: false,
            }),
            _ => None,
        };
    }
    let last_screen = tracking.last_screen;
    if let (Some(press), Some(cursor)) = (&mut tracking.press, last_screen)
        && cursor.distance(press.screen) > DRAG_THRESHOLD_PX
    {
        press.dragging = true;
    }
    let drag_end = last_screen.and_then(|s| world.clamped_view_at(s));
    if !state.held(InputAction::Select)
        && let Some(press) = tracking.press.take()
    {
        if press.dragging {
            tracking.last_click = None;
            if let Some(end) = drag_end {
                boxes.write(SelectionBox {
                    rect: Rect::from_corners(press.view, end),
                    modifiers,
                });
            }
        } else {
            let double = tracking.click(InputAction::Select, press.screen, press.time);
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
    if state.just_pressed(InputAction::ContextCommand)
        && let (Some(screen), Some(view), Some(ground)) = (screen, view, ground)
    {
        let double = tracking.click(InputAction::ContextCommand, screen, now);
        clicks.write(MapClick {
            action: InputAction::ContextCommand,
            screen,
            view,
            ground,
            modifiers,
            double,
        });
    }

    let drag_box = tracking
        .press
        .filter(|p| p.dragging)
        .and_then(|p| drag_end.map(|end| Rect::from_corners(p.view, end)));
    state.pointer = Pointer {
        screen,
        over_ui,
        view,
        ground,
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
