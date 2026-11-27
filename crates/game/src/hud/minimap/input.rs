//! The minimap's pointer (`systems/controls/hud.md > Minimap > Using it`): its presses and right-clicks, which belong
//! to it alone. The minimap is in the HUD Bar, off the world view, where the pointer is over the UI
//! (`crate::input::Pointer::over_ui`): no map press, click or box starts there, so a press on it never starts a box
//! selection, never deselects and never reaches the map's click handling (`crate::selection`, `crate::command_ui`).
//!
//! - **A left press** ([`MinimapPointer::press`]) goes down over the map area (`super::MinimapFrame::contains`) and
//!   lasts until its button goes up, wherever the cursor goes meanwhile ([`MinimapPress::drag_ground`], kept on the
//!   map). "Left-click or drag on the minimap: the camera centres on that point (follows the drag)"
//!   ([`follow_minimap_press`]): at once, as centring snaps (`core/camera.md > Window Mode > Centering`), and the
//!   camera keeps the view on the map as ever (`crate::camera`'s clamping).
//! - **While a target command waits** (`systems/controls/interface.md > AwaitingTarget`), the command flow takes the
//!   press as its targeting click on that ground point, at the press point and within the click allowance like a
//!   click on the map (`crate::command_ui`): it marks the press ([`MinimapPress::targets`]), which then moves no
//!   camera. AwaitingPlacement takes nothing from the minimap (the design: "Building placement (AwaitingPlacement)
//!   is not possible from the minimap"; the Director's reading: a left-click there centres the camera and never
//!   places), so there, as in every other state, the press moves the camera.
//! - **A right-click** ([`MinimapPointer::right_click`]), at its press like the map's: "the selection's right-click
//!   on that ground point, as a right-click on ground in the main view" (`systems/controls/interface.md >
//!   RightClickResolution`), which the command flow gives (`crate::command_ui`), with the modifiers held then (Shift
//!   queues).
//!
//! Nothing outside the game modes, and nothing while the results screen covers the HUD (`super::super::results`).

use bevy::prelude::*;

use super::MinimapFrame;
use crate::ClientMode;
use crate::camera::{CameraView, map_extent_su};
use crate::coords::drawn_position;
use crate::hud::results::has_left;
use crate::input::{ActionState, InputAction, Modifiers};
use crate::platform::SurfaceSize;
use crate::sim_driver::{LocalPlayer, SimRes};

/// A press of the Select action that went down on the minimap ([`MinimapPointer::press`]).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MinimapPress {
    /// Where it went down, in logical pixels.
    pub screen: Vec2,
    /// The ground point the minimap shows there: what a targeting click targets.
    pub ground: Vec2,
    /// The ground point under the cursor now, kept on the map (`MinimapFrame::ground_at`): where the camera follows
    /// the drag.
    pub drag_ground: Vec2,
    /// The farthest the pointer has been from where it went down, in logical pixels: the targeting click's allowance
    /// counts it (`InputTuning::targeting_click_allowance_px`).
    pub moved: f32,
    /// The command flow took it as a targeting click (`crate::command_ui`): it moves no camera.
    pub targets: bool,
    /// The ground point the camera last centred on for it.
    followed: Option<Vec2>,
}

/// A right-click on the minimap: the ContextCommand action pressed over its map area.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MinimapClick {
    /// The ground point the minimap shows there.
    pub ground: Vec2,
    /// The modifiers held as it was pressed: what they mean comes from the bindings, as for a click on the map.
    pub modifiers: Modifiers,
}

/// The minimap's presses and clicks this frame, as the map's pointer has its own (`crate::input::Pointer`).
#[derive(Resource, Copy, Clone, Debug, Default, PartialEq)]
pub struct MinimapPointer {
    /// The press of the Select action that went down on the minimap, from that frame until the frame its button goes
    /// up.
    pub press: Option<MinimapPress>,
    /// The press whose button went up this frame, as it ended: how far it moved included.
    pub released: Option<MinimapPress>,
    /// A right-click on the minimap this frame.
    pub right_click: Option<MinimapClick>,
}

/// Fills in [`MinimapPointer`] from the frame's actions, before the command flow reads them.
pub(super) fn track_minimap_pointer(
    mode: Res<ClientMode>,
    actions: Res<ActionState>,
    size: Res<SurfaceSize>,
    sim: Res<SimRes>,
    player: Res<LocalPlayer>,
    mut pointer: ResMut<MinimapPointer>,
) {
    let active = mode.is_game() && !has_left(&sim, player.0);
    let frame = MinimapFrame::on(*size, map_extent_su(sim.map()));
    let cursor = actions.cursor();
    let mut next = MinimapPointer {
        press: pointer.press,
        released: None,
        right_click: None,
    };
    if let Some(press) = &mut next.press
        && let Some(cursor) = cursor
    {
        press.moved = press.moved.max(cursor.distance(press.screen));
        press.drag_ground = frame.ground_at(cursor);
    }
    if !active || !actions.held(InputAction::Select) {
        next.released = next.press.take().filter(|_| active);
    }
    let over = cursor.filter(|&at| active && actions.pointer().over_ui && frame.contains(at));
    if let Some(at) = over {
        let ground = frame.ground_at(at);
        if actions.just_pressed(InputAction::Select) {
            next.press = Some(MinimapPress {
                screen: at,
                ground,
                drag_ground: ground,
                moved: 0.0,
                targets: false,
                followed: None,
            });
        }
        if actions.just_pressed(InputAction::ContextCommand) {
            next.right_click = Some(MinimapClick {
                ground,
                modifiers: actions.modifiers(),
            });
        }
    }
    pointer.set_if_neq(next);
}

/// `systems/controls/hud.md > Minimap > Using it`: "Left-click or drag on the minimap: the camera centres on that
/// point (follows the drag)", at once (`core/camera.md > Window Mode > Centering`): on the ground point under the
/// cursor, whenever it changes while the press lasts and in the frame of its release, unless the command flow took
/// the press as a targeting click. The view is centred on the point's [`drawn_position`], and kept on the map
/// afterwards as every view is (`crate::camera`).
pub(super) fn follow_minimap_press(
    sim: Res<SimRes>,
    mut pointer: ResMut<MinimapPointer>,
    mut view: ResMut<CameraView>,
) {
    let due = |press: &MinimapPress| !press.targets && press.followed != Some(press.drag_ground);
    if !pointer.press.iter().chain(&pointer.released).any(due) {
        return;
    }
    let pointer = &mut *pointer;
    for press in pointer.press.iter_mut().chain(pointer.released.iter_mut()) {
        if due(press) {
            view.center = drawn_position(sim.map(), press.drag_ground);
            press.followed = Some(press.drag_ground);
        }
    }
}
