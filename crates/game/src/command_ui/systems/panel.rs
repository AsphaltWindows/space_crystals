//! The command panel's systems (`super`): the ObjectInterfaceState validated every frame, Esc, and the command slots
//! pressed by hotkey or by a click on their button.

use bevy::prelude::*;

use super::OrderOutbox;
use crate::ClientMode;
use crate::command_ui::CommandRefused;
use crate::command_ui::interface::{
    Conditions, InterfaceState, PanelCommand, Placing, SlotPress, commandable, panel, press, recipients, still_valid,
};
use crate::hud::layout::CommandGrid;
use crate::input::{ActionState, CommandSlot, InputAction};
use crate::platform::SurfaceSize;
use crate::selection::LocalControl;

/// `systems/controls/selection_and_hud.md > ObjectInterfaceState`: "Each tick, the ObjectInterfaceState is validated
/// against the active SelectionGroup's game state": an AwaitingTarget whose command the ActiveGroup no longer offers, a
/// build menu whose ActiveGroup is no Deployment Center or Beacon, an AwaitingPlacement whose constructor no longer
/// holds the building ready, whose ActiveGroup is no longer Drones or the Beacon, or one under deployment (design
/// change 0116), or any of them whose selection is no longer commandable (no longer all the player's, or the game is
/// over: design change 0101), goes back to DefaultState. A selected unit that died has already left the selection
/// (`crate::selection`), and with the ActiveGroup left the target is still awaited (design change 0069).
pub(in crate::command_ui) fn validate_interface(mut control: LocalControl) {
    let state = control.state();
    if state.interface == InterfaceState::DefaultState {
        return;
    }
    let commandable = commandable(control.sim(), control.player(), &state.selection);
    let conditions = Conditions::of(control.sim(), control.player(), &state.selection);
    if !still_valid(state.interface, &state.selection, commandable, &conditions) {
        control.update(|state| state.interface = InterfaceState::DefaultState);
    }
}

/// `systems/controls/selection_and_hud.md > AwaitingTarget`: "Escape or right-click cancels via a StateOnlyTransition
/// back to DefaultState"; in the other multi-stage states Esc goes back one state (`InterfaceState::back`): from
/// AwaitingPlacement to the Deployment Center's build menu, the Beacon's Warp In menu, or the Extraction Facility's or
/// the Drones' DefaultState, from a build menu to DefaultState. Esc is the Cancel action; right-click is
/// [`super::right_click_orders`], Back (Z) [`press_command_slots`].
pub(in crate::command_ui) fn cancel_on_escape(actions: Res<ActionState>, mut control: LocalControl) {
    let state = control.state();
    if actions.just_pressed(InputAction::Cancel) && state.interface != InterfaceState::DefaultState {
        let back = state.interface.back(&state.selection);
        control.update(|state| state.interface = back);
    }
}

/// `systems/controls/selection_and_hud.md > DefaultState`: a command hotkey (`InputAction::CommandSlot`) or a click on
/// a panel button (a press of the Select action over the button, found by the panel's geometry like the
/// SelectionPanel's portraits) enters AwaitingTarget for a command that needs a target, or issues an immediate one: a
/// common command to the whole selection, a group command to the ActiveGroup (`systems/controls/selection_and_hud.md >
/// CommandPanel`), a Train or a Cancel Production to every producer among them as one command. A mode toggle (Raise and
/// Lower Shield, Convert) switches the type of the units it goes to at the press, and their merged group becomes the
/// ActiveGroup (design change 0198, `crate::selection::ControlState::toggle`), so that the next press, the next frame
/// or even before the sim's next tick, reaches them all. A constructor's enter its build menu
/// (`InterfaceState::BuildMenu`) or the placement of its building ready to place (`InterfaceState::AwaitingPlacement`),
/// or give its one constructor a construction command and return to DefaultState ("CommandIssuingTransition, returns to
/// DefaultState"). A Beacon's enter its Warp In menus and a Warp In's placement on it (`factions/colonists/economy.md >
/// Beacon > Beacon ObjectInterfaceState`, design change 0122: the Beacon Tabbed to), the Drones' A the placement of a
/// Beacon (design change 0125), an Egress Node's F selects its returning Drones instead
/// (`factions/colonists/economy.md > Egress Node > Egress Node ObjectInterfaceState`, slice M3-J2), and the X of
/// whatever transforms cancels it (design change 0116). Back (Z) returns to the
/// previous state. While a target is awaited or a building placed the other commands are unavailable (design change
/// 0066), and so is a production or construction command the gate would refuse (design change 0081; never for the
/// army cap, 0143): [`press`] finds nothing to do for them, and a greyed-out button says why it is
/// ([`CommandRefused`], design change 0162: "nothing is issued, and the player gets the same feedback as when an
/// order is refused, naming the reason"). Nothing while the selection isn't commandable (a defeated player's, or
/// anyone's once the game is over).
pub(in crate::command_ui) fn press_command_slots(
    actions: Res<ActionState>,
    mode: Res<ClientMode>,
    size: Res<SurfaceSize>,
    mut control: LocalControl,
    mut outbox: OrderOutbox,
    mut refused: MessageWriter<CommandRefused>,
) {
    let mut slots: Vec<CommandSlot> = actions
        .pressed_actions()
        .filter_map(|action| match action {
            InputAction::CommandSlot(slot) => Some(slot),
            _ => None,
        })
        .collect();
    if mode.is_game()
        && actions.just_pressed(InputAction::Select)
        && actions.pointer().over_ui
        && let Some(slot) = actions
            .cursor()
            .and_then(|cursor| CommandGrid::on(size.logical()).slot_at(cursor))
    {
        slots.push(slot);
    }
    let queue = actions.held(InputAction::Queue);
    for slot in slots {
        let state = control.state();
        let player = control.player();
        if !commandable(control.sim(), player, &state.selection) {
            return;
        }
        let conditions = Conditions::of(control.sim(), player, &state.selection);
        let shown = panel(&state.selection, state.interface, &conditions);
        match press(&shown, slot) {
            SlotPress::AwaitTarget(command) => {
                control.update(|state| state.interface = InterfaceState::awaiting(command));
            }
            SlotPress::Issue(command, kind) => {
                let units = recipients(&state.selection, command);
                outbox.give(player, units, kind, queue);
            }
            SlotPress::UseAbility(command, ability) => {
                let units = recipients(&state.selection, command);
                if outbox.use_ability(player, units.clone(), ability) {
                    control.update(|state| state.toggle(&units, ability));
                }
            }
            SlotPress::Produce(command, production) => {
                let producers = recipients(&state.selection, command);
                outbox.produce(player, producers, production);
            }
            SlotPress::Construct(command, construction) => {
                let constructors = recipients(&state.selection, command);
                if outbox.construct(player, constructors, construction) {
                    control.update(|state| state.interface = InterfaceState::DefaultState);
                }
            }
            SlotPress::OpenMenu(menu) => {
                control.update(|state| state.interface = InterfaceState::BuildMenu(menu));
            }
            SlotPress::AwaitPlacement(kind) => {
                // The one constructor of the ActiveGroup, which holds the building ready.
                if let [constructor] = recipients(&state.selection, PanelCommand::PlaceReady(kind))[..] {
                    let placing = Placing::new(constructor, kind);
                    control.update(|state| state.interface = InterfaceState::AwaitingPlacement(placing));
                }
            }
            SlotPress::AwaitWarpIn(kind) => {
                // The one Beacon of the ActiveGroup (Ungroupable), the one Tabbed to.
                if let [beacon] = recipients(&state.selection, PanelCommand::WarpIn(kind))[..] {
                    let placing = Placing::warp_in(beacon, kind);
                    control.update(|state| state.interface = InterfaceState::AwaitingPlacement(placing));
                }
            }
            SlotPress::AwaitDeploy => {
                control.update(|state| state.interface = InterfaceState::AwaitingPlacement(Placing::deploy()));
            }
            SlotPress::SelectReturningDrones => {
                let objects = control.objects();
                let drones = conditions.returning().to_vec();
                control.update(|state| state.replace_selection(drones, &objects));
            }
            SlotPress::Back => {
                let back = state.interface.back(&state.selection);
                control.update(|state| state.interface = back);
            }
            SlotPress::Nothing => {
                if let Some(why) = shown.get(slot).and_then(|shown| shown.why) {
                    let tick = control.sim().tick();
                    refused.write(CommandRefused { why, tick });
                }
            }
        }
    }
}
