//! The command flow's systems: the frame's input (`ARCHITECTURE.md > Client architecture > Input`) changes the local
//! player's ObjectInterfaceState and gives orders, by the rules of [`super::interface`] and [`super::resolution`].
//!
//! Orders go to the sim as `Command::Order` in `PendingCommands`, named for the local player's own units only, and
//! only when the sim would take them now (`Sim::validate`); each one given is also an [`OrderIssued`] message, for
//! the indicators. A command issued with the Queue modifier held (Shift) is queued (`systems/controls/commands.md >
//! Unit Command`), else it replaces the units' orders. Abilities (the Riot Shield) go as `Command::Ability`, at once
//! and never queued.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use sc_sim::{Ability, Command, ObjectId, OrderKind, PlayerCommand, PlayerId, Target};

use super::OrderIssued;
use super::interface::{
    InterfaceState, Modes, SlotPress, able, active_template, commandable, panel, press, recipients, still_valid,
};
use super::resolution::{CursorTarget, order_without_attack, right_click, target_order};
use crate::ClientMode;
use crate::hud::layout::CommandGrid;
use crate::input::{ActionState, Bindings, CommandSlot, InputAction, MapClick};
use crate::platform::SurfaceSize;
use crate::selection::LocalControl;
use crate::sim_driver::{PendingCommands, SimRes};

/// Where orders go: the sim's next tick, and the indicators.
#[derive(SystemParam)]
pub(super) struct OrderOutbox<'w> {
    sim: Res<'w, SimRes>,
    pending: ResMut<'w, PendingCommands>,
    issued: MessageWriter<'w, OrderIssued>,
}

impl OrderOutbox<'_> {
    /// Gives `kind` to `player`'s units among `units` (`> CommandPanel`: "Commands can only be issued to objects
    /// owned by the player"; only units take orders). Nothing when none of them is left, or when the sim would
    /// reject the order (`Sim::validate`: e.g. a point off the map, or an indestructible target to attack).
    fn give(&mut self, player: PlayerId, units: Vec<ObjectId>, kind: OrderKind, queue: bool) -> bool {
        let units = self.own_units(player, units);
        // `ARCHITECTURE.md > Commands, events and replays`: never a non-finite number, which a replay can't store.
        if units.is_empty() || !finite(kind) {
            return false;
        }
        let command = PlayerCommand {
            player,
            command: Command::Order {
                units: units.clone(),
                kind,
                queue,
            },
        };
        if let Err(reject) = self.sim.validate(&command) {
            debug!("order {kind:?} not given: {reject}");
            return false;
        }
        self.pending.0.push(command);
        self.issued.write(OrderIssued {
            kind,
            units,
            queue,
            tick: self.sim.tick(),
        });
        true
    }

    /// The order of a click that resolved to `kind` at the ground point `ground`, given to `units`: those whose
    /// `modes` keep them from attacking get [`order_without_attack`] instead (`factions/gdo/units.md > Peacekeeper >
    /// Riot Shield`: a Move there in place of an Attack), as an order of their own. Whether anything was given.
    fn give_click(
        &mut self,
        player: PlayerId,
        units: Vec<ObjectId>,
        kind: OrderKind,
        ground: Vec2,
        queue: bool,
        modes: &Modes,
    ) -> bool {
        let instead = order_without_attack(kind, ground);
        if instead == kind {
            return self.give(player, units, kind, queue);
        }
        let (armed, unarmed): (Vec<ObjectId>, Vec<ObjectId>) = units.into_iter().partition(|&id| modes.can_attack(id));
        let attacked = self.give(player, armed, kind, queue);
        let moved = self.give(player, unarmed, instead, queue);
        attacked || moved
    }

    /// `player`'s `units` use `ability` (`Command::Ability`), at once: an ability isn't queued. Nothing when none of
    /// them is the player's, or when the sim would reject it (`Sim::validate`: e.g. none of them has the ability).
    fn use_ability(&mut self, player: PlayerId, units: Vec<ObjectId>, ability: Ability) -> bool {
        let units = self.own_units(player, units);
        if units.is_empty() {
            return false;
        }
        let command = PlayerCommand {
            player,
            command: Command::Ability { units, ability },
        };
        if let Err(reject) = self.sim.validate(&command) {
            debug!("ability {ability:?} not used: {reject}");
            return false;
        }
        self.pending.0.push(command);
        true
    }

    /// The units among `units` that are `player`'s and take orders.
    fn own_units(&self, player: PlayerId, units: Vec<ObjectId>) -> Vec<ObjectId> {
        units
            .into_iter()
            .filter(|&id| {
                self.sim
                    .object(id)
                    .is_some_and(|o| o.owner == Some(player) && o.orders.is_some())
            })
            .collect()
    }
}

/// Whether every number of the order is finite.
fn finite(kind: OrderKind) -> bool {
    match kind {
        OrderKind::Move(Target::Point(p)) | OrderKind::AttackMove(p) | OrderKind::Patrol(p) => p.is_finite(),
        OrderKind::Move(Target::Object(_)) | OrderKind::Attack(_) | OrderKind::Hold | OrderKind::Stop => true,
    }
}

/// `> ObjectInterfaceState`: "Each tick, the ObjectInterfaceState is validated against the active SelectionGroup's
/// game state": an AwaitingTarget whose command the ActiveGroup no longer offers, or whose selection is no longer
/// all the player's, goes back to DefaultState.
pub(super) fn validate_interface(mut control: LocalControl) {
    let state = control.state();
    if state.interface == InterfaceState::DefaultState {
        return;
    }
    let commandable = commandable(control.sim(), control.player(), &state.selection);
    let modes = Modes::of(control.sim(), &state.selection);
    if !still_valid(state.interface, &state.selection, commandable, &modes) {
        control.update(|state| state.interface = InterfaceState::DefaultState);
    }
}

/// `> AwaitingTarget`: "Escape or right-click cancels via a StateOnlyTransition back to DefaultState". Esc is the
/// Cancel action; right-click is [`right_click_orders`], Back (Z) [`press_command_slots`].
pub(super) fn cancel_targeting(actions: Res<ActionState>, mut control: LocalControl) {
    if actions.just_pressed(InputAction::Cancel) && control.state().interface != InterfaceState::DefaultState {
        control.update(|state| state.interface = InterfaceState::DefaultState);
    }
}

/// `> DefaultState`: a command hotkey (`InputAction::CommandSlot`) or a click on a panel button (a press of the
/// Select action over the button, found by the panel's geometry like the SelectionPanel's portraits) enters
/// AwaitingTarget for a command that needs a target, or issues an immediate one: a common command to the whole
/// selection, a group command to the ActiveGroup (`> CommandPanel`). Back (Z) returns to DefaultState.
pub(super) fn press_command_slots(
    actions: Res<ActionState>,
    mode: Res<ClientMode>,
    size: Res<SurfaceSize>,
    mut control: LocalControl,
    mut outbox: OrderOutbox,
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
        let modes = Modes::of(control.sim(), &state.selection);
        match press(&panel(&state.selection, state.interface, &modes), slot) {
            SlotPress::AwaitTarget(command) => {
                control.update(|state| state.interface = InterfaceState::AwaitingTarget(command));
            }
            SlotPress::Issue(command, kind) => {
                let units = recipients(&state.selection, command);
                outbox.give(player, units, kind, queue);
            }
            SlotPress::UseAbility(command, ability) => {
                let units = recipients(&state.selection, command);
                outbox.use_ability(player, units, ability);
            }
            SlotPress::Back => {
                control.update(|state| state.interface = InterfaceState::DefaultState);
            }
            SlotPress::Nothing => {}
        }
    }
}

/// `> AwaitingTarget`: a left-click on the map (the Select action's click) on a valid target gives the
/// PendingCommand's order for what is under it (`> CursorTarget`), to the whole selection or to the ActiveGroup as
/// the command is common or not, and returns to DefaultState, with Shift too (the design returns to DefaultState
/// after every issuing click). A click on an invalid target does nothing and keeps waiting (placeholder: the
/// design doesn't say). Units whose Riot Shield is up move to an enemy clicked for an Attack
/// ([`OrderOutbox::give_click`]).
pub(super) fn click_targets(
    mut clicks: MessageReader<MapClick>,
    bindings: Res<Bindings>,
    mut control: LocalControl,
    mut outbox: OrderOutbox,
) {
    let clicks: Vec<MapClick> = clicks
        .read()
        .filter(|click| click.action == InputAction::Select)
        .copied()
        .collect();
    for click in clicks {
        let state = control.state();
        let InterfaceState::AwaitingTarget(pending) = state.interface else {
            continue;
        };
        let target = CursorTarget::at(&control.objects(), click.view, click.ground);
        let Some(kind) = target_order(pending, target) else {
            continue;
        };
        let units = recipients(&state.selection, pending);
        let queue = bindings.holds(InputAction::Queue, click.modifiers);
        let modes = Modes::of(control.sim(), &state.selection);
        if outbox.give_click(control.player(), units, kind, click.ground, queue, &modes) {
            control.update(|state| state.interface = InterfaceState::DefaultState);
        }
    }
}

/// `> RightClickResolution`: a right-click on the map (the ContextCommand action's click) in DefaultState gives the
/// order that the ActiveGroup's template resolves for what is under the cursor, to every selected unit able to
/// carry it out ([`able`]); those whose Riot Shield is up move to the clicked point instead of attacking an enemy
/// (`factions/gdo/units.md > Peacekeeper > Riot Shield`, [`OrderOutbox::give_click`]). Nothing unless the selection
/// is all the player's (`> CommandPanel`). While a target is awaited, a right-click anywhere, over the HUD too,
/// cancels instead (`> AwaitingTarget`).
pub(super) fn right_click_orders(
    actions: Res<ActionState>,
    mut clicks: MessageReader<MapClick>,
    bindings: Res<Bindings>,
    mut control: LocalControl,
    mut outbox: OrderOutbox,
) {
    let clicks: Vec<MapClick> = clicks
        .read()
        .filter(|click| click.action == InputAction::ContextCommand)
        .copied()
        .collect();
    if control.state().interface != InterfaceState::DefaultState {
        if actions.just_pressed(InputAction::ContextCommand) {
            control.update(|state| state.interface = InterfaceState::DefaultState);
        }
        return;
    }
    for click in clicks {
        let state = control.state();
        let player = control.player();
        if !commandable(control.sim(), player, &state.selection) {
            return;
        }
        let Some(template) = active_template(&state.selection) else {
            return;
        };
        let target = CursorTarget::at(&control.objects(), click.view, click.ground);
        let Some((command, kind)) = right_click(template, target) else {
            continue;
        };
        let units = able(&state.selection, command);
        let queue = bindings.holds(InputAction::Queue, click.modifiers);
        let modes = Modes::of(control.sim(), &state.selection);
        outbox.give_click(player, units, kind, click.ground, queue, &modes);
    }
}
