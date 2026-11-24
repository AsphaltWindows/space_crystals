//! The command flow's systems: the frame's input (`ARCHITECTURE.md > Client architecture > Input`) changes the local
//! player's ObjectInterfaceState and gives orders, by the rules of [`super::interface`] and [`super::resolution`].
//!
//! Orders go to the sim as `Command::Order` in `PendingCommands`, named for the local player's own units only, and
//! only when the sim would take them now (`Sim::validate`); each one given is also an [`OrderIssued`] message, for
//! the indicators. A click whose units get different orders gives one order per distinct order
//! ([`super::resolution::group_orders`]). A command issued with the Queue modifier held (Shift) is queued
//! (`systems/controls/commands.md > Unit Command`), else it replaces the units' orders. Abilities (the Riot Shield,
//! Convert) go as `Command::Ability`, at once and never queued. A producer's commands
//! (`factions/gdo/structures.md > Barracks > ObjectInterfaceState[Barracks]`) go as one `Command::Production` to all
//! the producers they are issued to, also never queued: Train and Cancel Production at once, the rally point on a
//! right-click or a click while Set Rally Point awaits its target. The sim decides which producers a Train reaches
//! (design change 0083); the client sends it to all of them (no smart-casting, design changes 0071 and 0075). A
//! constructor's commands (`factions/gdo/structures.md > DeploymentCenter > ObjectInterfaceState[DeploymentCenter]`,
//! `factions/gdo/economy.md > ExtractionFacility > ObjectInterfaceState[ExtractionFacility]`) go as
//! `Command::Construction` to the one constructor of the ActiveGroup, never queued: Construct and the Cancels at
//! once, Place on a left-click in AwaitingPlacement, only where the placement check allows it now.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use sc_sim::{
    Ability, Command, ConstructionCommand, ObjectId, OrderKind, PlayerCommand, PlayerId, ProductionCommand, Target,
};

use super::interface::{
    Conditions, InterfaceState, Modes, PanelCommand, Placing, SlotPress, commandable, panel, press, recipients,
    still_valid,
};
use super::resolution::{
    MapTarget, Resolved, UnitCapabilities, capabilities, group_orders, object_right_click, target_resolution,
    unit_target_order,
};
use super::{InvalidPlacement, InvalidTarget, OrderIssued};
use crate::ClientMode;
use crate::editor::orientation::Turn;
use crate::hud::layout::CommandGrid;
use crate::input::{self, ActionState, Bindings, CommandSlot, InputAction, MapClick, MapPress};
use crate::platform::SurfaceSize;
use crate::selection::{ControlStates, LocalControl};
use crate::sim_driver::{LocalPlayer, PendingCommands, SimRes};

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

    /// `player`'s producers among `objects` carry out `command` (`Command::Production`), one command for all of
    /// them, at once: production isn't queued. Nothing when none of them is the player's producer, or when the sim
    /// would reject it (`Sim::validate`, the predicates the panel greys its buttons out by: e.g. a full queue, too
    /// few Space Crystals, nothing to cancel, a rally point off the map, which a non-finite point is too).
    fn produce(&mut self, player: PlayerId, objects: Vec<ObjectId>, command: ProductionCommand) -> bool {
        let objects: Vec<ObjectId> = objects
            .into_iter()
            .filter(|&id| {
                self.sim
                    .object(id)
                    .is_some_and(|o| o.owner == Some(player) && o.production.is_some())
            })
            .collect();
        if objects.is_empty() {
            return false;
        }
        let sent = PlayerCommand {
            player,
            command: Command::Production { objects, command },
        };
        if let Err(reject) = self.sim.validate(&sent) {
            debug!("production command {command:?} not given: {reject}");
            return false;
        }
        self.pending.0.push(sent);
        true
    }

    /// `player`'s constructors among `objects` carry out `command` (`Command::Construction`), one command for all of
    /// them, at once: construction isn't queued. Nothing when none of them is the player's constructor, or when the
    /// sim would reject it (`Sim::validate`, the predicates the panel greys its buttons out by: e.g. too few Space
    /// Crystals, a constructor already busy, nothing to cancel; a Place the placement check refuses).
    fn construct(&mut self, player: PlayerId, objects: Vec<ObjectId>, command: ConstructionCommand) -> bool {
        let objects: Vec<ObjectId> = objects
            .into_iter()
            .filter(|&id| {
                self.sim
                    .object(id)
                    .is_some_and(|o| o.owner == Some(player) && o.constructor.is_some())
            })
            .collect();
        if objects.is_empty() {
            return false;
        }
        let sent = PlayerCommand {
            player,
            command: Command::Construction { objects, command },
        };
        if let Err(reject) = self.sim.validate(&sent) {
            debug!("construction command {command:?} not given: {reject}");
            return false;
        }
        self.pending.0.push(sent);
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
        OrderKind::Gather(_) | OrderKind::DropOff(_) | OrderKind::Deploy { .. } => true,
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
    pending: PanelCommand,
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
/// game state": an AwaitingTarget whose command the ActiveGroup no longer offers, a build menu whose ActiveGroup is
/// no Deployment Center, an AwaitingPlacement whose constructor no longer holds the building ready, or any of them
/// whose selection is no longer commandable (no longer all the player's, or the game is over: design change 0101),
/// goes back to DefaultState. A selected unit that died has already left the selection (`crate::selection`), and with
/// the ActiveGroup left the target is still awaited (design change 0069).
pub(super) fn validate_interface(mut control: LocalControl) {
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

/// `> AwaitingTarget`: "Escape or right-click cancels via a StateOnlyTransition back to DefaultState"; in the other
/// multi-stage states Esc goes back one state (`InterfaceState::back`): from AwaitingPlacement to the Deployment
/// Center's build menu or the Extraction Facility's DefaultState, from a build menu to DefaultState. Esc is the
/// Cancel action; right-click is [`right_click_orders`], Back (Z) [`press_command_slots`].
pub(super) fn cancel_on_escape(actions: Res<ActionState>, mut control: LocalControl) {
    let state = control.state();
    if actions.just_pressed(InputAction::Cancel) && state.interface != InterfaceState::DefaultState {
        let back = state.interface.back(&state.selection);
        control.update(|state| state.interface = back);
    }
}

/// `> DefaultState`: a command hotkey (`InputAction::CommandSlot`) or a click on a panel button (a press of the
/// Select action over the button, found by the panel's geometry like the SelectionPanel's portraits) enters
/// AwaitingTarget for a command that needs a target, or issues an immediate one: a common command to the whole
/// selection, a group command to the ActiveGroup (`> CommandPanel`), a Train or a Cancel Production to every
/// producer among them as one command. A constructor's enter its build menu (`InterfaceState::BuildMenu`) or the
/// placement of its building ready to place (`InterfaceState::AwaitingPlacement`), or give its one constructor a
/// construction command and return to DefaultState ("CommandIssuingTransition, returns to DefaultState"). Back (Z)
/// returns to the previous state. While a target is awaited or a building placed the other commands are unavailable
/// (design change 0066), and so is a production or construction command the gate would refuse (design changes 0081
/// and 0082): [`press`] finds nothing to do for them. Nothing while the selection isn't commandable (a defeated
/// player's, or anyone's once the game is over).
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
        let conditions = Conditions::of(control.sim(), player, &state.selection);
        match press(&panel(&state.selection, state.interface, &conditions), slot) {
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
            SlotPress::Back => {
                let back = state.interface.back(&state.selection);
                control.update(|state| state.interface = back);
            }
            SlotPress::Nothing => {}
        }
    }
}

/// `> AwaitingPlacement`: "R rotates the ghost 90 degrees clockwise, Shift+R rotates 90 degrees counter-clockwise.
/// [...] F flips the ghost horizontally, Shift+F flips vertically": the placement keys, which the input layer fires
/// in place of the command slots R and F while a building is placed ([`report_placing`]), the editor's own
/// (`InputAction::is_placement`, `Turn::of_action`).
pub(super) fn turn_ghost(actions: Res<ActionState>, mut control: LocalControl) {
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
///   placement then ends ([`validate_interface`]).
pub(super) fn place_on_click(
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
/// defeated player, during a playback), when the placement ends ([`validate_interface`]).
pub(super) fn report_placing(
    mode: Res<ClientMode>,
    player: Res<LocalPlayer>,
    states: Res<ControlStates>,
    mut placing: ResMut<input::Placing>,
) {
    if mode.is_game() {
        placing.set_if_neq(input::Placing(states.of(player.0).interface.placing().is_some()));
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
/// - **Set Rally Point** (`factions/gdo/structures.md > Barracks`: "Left-click ground or object sets the rally
///   point (CommandIssuingTransition, returns to DefaultState)"): the rally point goes to the command's recipients,
///   every selected producer of the ActiveGroup, as one `Command::Production`; a click on the producer itself clears
///   its rally point (design change 0086). A rally point isn't queued, so a Shift-click sets it and ends the
///   targeting like a plain click ([`PanelCommand::queues`], a placeholder reading).
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
        } else if click.queue && pending.queues() {
            control.update(|state| state.interface = InterfaceState::AwaitingTarget { pending, queued: true });
        } else {
            control.update(|state| state.interface = InterfaceState::DefaultState);
        }
    }
    if awaited.is_none() || actions.pointer().press.is_none() {
        pressed.0 = None;
    }
}

/// Gives what the targeting click `click` resolves to ([`target_resolution`]) to its command's recipients: an order
/// to each unit as [`unit_target_order`] has it, queued if it was a Shift-click, or the rally point to all the
/// producers. Whether anything was given: nothing for a target the command can't use, nor when the sim would take
/// none of it.
fn give_targeted(control: &LocalControl, outbox: &mut OrderOutbox, click: PressedTarget) -> bool {
    let state = control.state();
    let recipients = recipients(&state.selection, click.pending);
    match target_resolution(click.pending, &click.target) {
        None => false,
        Some(Resolved::Order(kind)) => {
            let modes = Modes::of(control.sim(), &state.selection);
            let orders = recipients
                .into_iter()
                .filter_map(|id| {
                    let unit = UnitCapabilities::of(id, state.selection.kind_of(id)?, &modes);
                    Some((id, unit_target_order(unit, kind)?))
                })
                .collect();
            outbox.give_each(control.player(), orders, click.queue)
        }
        Some(Resolved::Rally(target)) => {
            outbox.produce(control.player(), recipients, ProductionCommand::SetRally(target))
        }
    }
}

/// `> RightClickResolution`: a right-click on the map (the ContextCommand action's click) in DefaultState. Each
/// selected object resolves it for itself ([`object_right_click`], design changes 0062 and 0063), whichever group is
/// active: the units that can carry out the resolved command do, the others move to the location or follow the
/// unit under the cursor, and so do those whose Riot Shield is up, instead of attacking an enemy: they follow a unit
/// and move to a structure (`super::resolution::order_without_attack`, design change 0074). The orders go out
/// grouped ([`OrderOutbox::give_each`]). The selected producers set their rally point on what was clicked, as one
/// `Command::Production` (`> Production Building Default Right-Click`; on a producer itself, which clears its rally
/// point: design change 0086), whatever the Queue modifier. Objects without commands do nothing. Nothing unless the
/// selection is all the player's (`> CommandPanel`). While a target is awaited, a right-click anywhere, over the HUD
/// too, cancels instead (`> AwaitingTarget`), and in the other multi-stage states it goes back one state
/// (`InterfaceState::back`: "Escape/right-click: returns to BuildMenu" from the Deployment Center's AwaitingPlacement,
/// "returns to DefaultState" from a build menu and from the Extraction Facility's AwaitingPlacement).
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
    let state = control.state();
    if state.interface != InterfaceState::DefaultState {
        if actions.just_pressed(InputAction::ContextCommand) {
            let back = state.interface.back(&state.selection);
            control.update(|state| state.interface = back);
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
        let mut orders: Vec<(ObjectId, OrderKind)> = Vec::new();
        let mut rally: Option<(Target, Vec<ObjectId>)> = None;
        for (id, object) in capabilities(&state.selection, &modes) {
            match object_right_click(object, &target) {
                Some(Resolved::Order(kind)) => orders.push((id, kind)),
                // Every producer rallies on the same target: what was clicked.
                Some(Resolved::Rally(on)) => rally.get_or_insert_with(|| (on, Vec::new())).1.push(id),
                None => {}
            }
        }
        let queue = bindings.holds(InputAction::Queue, click.modifiers);
        outbox.give_each(player, orders, queue);
        if let Some((on, producers)) = rally {
            outbox.produce(player, producers, ProductionCommand::SetRally(on));
        }
    }
}
