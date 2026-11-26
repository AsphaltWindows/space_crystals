//! AwaitingPlacement's systems (`super`): the placement keys turn the ghost, a left-click places it, letting go of
//! Shift ends the Drones' queued placement, and the input layer is told whether a building is being placed.

use bevy::prelude::*;
use sc_sim::{ConstructionCommand, OrderKind, Reject, TunnelCommand};

use super::OrderOutbox;
use crate::ClientMode;
use crate::command_ui::interface::{Builder, InterfaceState, Placing, WhyUnavailable, commandable, deploy_recipient};
use crate::command_ui::{CommandRefused, InvalidPlacement};
use crate::editor::orientation::Turn;
use crate::input::{self, ActionState, Bindings, InputAction, MapClick};
use crate::selection::{ControlStates, LocalControl};
use crate::sim_driver::LocalPlayer;

/// `factions/gdo/structures/deployment_center.md > AwaitingPlacement`: "R rotates the ghost 90 degrees clockwise,
/// Shift+R rotates 90 degrees counter-clockwise. [...] F flips the ghost horizontally, Shift+F flips vertically": the
/// placement keys, which the input layer fires in place of the command slots R and F while a building is placed
/// ([`report_placing`]), the editor's own (`InputAction::is_placement`, `Turn::of_action`). A Warp In's ghost turns
/// about its Beacon's centre (design change 0121); a Deploy's Beacon doesn't turn (`Placing::turned`).
pub(in crate::command_ui) fn turn_ghost(actions: Res<ActionState>, mut control: LocalControl) {
    let Some(placing) = control.state().interface.placing() else {
        return;
    };
    let turned = actions
        .pressed_actions()
        .filter_map(Turn::of_action)
        .fold(placing, Placing::turned);
    if turned != placing {
        control.update(|state| state.interface = InterfaceState::AwaitingPlacement(turned));
    }
}

/// `systems/controls/interface.md > AwaitingTarget > Shift while placing`
/// (`systems/controls/interface.md > Exception, Drones`, design change 0125): once a Shift-click has sent a
/// Drone of the group to deploy and placement stayed for the next one (`Builder::Drones::queued`), letting go of Shift,
/// the Queue modifier, ends placement: back to DefaultState ("releasing Shift ends placement"). Before the first
/// Shift-click, letting go of it changes nothing, as for a target (`super::end_queued_targeting`, the design's proposed
/// default). It runs before the other input systems, as that one does, so that a command key pressed as Shift is let go
/// starts anew.
pub(in crate::command_ui) fn end_queued_placement(actions: Res<ActionState>, mut control: LocalControl) {
    let queued = control
        .state()
        .interface
        .placing()
        .is_some_and(|placing| placing.builder == Builder::Drones { queued: true });
    if queued && !actions.held(InputAction::Queue) {
        control.update(|state| state.interface = InterfaceState::DefaultState);
    }
}

/// `systems/controls/hud.md > AwaitingPlacement`: a left-click on the map (the Select action's click, at
/// the point where it went down) confirms the ghost, where its builder puts it ([`Builder`]), if the check the ghost
/// shows allows it now; the sim checks it again as it applies.
///
/// - **A constructor's building** (`factions/gdo/structures/deployment_center.md >
///   ObjectInterfaceState[DeploymentCenter] > AwaitingPlacement`: "Left-click valid location within build radius:
///   places building at the ghost's position and rotation, clears ReadyToPlace (CommandIssuingTransition, returns to
///   DefaultState)"): the ghost under the click ([`Placing::origin_under`]), if `Sim::placement_check` allows it: one
///   `ConstructionCommand::Place` to the one constructor, and back to DefaultState (`SimEvent::PlacementRefused`
///   when the sim refuses it as it applies, `super::feedback`). Shift (`systems/controls/interface.md >
///   AwaitingTarget > Shift while placing`): a GDO constructor "only ever has one building ready to place", so a
///   Shift-click places it and ends placement like a plain click.
/// - **A Beacon's Warp In** (`factions/colonists/economy.md > Warp In > Warp In Command`: "Left-click confirms the
///   footprint and begins the transformation"): wherever the click is, the ghost standing on the Beacon (design
///   change 0121), if `Sim::warp_in_check` allows it: one `ConstructionCommand::WarpIn` to the Beacon, and back to
///   DefaultState. The bank pays as it is given: a Warp In the gate refuses (too few Space Crystals since its entry
///   was pressed) sends nothing, says why ([`CommandRefused`], design change 0162) and placement goes on.
/// - **The Drones' Beacon** (`factions/colonists/units/drone.md > Drone ObjectInterfaceState > A: Deploy`; design
///   change 0125, `systems/controls/selection.md > SelectionGroup > Groupable workers used up by a build`): the
///   ghost under the click, if `Sim::deploy_check` allows it for the Drone the click goes to ([`deploy_recipient`]): a
///   plain click to the first Drone of the group, replacing its orders, and back to DefaultState; a Shift-click to the
///   first one with no Deploy yet, queued, and placement stays while another Drone of the group has none
///   (`systems/controls/interface.md > AwaitingTarget > Shift while placing > Exception, Drones`), until Shift
///   is let go ([`end_queued_placement`]). **Reading**: a Shift-click while every Drone of the group has a Deploy
///   already sends nothing and ends placement ("Once every Drone in the group has a Deploy, placement ends").
/// - **An Agent's building** (`factions/syndicate/units/agent.md > Agent ObjectInterfaceState > A: Build Tunnel`:
///   "Left-click valid location confirms placement and dispatches the Agent (CommandIssuingTransition, returns to
///   DefaultState)"; slice M4-J1): the ghost under the click, if `Sim::build_check` allows it: one `OrderKind::Build`
///   to the Agent placing, queued with Shift (`systems/controls/interface.md > AwaitingTarget > Shift while
///   placing`: "each Agent takes exactly one placement", and placement ends like a plain click's; Tab and A again
///   place with the next Agent), and back to DefaultState. It is paid as it is given (design change 0233): a Build
///   the gate refuses (too few Supplies since A was pressed, the Agent building already) sends nothing, says why
///   ([`CommandRefused`], design change 0162) and placement goes on, as a Warp In's.
/// - **A Tunnel's expansion** (`factions/syndicate/tunnel_menus.md > AwaitingPlacement (Expansion)`: "Left-click valid
///   location: places expansion, begins construction (CommandIssuingTransition, returns to DefaultState)"; slice M4-F):
///   the ghost held within the Tunnel Area ([`Placing::origin`]), if `Sim::expansion_check` allows it, as far as the
///   player knows: one `TunnelCommand::Expand` to the Tunnel, and back to DefaultState. Shift changes nothing: a Tunnel
///   performs one operation at a time, with no queue (`factions/syndicate/tunnel_menus.md > Expansions under
///   construction`: "the Tunnel does one operation at a time, so there's no queue"), so, as a GDO constructor's, a
///   Shift-click places it and ends placement (`systems/controls/interface.md > AwaitingTarget > Shift while placing`:
///   "This only applies when the builder **can carry out more queued buildings**"). It is paid as it is given: an
///   Expand the gate refuses (too few Space Crystals since its entry was pressed) sends nothing, says why
///   ([`CommandRefused`]) and placement goes on; one the truth refuses (design change 0240) as an invalid click.
///
/// An invalid click sends nothing and placement goes on (M2 plan, H2: "cannot be confirmed"), with a short message
/// ([`InvalidPlacement`]; placeholder: the design gives feedback for invalid target clicks,
/// `systems/controls/interface.md > AwaitingTarget > Invalid target clicks`, and is silent for placement).
/// Nothing while the selection isn't commandable (a defeated player's, or anyone's once the game is over), and
/// placement then ends ([`super::validate_interface`]).
pub(in crate::command_ui) fn place_on_click(
    mut clicks: MessageReader<MapClick>,
    bindings: Res<Bindings>,
    mut control: LocalControl,
    mut outbox: OrderOutbox,
    mut invalid: MessageWriter<InvalidPlacement>,
    mut refused: MessageWriter<CommandRefused>,
) {
    let clicks: Vec<MapClick> = clicks
        .read()
        .filter(|click| click.action == InputAction::Select)
        .copied()
        .collect();
    for click in clicks {
        let state = control.state();
        let player = control.player();
        let Some(placing) = state.interface.placing() else {
            return;
        };
        if !commandable(control.sim(), player, &state.selection) {
            return;
        }
        let tick = control.sim().tick();
        match placing.builder {
            Builder::Constructor(constructor) => {
                let Some(origin) = placing.origin_under(click.ground) else {
                    continue;
                };
                let check = control
                    .sim()
                    .placement_check(player, placing.kind, origin, placing.orientation);
                let place = ConstructionCommand::Place {
                    origin,
                    orientation: placing.orientation,
                };
                if check.is_valid() && outbox.construct(player, vec![constructor], place) {
                    control.update(|state| state.interface = InterfaceState::DefaultState);
                } else {
                    let problem = check.first_problem();
                    invalid.write(InvalidPlacement { problem, tick });
                }
            }
            Builder::Beacon(beacon) => {
                let Some(check) = control.sim().warp_in_check(beacon, placing.kind, placing.orientation) else {
                    continue;
                };
                if !check.is_valid() {
                    let problem = check.first_problem();
                    invalid.write(InvalidPlacement { problem, tick });
                    continue;
                }
                let warp_in = ConstructionCommand::WarpIn {
                    kind: placing.kind,
                    orientation: placing.orientation,
                };
                match outbox.try_construct(player, vec![beacon], warp_in) {
                    Ok(()) => control.update(|state| state.interface = InterfaceState::DefaultState),
                    Err(reject) => {
                        let why = WhyUnavailable::Refused(reject);
                        refused.write(CommandRefused { why, tick });
                    }
                }
            }
            Builder::Drones { .. } => {
                let Some(origin) = placing.origin_under(click.ground) else {
                    continue;
                };
                let queue = bindings.holds(InputAction::Queue, click.modifiers);
                let Some(drone) = deploy_recipient(&state.selection, queue, &outbox.deploying(player)) else {
                    control.update(|state| state.interface = InterfaceState::DefaultState);
                    continue;
                };
                let Some(check) = control.sim().deploy_check(drone, origin, queue) else {
                    continue;
                };
                if !check.is_valid() || !outbox.give(player, vec![drone], OrderKind::Deploy { origin }, queue) {
                    let problem = check.first_problem();
                    invalid.write(InvalidPlacement { problem, tick });
                    continue;
                }
                let next = deploy_recipient(&state.selection, true, &outbox.deploying(player))
                    .filter(|_| queue)
                    .map_or(InterfaceState::DefaultState, |_| {
                        let builder = Builder::Drones { queued: true };
                        InterfaceState::AwaitingPlacement(Placing { builder, ..placing })
                    });
                control.update(|state| state.interface = next);
            }
            Builder::Agent(agent) => {
                let Some(origin) = placing.origin_under(click.ground) else {
                    continue;
                };
                let queue = bindings.holds(InputAction::Queue, click.modifiers);
                let (kind, orientation) = (placing.kind, placing.orientation);
                let Some(check) = control.sim().build_check(agent, kind, origin, orientation, queue) else {
                    continue;
                };
                if !check.is_valid() {
                    let problem = check.first_problem();
                    invalid.write(InvalidPlacement { problem, tick });
                    continue;
                }
                let build = OrderKind::Build {
                    kind,
                    origin,
                    orientation,
                };
                match outbox.try_give(player, vec![agent], build, queue) {
                    Ok(()) => control.update(|state| state.interface = InterfaceState::DefaultState),
                    Err(Reject::Unplaceable(problem)) => {
                        let problem = Some(problem);
                        invalid.write(InvalidPlacement { problem, tick });
                    }
                    Err(reject) => {
                        let why = WhyUnavailable::Refused(reject);
                        refused.write(CommandRefused { why, tick });
                    }
                }
            }
            Builder::Tunnel(tunnel) => {
                let Some(origin) = placing.origin(control.sim(), Some(click.ground)) else {
                    continue;
                };
                let (kind, orientation) = (placing.kind, placing.orientation);
                let Some(check) = control.sim().expansion_check(tunnel, kind, origin, orientation) else {
                    continue;
                };
                if !check.is_valid() {
                    let problem = check.first_problem();
                    invalid.write(InvalidPlacement { problem, tick });
                    continue;
                }
                let expand = TunnelCommand::Expand {
                    kind,
                    origin,
                    orientation,
                };
                match outbox.try_tunnel(player, tunnel, expand) {
                    Ok(()) => control.update(|state| state.interface = InterfaceState::DefaultState),
                    Err(Reject::Unplaceable(problem)) => {
                        let problem = Some(problem);
                        invalid.write(InvalidPlacement { problem, tick });
                    }
                    Err(reject) => {
                        let why = WhyUnavailable::Refused(reject);
                        refused.write(CommandRefused { why, tick });
                    }
                }
            }
        }
    }
}

/// Tells the input layer whether the local player places a building (`crate::input::Placing`), so that from the next
/// frame's input on R, Shift+R, F and Shift+F are the placement keys, which turn the ghost, instead of the command
/// slots R and F (plan decision T10, risk R11), as they turn the editor's structures. In the game modes only: the
/// editor reports its own placements (`crate::editor::input::report_placing`). Every other state ends it, so the
/// keys never outlive the placement: it runs even while the command input doesn't (once the game is over, for a
/// defeated player, during a playback), when the placement ends ([`super::validate_interface`]).
pub(in crate::command_ui) fn report_placing(
    mode: Res<ClientMode>,
    player: Res<LocalPlayer>,
    states: Res<ControlStates>,
    mut placing: ResMut<input::Placing>,
) {
    if mode.is_game() {
        placing.set_if_neq(input::Placing(states.of(player.0).interface.placing().is_some()));
    }
}
