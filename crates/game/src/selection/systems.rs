//! The selection's systems: the input layer's clicks, boxes and actions (`docs/architecture/client-architecture-input-and-selection.md > Client architecture >
//! Input`) become the selection rules (`super::rules`) on the local player's ControlState, and the sim's removals
//! leave every ControlState at once, each player's those they perceive (`crate::viewer_events`).

use std::collections::BTreeSet;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use sc_sim::{ObjectId, PlayerId, SimEvent};

use super::pick::{selectable_by, selectable_id};
use super::rules::{ClickRoles, drawn_centroid};
use super::{ControlStates, GroupType, LocalControl};
use crate::ClientMode;
use crate::camera::{CameraView, CameraZoom, ViewportInsets, view_rect};
use crate::fog::{FogOfWar, Sight};
use crate::input::{ActionState, Bindings, InputAction, MapClick, SelectionBox};
use crate::platform::SurfaceSize;
use crate::replay::Playback;
use crate::sim_driver::{LocalPlayer, PendingCommands, SimRes};
use crate::viewer_events::ViewerEvents;

/// `systems/controls/selection.md > ControlState`: validated against the game state every tick. Removed objects
/// (`SimEvent::Removed`, which combat's deaths send too) leave every player's selection and control groups in the frame
/// of their tick, each player's the removals they perceive (`crate::viewer_events`); so does anything else that no
/// longer exists, and anything that player may no longer select (`pick::selectable_id`): an enemy that walks out of
/// sight is deselected at once (a placeholder rule: the design doesn't say). A record the player may select (a GDO
/// player's remembered SupplyDeliveryStation) stays selected even if its object is removed out of sight: the player
/// doesn't learn of it (the filter gives them no such removal). A unit that walked into a Tunnel leaves the
/// selection and stays in its control groups, whose recall selects the members on the map (design change 0249). An
/// object the player sees that changed its kind in place (`SimEvent::KindChanged`, plan decision T15), or that started
/// or stopped being unfinished (design change 0185: a building whose Warp In completes or is cancelled), stays selected
/// as the type it groups as now (`ControlState::set_type`), so the panels show its new group: the group its members
/// joined becomes the ActiveGroup if theirs is gone (design change 0199); and the ActiveGroup whose last member left
/// gives way to the next group in Tab order. The type a mode toggle pressed for the next tick switches it to counts as
/// its type from the press (design change 0198: `PendingCommands::kind_of`, which `ControlState::toggle` took at the
/// press), so that the selection keeps it until the sim has it too. A replaced sim (`SimRes::generation`) starts every
/// ControlState over, because its ids name other objects.
pub(super) fn sync_control_states(
    sim: Res<SimRes>,
    fog: Res<FogOfWar>,
    mode: Res<ClientMode>,
    pending: Res<PendingCommands>,
    events: Res<ViewerEvents>,
    mut states: ResMut<ControlStates>,
) {
    if states.generation != Some(sim.generation()) {
        states.by_player.clear();
        states.generation = Some(sim.generation());
        return;
    }
    if events.is_empty() && !sim.is_changed() && !fog.is_changed() && !mode.is_changed() {
        return;
    }
    let mut changed = false;
    for (&player, state) in &mut states.bypass_change_detection().by_player {
        let sight = Sight::new(&sim, player, fog.applies(*mode));
        let removed: BTreeSet<ObjectId> = (events.perceived_by(&sim, sight))
            .filter_map(|(_, event, _)| match *event {
                SimEvent::Removed(id) => Some(id),
                _ => None,
            })
            .collect();
        // A unit inside the player's Tunnel Network is off the map: out of the selection, kept in the control groups
        // (design change 0249, `sc_sim::network`).
        let held = |id: ObjectId| (sim.held_unit(id)).is_some_and(|held| held.object.owner == Some(player));
        changed |= state.retain_on_map(|id| !removed.contains(&id) && selectable_id(&sim, &sight, id), held);
        let changed_types: Vec<(ObjectId, GroupType)> = state
            .selection
            .member_types()
            .filter_map(|(id, ty)| {
                let object = sim.object(id).filter(|o| selectable_by(o, &sight))?;
                let kind = pending.kind_of(&sim, player, id)?;
                let now = GroupType {
                    kind,
                    ..GroupType::of(object)
                };
                (now != ty).then_some((id, now))
            })
            .collect();
        for (id, ty) in changed_types {
            changed |= state.set_type(id, ty);
        }
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

/// `systems/controls/selection.md > MapClickSelection` and `systems/controls/selection.md >
/// BoxSelection`: the `Select` action's clicks and boxes on the map, each with the roles of the modifiers it was sent
/// with. While a target is awaited (`systems/controls/interface.md > AwaitingTarget`) or a building is placed
/// (`systems/controls/hud.md > AwaitingPlacement`), a left-click on the map is the target or the
/// placement (`crate::command_ui`, `InterfaceState::takes_map_clicks`) and selects nothing, and a drag does nothing: no
/// box, no selection (`systems/controls/interface.md > AwaitingTarget > Dragging while targeting`, design
/// change 0067). In a build menu the map's clicks select as in DefaultState.
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
    if clicks.is_empty() && boxes.is_empty() || control.state().interface.takes_map_clicks() {
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
            state.select_box(&objects, dragged.rect, ClickRoles::of(&bindings, dragged.modifiers));
        }
    });
}

/// `systems/controls/selection.md > ControlGroups > Operations` and `systems/controls/selection.md >
/// GroupCycling`: the control group keys and Tab / Shift-Tab. Recall and Center snaps the camera to the centroid of
/// where the recalled objects are drawn (`core/camera.md > Panning > Centering`); the camera then keeps the view on the
/// map.
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
                A::AssignGroup(group) => {
                    state.assign_group(group, &objects);
                }
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
/// are on (`GameSetup::config`, `SimConfig::debug`); and in a replay's playback, where it only changes what the
/// replay shows (`crate::replay`).
pub(super) fn switch_player(
    actions: Res<ActionState>,
    sim: Res<SimRes>,
    playback: Option<Res<Playback>>,
    mut player: ResMut<LocalPlayer>,
) {
    if !actions.just_pressed(InputAction::DebugSwitchPlayer) || !(sim.config().debug || playback.is_some()) {
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
