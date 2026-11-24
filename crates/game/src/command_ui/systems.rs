//! The command flow's systems: the frame's input (`ARCHITECTURE.md > Client architecture > Input`) changes the local
//! player's ObjectInterfaceState and gives orders, by the rules of [`super::interface`] and [`super::resolution`].
//!
//! Orders go to the sim as `Command::Order` in `PendingCommands`, named for the local player's own units only, and
//! only when the sim would take them now (`Sim::validate`); each one given is also an [`OrderIssued`] message, for
//! the indicators. A click whose units get different orders gives one order per distinct order
//! ([`super::resolution::group_orders`]). A command issued with the Queue modifier held (Shift) is queued
//! (`systems/controls/commands.md > Unit Command`), else it replaces the units' orders. Abilities (the Riot Shield)
//! go as `Command::Ability`, at once and never queued.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use sc_sim::{Ability, Command, ObjectId, OrderKind, PlayerCommand, PlayerId, Target};

use super::interface::{
    InterfaceState, Modes, SlotPress, UnitCommand, commandable, panel, press, recipients, still_valid,
};
use super::resolution::{MapTarget, capabilities, group_orders, order_without_attack, target_order, unit_right_click};
use super::{InvalidTarget, OrderIssued};
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

    /// Gives each of `orders` (a unit and its order, in the units' order) as few orders as possible: one per
    /// distinct order ([`group_orders`]), so the units sent to one location go as one group. Whether anything was
    /// given.
    fn give_each(&mut self, player: PlayerId, orders: Vec<(ObjectId, OrderKind)>, queue: bool) -> bool {
        let mut given = false;
        for (kind, units) in group_orders(orders) {
            given |= self.give(player, units, kind, queue);
        }
        given
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

/// `> AwaitingTarget > Dragging while targeting` (design change 0068): "A targeting click counts at the press
/// point". The target found under a press of the Select action that went down on the map while a target was
/// awaited, as the frame of the press drew the objects; its click, sent at the release, uses it. Cleared when the
/// press ends or targeting does.
#[derive(Resource, Default, Debug)]
pub(super) struct TargetPress(Option<PressedTarget>);

/// What was under a press while a target was awaited ([`TargetPress`]).
#[derive(Copy, Clone, Debug, PartialEq)]
struct PressedTarget {
    /// The command awaiting its target as the button went down.
    pending: UnitCommand,
    /// Where it went down, logical pixels: the click's own (`MapClick::screen`).
    screen: Vec2,
    target: MapTarget,
}

/// `> ObjectInterfaceState`: "Each tick, the ObjectInterfaceState is validated against the active SelectionGroup's
/// game state": an AwaitingTarget whose command the ActiveGroup no longer offers, or whose selection is no longer
/// all the player's, goes back to DefaultState. A selected unit that died has already left the selection
/// (`crate::selection`), and with the ActiveGroup left the target is still awaited (design change 0069).
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
/// selection, a group command to the ActiveGroup (`> CommandPanel`). Back (Z) returns to DefaultState. While a target
/// is awaited the other commands are unavailable (design change 0066): [`press`] finds nothing to do for them.
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
/// PendingCommand's order for what is under it ([`target_order`]), to the whole selection or to the ActiveGroup as
/// the command is common or not, and returns to DefaultState, with Shift too (the design returns to DefaultState
/// after every issuing click). Units whose Riot Shield is up get [`order_without_attack`] for an Attack.
///
/// - **The press point** (design change 0068): the target is what was under the cursor when the button went down,
///   as that frame drew it ([`TargetPress`]), even if the cursor drifted or the target moved before the release.
///   A press that moved far enough to be a drag is no click: it does nothing (design change 0067), and neither
///   does a click whose press went down before the target was awaited.
/// - **An invalid target** (design change 0065): a click that gives nothing, on a target the command can't use
///   (the Inactive pointer), is ignored and the target is still awaited; an [`InvalidTarget`] message tells the
///   player (`super::feedback`).
pub(super) fn click_targets(
    mut clicks: MessageReader<MapClick>,
    actions: Res<ActionState>,
    bindings: Res<Bindings>,
    mut pressed: ResMut<TargetPress>,
    mut control: LocalControl,
    mut outbox: OrderOutbox,
    mut invalid: MessageWriter<InvalidTarget>,
) {
    let clicks: Vec<MapClick> = clicks
        .read()
        .filter(|click| click.action == InputAction::Select)
        .copied()
        .collect();
    let awaited = match control.state().interface {
        InterfaceState::AwaitingTarget(pending) => Some(pending),
        InterfaceState::DefaultState => None,
    };
    if let Some(pending) = awaited
        && actions.just_pressed(InputAction::Select)
        && let Some(press) = actions.pointer().press.filter(|p| p.action == InputAction::Select)
    {
        let target = MapTarget::at(control.sim(), &control.objects(), press.view, press.ground);
        pressed.0 = Some(PressedTarget {
            pending,
            screen: press.screen,
            target,
        });
    }
    for click in clicks {
        let state = control.state();
        let InterfaceState::AwaitingTarget(pending) = state.interface else {
            continue;
        };
        let Some(target) = pressed
            .0
            .take()
            .filter(|p| p.pending == pending && p.screen == click.screen)
            .map(|p| p.target)
        else {
            continue;
        };
        let player = control.player();
        let queue = bindings.holds(InputAction::Queue, click.modifiers);
        let given = target_order(pending, &target).is_some_and(|kind| {
            let modes = Modes::of(control.sim(), &state.selection);
            let orders = recipients(&state.selection, pending)
                .into_iter()
                .map(|id| {
                    if modes.can_attack(id) {
                        (id, kind)
                    } else {
                        (id, order_without_attack(kind, target.ground))
                    }
                })
                .collect();
            outbox.give_each(player, orders, queue)
        });
        if given {
            control.update(|state| state.interface = InterfaceState::DefaultState);
        } else {
            invalid.write(InvalidTarget {
                command: pending,
                tick: control.sim().tick(),
            });
        }
    }
    if awaited.is_none() || actions.pointer().press.is_none() {
        pressed.0 = None;
    }
}

/// `> RightClickResolution`: a right-click on the map (the ContextCommand action's click) in DefaultState. Each
/// selected unit resolves it for itself ([`unit_right_click`], design changes 0062 and 0063), whichever group is
/// active: those that can carry out the resolved command do, the others move to the location or follow the unit
/// under the cursor, and those whose Riot Shield is up move to the clicked point instead of attacking an enemy
/// (question Q0020 point 1). Objects without commands do nothing. The orders go out grouped
/// ([`OrderOutbox::give_each`]). Nothing unless the selection is all the player's (`> CommandPanel`). While a target
/// is awaited, a right-click anywhere, over the HUD too, cancels instead (`> AwaitingTarget`).
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
        let target = MapTarget::at(control.sim(), &control.objects(), click.view, click.ground);
        let modes = Modes::of(control.sim(), &state.selection);
        let orders = capabilities(&state.selection, &modes)
            .into_iter()
            .filter_map(|(id, unit)| Some((id, unit_right_click(unit, &target)?)))
            .collect();
        let queue = bindings.holds(InputAction::Queue, click.modifiers);
        outbox.give_each(player, orders, queue);
    }
}
