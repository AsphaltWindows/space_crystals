//! AwaitingPlacement's systems (`super`): the placement keys turn the ghost, a left-click places it, and the input
//! layer is told whether a building is being placed.

use bevy::prelude::*;
use sc_sim::ConstructionCommand;

use super::OrderOutbox;
use crate::ClientMode;
use crate::command_ui::InvalidPlacement;
use crate::command_ui::interface::{InterfaceState, Placing, commandable};
use crate::editor::orientation::Turn;
use crate::input::{self, ActionState, InputAction, MapClick};
use crate::selection::{ControlStates, LocalControl};
use crate::sim_driver::LocalPlayer;

/// `> AwaitingPlacement`: "R rotates the ghost 90 degrees clockwise, Shift+R rotates 90 degrees counter-clockwise.
/// [...] F flips the ghost horizontally, Shift+F flips vertically": the placement keys, which the input layer fires
/// in place of the command slots R and F while a building is placed ([`report_placing`]), the editor's own
/// (`InputAction::is_placement`, `Turn::of_action`).
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

/// `> AwaitingPlacement`: "Left-click valid location within build radius: places building at the ghost's position
/// and rotation, clears ReadyToPlace (CommandIssuingTransition, returns to DefaultState)". A left-click on the map
/// (the Select action's click, at the point where it went down) places the ghost there
/// ([`Placing::origin_under`]) if the placement check allows it now (`Sim::placement_check`, the rules the ghost
/// shows): one `ConstructionCommand::Place` to the one constructor, and back to DefaultState. The sim checks it
/// again as it applies (`SimEvent::PlacementRefused`, `super::feedback`).
///
/// - **An invalid click** sends nothing and placement goes on (M2 plan, H2: "cannot be confirmed"), with a short
///   message ([`InvalidPlacement`]; placeholder: the design gives feedback for invalid target clicks, `>
///   AwaitingTarget > Invalid target clicks`, and is silent for placement).
/// - **Shift** (`> AwaitingTarget > Shift while placing`, design change 0071): a GDO constructor "only ever has one
///   building ready to place", so a Shift-click places it and ends placement like a plain click.
/// - Nothing while the selection isn't commandable (a defeated player's, or anyone's once the game is over), and
///   placement then ends ([`super::validate_interface`]).
pub(in crate::command_ui) fn place_on_click(
    mut clicks: MessageReader<MapClick>,
    mut control: LocalControl,
    mut outbox: OrderOutbox,
    mut invalid: MessageWriter<InvalidPlacement>,
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
        if check.is_valid() && outbox.construct(player, vec![placing.constructor], place) {
            control.update(|state| state.interface = InterfaceState::DefaultState);
        } else {
            invalid.write(InvalidPlacement {
                problem: check.first_problem(),
                tick: control.sim().tick(),
            });
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
