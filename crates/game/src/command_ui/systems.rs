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
use super::resolution::{
    MapTarget, UnitCapabilities, capabilities, group_orders, target_order, unit_right_click, unit_target_order,
};
use super::{InvalidTarget, OrderIssued};
use crate::ClientMode;
use crate::hud::layout::CommandGrid;
use crate::input::{ActionState, Bindings, CommandSlot, InputAction, MapClick, MapPress};
use crate::platform::SurfaceSize;
use crate::selection::LocalControl;
use crate::sim_driver::{PendingCommands, SimRes};

/// `> AwaitingTarget > Dragging while targeting` (design change 0073): while a target is awaited, a press of the
/// Select action that moves at most this far (logical pixels) before its release is a targeting click, at the press
/// point; one that moves further is a drag, which does nothing (0067). The normal drag threshold
/// (`crate::input::DRAG_THRESHOLD_PX`) applies everywhere else. "16 pixels (placeholder, half a tile)": the one
/// value to tune.
pub const TARGETING_CLICK_ALLOWANCE_PX: f32 = 16.0;

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
/// awaited, as the frame of the press drew the objects; its click, at the release, uses it. Cleared when the press
/// ends or targeting does.
#[derive(Resource, Default, Debug)]
pub(super) struct TargetPress(Option<PressedTarget>);

/// What was under a press while a target was awaited ([`TargetPress`]).
#[derive(Copy, Clone, Debug, PartialEq)]
struct PressedTarget {
    /// The command awaiting its target as the button went down.
    pending: UnitCommand,
    /// Where it went down, logical pixels: the press's own (`MapPress::screen`).
    screen: Vec2,
    target: MapTarget,
    /// The Queue modifier (Shift) was held as the button went down: a Shift-click, which queues its order and keeps
    /// the target awaited (design change 0070). Taken at the press, where the click counts (0068), like a
    /// right-click's: a Shift let go just before the button still queues (placeholder: the design doesn't say when
    /// Shift counts).
    queue: bool,
}

impl PressedTarget {
    /// Whether `released`, the press that ended this frame, is this one, and a click: it moved no further than
    /// [`TARGETING_CLICK_ALLOWANCE_PX`]. A drag beyond it does nothing (design changes 0067 and 0073).
    fn clicked(&self, released: &MapPress) -> bool {
        released.action == InputAction::Select
            && released.screen == self.screen
            && released.moved <= TARGETING_CLICK_ALLOWANCE_PX
    }
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
                control.update(|state| state.interface = InterfaceState::awaiting(command));
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

/// `> AwaitingTarget > Shift while targeting` (design change 0070): once a Shift-click has queued the command
/// (`InterfaceState::AwaitingTarget::queued`), letting go of Shift, the Queue modifier, ends targeting: back to
/// DefaultState. A targeting press still held goes first: its click counts at its release, a Shift-click if Shift
/// was held as the button went down ([`TargetPress`]), and targeting ends after it. Before the first queued click,
/// letting go of Shift changes nothing (the design's TBD, its proposed default: targeting simply continues). It runs
/// before the other input systems, so that a command key pressed as Shift is let go starts a new targeting.
pub(super) fn end_queued_targeting(actions: Res<ActionState>, pressed: Res<TargetPress>, mut control: LocalControl) {
    let queued = matches!(
        control.state().interface,
        InterfaceState::AwaitingTarget { queued: true, .. }
    );
    if queued && !actions.held(InputAction::Queue) && pressed.0.is_none() {
        control.update(|state| state.interface = InterfaceState::DefaultState);
    }
}

/// `> AwaitingTarget`: a left-click on the map (a press and release of the Select action) on a valid target gives
/// the PendingCommand's order for what is under it ([`target_order`]) to the command's recipients ([`recipients`]:
/// the whole selection for a common command, the ActiveGroup for a group one), each as [`unit_target_order`] has it:
/// an explicit Attack leaves out the units whose Riot Shield is up (design change 0075). A plain click then returns
/// to DefaultState. A Shift-click queues the order and keeps the target awaited, marked queued, until Shift is let go
/// ([`end_queued_targeting`], design change 0070); each one appends one order to each unit's queue.
///
/// - **The press point** (design change 0068): the target is what was under the cursor when the button went down,
///   as that frame drew it ([`TargetPress`]), even if the cursor drifted or the target moved before the release.
/// - **The click allowance** (design change 0073): the press is a click if it moved no further than
///   [`TARGETING_CLICK_ALLOWANCE_PX`] before its release (`Pointer::released`), more than the normal drag
///   threshold. A press that moved further is a drag and does nothing (design change 0067), and so does a press
///   that went down before the target was awaited.
/// - **An invalid target** (design change 0065): a click that gives nothing, on a target the command can't use
///   (the Inactive pointer), is ignored and the target is still awaited, with Shift too; an [`InvalidTarget`]
///   message tells the player (`super::feedback`).
pub(super) fn click_targets(
    actions: Res<ActionState>,
    mut pressed: ResMut<TargetPress>,
    mut control: LocalControl,
    mut outbox: OrderOutbox,
    mut invalid: MessageWriter<InvalidTarget>,
) {
    let awaited = control.state().interface.pending();
    if let Some(pending) = awaited
        && actions.just_pressed(InputAction::Select)
        && let Some(press) = actions.pointer().press.filter(|p| p.action == InputAction::Select)
    {
        let target = MapTarget::at(control.sim(), &control.objects(), press.view, press.ground);
        pressed.0 = Some(PressedTarget {
            pending,
            screen: press.screen,
            target,
            queue: actions.held(InputAction::Queue),
        });
    }
    if let Some(released) = actions.pointer().released
        && let Some(click) = pressed
            .0
            .take()
            .filter(|p| awaited == Some(p.pending) && p.clicked(&released))
    {
        let pending = click.pending;
        if !give_targeted(&control, &mut outbox, click) {
            invalid.write(InvalidTarget {
                command: pending,
                tick: control.sim().tick(),
            });
        } else if click.queue {
            control.update(|state| state.interface = InterfaceState::AwaitingTarget { pending, queued: true });
        } else {
            control.update(|state| state.interface = InterfaceState::DefaultState);
        }
    }
    if awaited.is_none() || actions.pointer().press.is_none() {
        pressed.0 = None;
    }
}

/// Gives the order of the targeting click `click` to its command's recipients, each as [`unit_target_order`] has
/// it, queued if it was a Shift-click. Whether anything was given: nothing for a target the command can't use, nor
/// when the sim would take none of it.
fn give_targeted(control: &LocalControl, outbox: &mut OrderOutbox, click: PressedTarget) -> bool {
    let Some(kind) = target_order(click.pending, &click.target) else {
        return false;
    };
    let state = control.state();
    let modes = Modes::of(control.sim(), &state.selection);
    let orders = recipients(&state.selection, click.pending)
        .into_iter()
        .filter_map(|id| {
            let unit = UnitCapabilities::of(id, state.selection.kind_of(id)?, &modes);
            Some((id, unit_target_order(unit, kind)?))
        })
        .collect();
    outbox.give_each(control.player(), orders, click.queue)
}

/// `> RightClickResolution`: a right-click on the map (the ContextCommand action's click) in DefaultState. Each
/// selected unit resolves it for itself ([`unit_right_click`], design changes 0062 and 0063), whichever group is
/// active: those that can carry out the resolved command do, the others move to the location or follow the unit
/// under the cursor, and so do those whose Riot Shield is up, instead of attacking an enemy: they follow a unit and
/// move to a structure (`super::resolution::order_without_attack`, design change 0074). Objects without commands do
/// nothing. The orders go out grouped
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
