//! The command flow's systems: the frame's input (`ARCHITECTURE.md > Client architecture > Input`) changes the local
//! player's ObjectInterfaceState and gives orders, by the rules of [`super::interface`] and [`super::resolution`].
//!
//! Orders go to the sim as `Command::Order` in `PendingCommands`, named for the local player's own units only, and
//! only when the sim would take them now (`Sim::validate`); each one given is also an [`OrderIssued`] message, for
//! the indicators. A click whose units get different orders gives one order per distinct order
//! ([`super::resolution::group_orders`]). A command issued with the Queue modifier held (Shift) is queued
//! (`systems/controls/commands.md > Unit Command`), else it replaces the units' orders. Abilities (the Riot Shield,
//! Convert) go as `Command::Ability`, at once and never queued. A producer's commands
//! (`factions/gdo/structures/barracks.md > ObjectInterfaceState[Barracks]`) go as one `Command::Production` to all
//! the producers they are issued to, also never queued: Train and Cancel Production at once, the rally point on a
//! right-click or a click while Set Rally Point awaits its target. The sim decides which producers a Train reaches
//! (design change 0083); the client sends it to all of them (no smart-casting, design changes 0071 and 0075). A
//! constructor's commands (`factions/gdo/structures/deployment_center.md > ObjectInterfaceState[DeploymentCenter]`,
//! `factions/gdo/economy.md > ExtractionFacility > ObjectInterfaceState[ExtractionFacility]`) go as
//! `Command::Construction` to the one constructor of the ActiveGroup, never queued: Construct and the Cancels at
//! once, Place on a left-click in AwaitingPlacement, only where the placement check allows it now. A Beacon's Warp In
//! goes the same way to the Beacon, a transforming object's Cancel to the ActiveGroup, and a Deploy as an order to
//! one Drone of the ActiveGroup (slice M3-J1).
//!
//! The systems are split by what they handle: the panel's ([`panel`](mod@panel): the state validated, Esc, the command
//! slots), AwaitingPlacement's ([`placing`]), and here the targeting clicks and the right-click. All of them give
//! their orders through the [`outbox`].

mod outbox;
mod panel;
mod placing;

use bevy::prelude::*;
use sc_sim::{ObjectId, OrderKind, ProductionCommand, Target};

use super::InvalidTarget;
use super::interface::{InterfaceState, Modes, PanelCommand, commandable, recipients};
use super::resolution::{
    MapTarget, Resolved, UnitCapabilities, capabilities, object_right_click, target_resolution, unit_target_order,
};
use crate::input::{ActionState, Bindings, InputAction, MapClick, MapPress};
use crate::selection::LocalControl;

use outbox::OrderOutbox;
pub(super) use panel::{cancel_on_escape, press_command_slots, validate_interface};
pub(super) use placing::{end_queued_placement, place_on_click, report_placing, turn_ghost};

pub use crate::tuning::input::TARGETING_CLICK_ALLOWANCE_PX;

/// `systems/controls/selection_and_hud.md > AwaitingTarget > Dragging while targeting` (design change 0068): "A
/// targeting click counts at the press point". The target found under a press of the Select action that went down on
/// the map while a target was awaited, as the frame of the press drew the objects; its click, at the release, uses it.
/// Cleared when the press ends or targeting does.
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

/// `systems/controls/selection_and_hud.md > AwaitingTarget > Shift while targeting` (design change 0070): once a
/// Shift-click has queued the command (`InterfaceState::AwaitingTarget::queued`), letting go of Shift, the Queue
/// modifier, ends targeting: back to DefaultState. A targeting press still held goes first: its click counts at its
/// release, a Shift-click if Shift was held as the button went down ([`TargetPress`]), and targeting ends after it.
/// Before the first queued click, letting go of Shift changes nothing (the design's TBD, its proposed default:
/// targeting simply continues). It runs before the other input systems, so that a command key pressed as Shift is let
/// go starts a new targeting.
pub(super) fn end_queued_targeting(actions: Res<ActionState>, pressed: Res<TargetPress>, mut control: LocalControl) {
    let queued = matches!(
        control.state().interface,
        InterfaceState::AwaitingTarget { queued: true, .. }
    );
    if queued && !actions.held(InputAction::Queue) && pressed.0.is_none() {
        control.update(|state| state.interface = InterfaceState::DefaultState);
    }
}

/// `systems/controls/selection_and_hud.md > AwaitingTarget`: a left-click on the map (a press and release of the Select
/// action) on a valid target gives the PendingCommand's order for what is under it ([`target_order`]) to the command's
/// recipients ([`recipients`]: the whole selection for a common command, the ActiveGroup for a group one), each as
/// [`unit_target_order`] has it: an explicit Attack leaves out the Peacekeepers (Shielded) (design changes 0075 and
/// 0202). A plain click then returns to DefaultState. A Shift-click queues the order and keeps the target awaited,
/// marked queued, until Shift is let go ([`end_queued_targeting`], design change 0070); each one appends one order to
/// each unit's queue.
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
/// - **Set Rally Point** (`factions/gdo/structures/barracks.md`: "Left-click ground or object sets the rally
///   point (CommandIssuingTransition, returns to DefaultState)"): the rally point goes to the command's recipients,
///   every selected producer of the ActiveGroup, as one `Command::Production`; a click on the producer itself clears
///   its rally point (design change 0086). A rally point isn't queued, so a Shift-click sets it and ends the
///   targeting like a plain click ([`PanelCommand::queues`], a placeholder reading).
/// - **Set Drop-off** (`factions/colonists/units/drone.md > Drone ObjectInterfaceState > S: Set Drop-off Location`,
///   slice M3-J2): a click on an own Egress Node makes it the AssignedDropOff of every selected Drone, as one
///   `Command::Ability` (question Q0040 point 9), and returns to DefaultState ("CommandIssuingTransition"); a
///   Shift-click too, like a rally point's. Their orders go on (`sc_sim::Ability::SetDropOff`: "Does not interrupt
///   current mining/movement"). Anything else is an invalid target.
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
        let player = control.player();
        let target = MapTarget::at(control.sim(), player, &control.objects(), press.view, press.ground);
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
/// to each unit as [`unit_target_order`] has it, queued if it was a Shift-click, the rally point to all the
/// producers, or the ability to all the units. Whether anything was given: nothing for a target the command can't
/// use, nor when the sim would take none of it.
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
        Some(Resolved::Ability(ability)) => outbox.use_ability(control.player(), recipients, ability),
    }
}

/// `systems/controls/selection_and_hud.md > RightClickResolution`: a right-click on the map (the ContextCommand
/// action's click) in DefaultState. Each selected object resolves it for itself ([`object_right_click`], design changes
/// 0062 and 0063), whichever group is active: each unit gets its default command on what was clicked, as the player
/// knows it: an object it sees or one it remembers (design change 0139). The units that can carry out the resolved
/// command do, the others move to the location or follow the unit under the cursor, and so do the Peacekeepers
/// (Shielded), instead of attacking an enemy: they follow a unit and move to a structure
/// (`super::resolution::unit_right_click`, design changes 0074 and 0202).
/// A unit right-clicked among the selected ones stops, and the others follow it (design change 0168: "Following
/// itself = Stop", the sim's). The orders go out
/// grouped ([`OrderOutbox::give_each`]). The selected producers set their rally point on what was clicked, as one
/// `Command::Production` (`systems/controls/selection_and_hud.md > Production Building Default Right-Click`; on a
/// producer itself, which clears its rally point: design change 0086), whatever the Queue modifier. Objects without
/// commands do nothing. Nothing unless the selection is all the player's (`systems/controls/selection_and_hud.md >
/// CommandPanel`). While a target is awaited, a right-click anywhere, over the HUD too, cancels instead
/// (`systems/controls/selection_and_hud.md > AwaitingTarget`), and in the other multi-stage states it goes back one
/// state (`InterfaceState::back`: "Escape/right-click: returns to BuildMenu" from the Deployment Center's
/// AwaitingPlacement, "returns to DefaultState" from a build menu and from the Extraction Facility's
/// AwaitingPlacement).
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
        let target = MapTarget::at(control.sim(), player, &control.objects(), click.view, click.ground);
        let modes = Modes::of(control.sim(), &state.selection);
        let mut orders: Vec<(ObjectId, OrderKind)> = Vec::new();
        let mut rally: Option<(Target, Vec<ObjectId>)> = None;
        for (id, object) in capabilities(&state.selection, &modes) {
            match object_right_click(object, &target) {
                Some(Resolved::Order(kind)) => orders.push((id, kind)),
                // Every producer rallies on the same target: what was clicked.
                Some(Resolved::Rally(on)) => rally.get_or_insert_with(|| (on, Vec::new())).1.push(id),
                // No right-click uses an ability.
                Some(Resolved::Ability(_)) | None => {}
            }
        }
        let queue = bindings.holds(InputAction::Queue, click.modifiers);
        outbox.give_each(player, orders, queue);
        if let Some((on, producers)) = rally {
            outbox.produce(player, producers, ProductionCommand::SetRally(on));
        }
    }
}
