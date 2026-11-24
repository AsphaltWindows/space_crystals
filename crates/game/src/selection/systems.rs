//! The selection's systems: the input layer's clicks, boxes and actions (`ARCHITECTURE.md > Client architecture >
//! Input`) become the selection rules (`super::rules`) on the local player's ControlState, and the sim's removals
//! leave every ControlState at once.

use std::collections::BTreeSet;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use sc_sim::{ObjectId, PlayerId, SimEvent};

use super::pick::selectable_by;
use super::rules::{ClickRoles, drawn_centroid};
use super::{ControlStates, LocalControl};
use crate::ClientMode;
use crate::camera::{CameraView, CameraZoom, ViewportInsets, view_rect};
use crate::command_ui::InterfaceState;
use crate::fog::{FogOfWar, Sight};
use crate::input::{ActionState, Bindings, InputAction, MapClick, SelectionBox};
use crate::platform::SurfaceSize;
use crate::sim_driver::{LocalPlayer, SimEventMessage, SimRes};

/// `> ControlState`: validated against the game state every tick. Removed objects (`SimEvent::Removed`, which
/// combat's deaths send too) leave every player's selection and control groups in the frame of their tick; so does
/// anything else that no longer exists, and anything that player may no longer select (`pick::selectable_by`):
/// an enemy that walks out of sight is deselected at once (a placeholder rule: the design doesn't say). A replaced
/// sim (`SimRes::generation`) starts every ControlState over, because its ids name other objects.
pub(super) fn sync_control_states(
    sim: Res<SimRes>,
    fog: Res<FogOfWar>,
    mode: Res<ClientMode>,
    mut events: MessageReader<SimEventMessage>,
    mut states: ResMut<ControlStates>,
) {
    let removed: BTreeSet<ObjectId> = events
        .read()
        .filter_map(|m| match m.event {
            SimEvent::Removed(id) => Some(id),
            _ => None,
        })
        .collect();
    if states.generation != Some(sim.generation()) {
        states.by_player.clear();
        states.generation = Some(sim.generation());
        return;
    }
    if removed.is_empty() && !sim.is_changed() && !fog.is_changed() && !mode.is_changed() {
        return;
    }
    let mut changed = false;
    for (&player, state) in &mut states.bypass_change_detection().by_player {
        let sight = Sight::new(&sim, player, fog.applies(*mode));
        changed |=
            state.retain(|id| !removed.contains(&id) && sim.object(id).is_some_and(|o| selectable_by(o, &sight)));
    }
    if changed {
        states.set_changed();
    }
}

/// The world view, for what is "on screen".
#[derive(SystemParam)]
pub(super) struct WorldView<'w> {
    size: Res<'w, SurfaceSize>,
    insets: Res<'w, ViewportInsets>,
    camera: Res<'w, CameraView>,
    zoom: Res<'w, CameraZoom>,
}

impl WorldView<'_> {
    /// The area the world view shows, in view space.
    fn visible(&self) -> Rect {
        view_rect(*self.size, *self.insets, self.camera.center, self.zoom.0)
    }
}

/// `> MapClickSelection` and `> BoxSelection`: the `Select` action's clicks and boxes on the map. While a target is
/// awaited (`> AwaitingTarget`), a left-click on the map is the target (`crate::command_ui`) and selects nothing, and
/// neither does a box (placeholder: the design doesn't say what a drag does then).
pub(super) fn select_on_map(
    mut clicks: MessageReader<MapClick>,
    mut boxes: MessageReader<SelectionBox>,
    bindings: Res<Bindings>,
    view: WorldView,
    mut control: LocalControl,
) {
    let clicks: Vec<MapClick> = clicks
        .read()
        .filter(|c| c.action == InputAction::Select)
        .copied()
        .collect();
    let boxes: Vec<SelectionBox> = boxes.read().copied().collect();
    if clicks.is_empty() && boxes.is_empty() || control.state().interface != InterfaceState::DefaultState {
        return;
    }
    let objects = control.objects();
    let visible = view.visible();
    control.update(|state| {
        for click in &clicks {
            let roles = ClickRoles::of(&bindings, click.modifiers);
            state.click(&objects, click.view, roles, click.double, visible);
        }
        for dragged in &boxes {
            state.select_box(&objects, dragged.rect);
        }
    });
}

/// `> ControlGroups > Operations` and `> GroupCycling`: the control group keys and Tab / Shift-Tab. Recall and
/// Center snaps the camera to the centroid of where the recalled objects are drawn (`core/camera.md > Panning >
/// Centering`); the camera then keeps the view on the map.
pub(super) fn control_group_keys(actions: Res<ActionState>, mut control: LocalControl, mut camera: ResMut<CameraView>) {
    use InputAction as A;
    let pressed: Vec<InputAction> = actions
        .pressed_actions()
        .filter(|a| {
            matches!(
                a,
                A::AssignGroup(_)
                    | A::AddToGroup(_)
                    | A::RecallGroup(_)
                    | A::RecallAndCenterGroup(_)
                    | A::NextGroup
                    | A::PreviousGroup
            )
        })
        .collect();
    if pressed.is_empty() {
        return;
    }
    let objects = control.objects();
    let center = control.update(|state| {
        let mut center = None;
        for action in pressed {
            match action {
                A::AssignGroup(group) => state.assign_group(group),
                A::AddToGroup(group) => {
                    state.add_to_group(group, &objects);
                }
                A::RecallGroup(group) => {
                    state.recall_group(group, &objects);
                }
                A::RecallAndCenterGroup(group) => {
                    if state.recall_group(group, &objects) {
                        center = drawn_centroid(state.selection.ids(), &objects);
                    }
                }
                A::NextGroup => {
                    state.cycle_groups(true);
                }
                A::PreviousGroup => {
                    state.cycle_groups(false);
                }
                _ => {}
            }
        }
        center
    });
    if let Some(center) = center {
        camera.center = center;
    }
}

/// The debug player switch: the local client acts as the next player of the game (in the sim's order, wrapping
/// around), whose ControlState and "own" objects are then the ones in use. Only while the game's debug commands
/// are on (`GameSetup::config`, `SimConfig::debug`).
pub(super) fn switch_player(actions: Res<ActionState>, sim: Res<SimRes>, mut player: ResMut<LocalPlayer>) {
    if !actions.just_pressed(InputAction::DebugSwitchPlayer) || !sim.config().debug {
        return;
    }
    let ids: Vec<PlayerId> = sim.players().iter().map(|p| p.id).collect();
    let next = match ids.iter().position(|&id| id == player.0) {
        Some(i) => ids[(i + 1) % ids.len()],
        None => match ids.first() {
            Some(&first) => first,
            None => return,
        },
    };
    if next != player.0 {
        player.0 = next;
    }
}
