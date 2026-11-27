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
use crate::hud::minimap::MinimapPointer;
use crate::input::{ActionState, Bindings, InputAction, MapClick, Modifiers};
use crate::selection::LocalControl;
use crate::tuning::ClientTuning;

use outbox::OrderOutbox;
pub(super) use panel::{cancel_on_escape, press_command_slots, validate_interface};
pub(super) use placing::{end_queued_placement, place_on_click, report_placing, turn_ghost};

/// `systems/controls/interface.md > AwaitingTarget > Dragging while targeting` (design change 0068): "A
/// targeting click counts at the press point". The target found under a press of the Select action that went down on
/// the map while a target was awaited, as the frame of the press drew the objects; its click, at the release, uses it.
/// Cleared when the press ends or targeting does.
#[derive(Resource, Default, Debug)]
pub(super) struct TargetPress(Option<PressedTarget>);

/// Where a press of the Select action went down: on the map (`Pointer::press`), or (slice M5-MM) on the minimap
/// (`crate::hud::minimap::MinimapPointer::press`), whose clicks are on the ground alone.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum PressedOn {
    Map,
    Minimap,
}

/// A press of the Select action that ended this frame, on the map or on the minimap: where it went down and how far
/// it moved.
#[derive(Copy, Clone, Debug, PartialEq)]
struct Released {
    on: PressedOn,
    screen: Vec2,
    moved: f32,
}

impl Released {
    /// The press that ended this frame, on the map or on the minimap: a Select press either way.
    fn of(actions: &ActionState, minimap: &MinimapPointer) -> Option<Self> {
        let map = actions.pointer().released.filter(|r| r.action == InputAction::Select);
        let map = map.map(|r| (PressedOn::Map, r.screen, r.moved));
        let minimap = minimap.released.map(|r| (PressedOn::Minimap, r.screen, r.moved));
        map.or(minimap).map(|(on, screen, moved)| Self { on, screen, moved })
    }
}

/// What was under a press while a target was awaited ([`TargetPress`]).
#[derive(Copy, Clone, Debug, PartialEq)]
struct PressedTarget {
    /// The command awaiting its target as the button went down.
    pending: PanelCommand,
    /// Where it went down: on the map or on the minimap, at this point (logical pixels), the press's own.
    on: PressedOn,
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
    /// `allowance` (`InputTuning::targeting_click_allowance_px`). A drag beyond it does nothing (design changes 0067
    /// and 0073).
    fn clicked(&self, released: Released, allowance: f32) -> bool {
        released.on == self.on && released.screen == self.screen && released.moved <= allowance
    }
}

/// `systems/controls/interface.md > AwaitingTarget > Shift while targeting` (design change 0070): once a
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

/// `systems/controls/interface.md > AwaitingTarget`: a left-click on the map (a press and release of the Select
/// action) on a valid target gives the PendingCommand's order for what is under it ([`target_order`]) to the command's
/// recipients ([`recipients`]: the whole selection for a common command, the ActiveGroup for a group one, every
/// selected unit for an Attack), each as [`unit_target_order`] has it: a unit that can't attack moves to the object of
/// an explicit Attack (design change 0398). A plain click then returns to DefaultState. A Shift-click queues the order
/// and keeps the target awaited,
/// marked queued, until Shift is let go ([`end_queued_targeting`], design change 0070); each one appends one order to
/// each unit's queue.
///
/// - **The press point** (design change 0068): the target is what was under the cursor when the button went down,
///   as that frame drew it ([`TargetPress`]), even if the cursor drifted or the target moved before the release.
/// - **The click allowance** (design change 0073): the press is a click if it moved no further than
///   `InputTuning::targeting_click_allowance_px` before its release (`Pointer::released`), more than the normal drag
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
/// - **A Shrine's cast** (slice M5-N5, `factions/cults/structures/shrine.md > Placeholder stats`: "Targeting:
///   AwaitingTarget, left-click a point in vision"): a click on a point the player sees now casts the ability there
///   from the one Shrine of the ActiveGroup, as one `Command::Cast`, and returns to DefaultState; a Shift-click too,
///   like a rally point's (`PanelCommand::queues`). A point out of vision is an invalid target.
/// - **On the minimap** (slice M5-MM, `systems/controls/hud.md > Minimap > Using it`: "a left-click on the minimap
///   targets that ground point (e.g. Attack Move there), as a left-click on ground would"): a press that goes down on
///   the minimap is a targeting click on the ground point it shows there (`MapTarget::on_ground`: the designer's
///   placeholder, ground only), by the same rules; the command flow takes it, so that it moves no camera
///   (`crate::hud::minimap::MinimapPress::targets`).
pub(super) fn click_targets(
    actions: Res<ActionState>,
    tuning: Res<ClientTuning>,
    mut minimap: ResMut<MinimapPointer>,
    mut pressed: ResMut<TargetPress>,
    mut control: LocalControl,
    mut outbox: OrderOutbox,
    mut invalid: MessageWriter<InvalidTarget>,
) {
    let awaited = control.state().interface.pending();
    if let Some(pending) = awaited
        && actions.just_pressed(InputAction::Select)
    {
        let (sim, player) = (control.sim(), control.player());
        let on_map = actions.pointer().press.filter(|p| p.action == InputAction::Select);
        let press = if let Some(press) = on_map {
            let target = MapTarget::at(sim, player, &control.objects(), press.view, press.ground);
            Some((PressedOn::Map, press.screen, target))
        } else if let Some(press) = minimap.press.as_mut() {
            press.targets = true;
            Some((
                PressedOn::Minimap,
                press.screen,
                MapTarget::on_ground(sim, player, press.ground),
            ))
        } else {
            None
        };
        if let Some((on, screen, target)) = press {
            pressed.0 = Some(PressedTarget {
                pending,
                on,
                screen,
                target,
                queue: actions.held(InputAction::Queue),
            });
        }
    }
    if let Some(released) = Released::of(&actions, &minimap)
        && let Some(click) = pressed
            .0
            .take()
            .filter(|p| awaited == Some(p.pending) && p.clicked(released, tuning.input.targeting_click_allowance_px))
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
    if awaited.is_none() || (actions.pointer().press.is_none() && minimap.press.is_none()) {
        pressed.0 = None;
    }
}

/// Gives what the targeting click `click` resolves to ([`target_resolution`]) to its command's recipients: an order
/// to each unit as [`unit_target_order`] has it, queued if it was a Shift-click, the rally point to all the
/// producers, the ability to all the units, or (M5-N5) the cast to the Shrine. Whether anything was given: nothing
/// for a target the command can't use, nor when the sim would take none of it.
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
                    Some((id, unit_target_order(unit, kind)))
                })
                .collect();
            outbox.give_each(control.player(), orders, click.queue)
        }
        Some(Resolved::Rally(target)) => {
            outbox.produce(control.player(), recipients, ProductionCommand::SetRally(target))
        }
        Some(Resolved::Ability(ability)) => outbox.use_ability(control.player(), recipients, ability),
        Some(Resolved::Cast(ability, at)) => outbox.cast(control.player(), recipients, ability, at),
    }
}

/// `systems/controls/interface.md > RightClickResolution`: a right-click on the map (the ContextCommand action's click)
/// in DefaultState, or in a menu. Each selected object resolves it for itself ([`object_right_click`], design changes
/// 0062 and 0063), whichever group is active: each unit gets its default command on what was clicked, as the player
/// knows it: an object it sees or one it remembers (design change 0139). The units that can carry out the resolved
/// command do, the others move to the location or follow the unit under the cursor, and so do the Peacekeepers
/// (Shielded), instead of attacking an enemy: they follow a unit and move to a structure
/// (`super::resolution::unit_right_click`, design changes 0074 and 0202). A unit right-clicked among the selected ones
/// stops, and the others follow it (design change 0168: "Following itself = Stop", the sim's). The orders go out
/// grouped ([`OrderOutbox::give_each`]). The selected producers set their rally point on what was clicked, as one
/// `Command::Production` (`systems/controls/interface.md > Production Building Default Right-Click`; on a producer
/// itself, which clears its rally point: design change 0086), whatever the Queue modifier. Objects without commands do
/// nothing. Nothing unless the selection is all the player's (`systems/controls/interface.md > CommandPanel`).
///
/// Design change 0400 (`systems/controls/interface.md > CommandPanel > Standard Slot Assignments > Right-click in
/// multi-stage menus`): "right-click **doesn't reset a menu**". In a menu where the next step is choosing an option (a
/// build menu, the Tunnel's EjectMenu and ExpandMenu, a KindMenu, the Altar's AugmentMenus) the right-click is the
/// same as in DefaultState, and "the menu **stays open**". "Only when the next step is a **left-click on a location or
/// target** (AwaitingTarget, AwaitingPlacement) does a right-click cancel, and it goes **straight back to
/// DefaultState** (not to the previous menu)": a right-click anywhere, over the HUD too, then orders nothing and
/// returns to DefaultState ([`InterfaceState::takes_map_clicks`]). A building ready to place stays ready. Escape and
/// Back (Z) still go back one step ([`InterfaceState::back`]).
///
/// Slice M5-MM (`systems/controls/hud.md > Minimap > Using it`): a right-click on the minimap is "the selection's
/// right-click on that ground point, as a right-click on ground in the main view", the same function's
/// ([`right_click`]) on the ground point the minimap shows there (`MapTarget::on_ground`, the designer's placeholder:
/// ground only); in AwaitingTarget and AwaitingPlacement it cancels, as anywhere.
pub(super) fn right_click_orders(
    actions: Res<ActionState>,
    mut clicks: MessageReader<MapClick>,
    minimap: Res<MinimapPointer>,
    bindings: Res<Bindings>,
    mut control: LocalControl,
    mut outbox: OrderOutbox,
) {
    let clicks: Vec<MapClick> = clicks
        .read()
        .filter(|click| click.action == InputAction::ContextCommand)
        .copied()
        .collect();
    if control.state().interface.takes_map_clicks() {
        if actions.just_pressed(InputAction::ContextCommand) {
            control.update(|state| state.interface = InterfaceState::DefaultState);
        }
        return;
    }
    let (sim, player) = (control.sim(), control.player());
    let on_map = clicks.iter().map(|click| {
        let target = MapTarget::at(sim, player, &control.objects(), click.view, click.ground);
        (target, click.modifiers)
    });
    let on_minimap =
        (minimap.right_click).map(|click| (MapTarget::on_ground(sim, player, click.ground), click.modifiers));
    let targets: Vec<(MapTarget, Modifiers)> = on_map.chain(on_minimap).collect();
    for (target, modifiers) in targets {
        if !right_click(
            &control,
            &mut outbox,
            &target,
            bindings.holds(InputAction::Queue, modifiers),
        ) {
            return;
        }
    }
}

/// One right-click on `target`, queued or not, as [`right_click_orders`] has it: each selected object's own
/// resolution, the units' orders grouped and the producers' rally point. Whether the selection may be commanded at
/// all: nothing is given otherwise.
fn right_click(control: &LocalControl, outbox: &mut OrderOutbox, target: &MapTarget, queue: bool) -> bool {
    let state = control.state();
    let player = control.player();
    if !commandable(control.sim(), player, &state.selection) {
        return false;
    }
    let modes = Modes::of(control.sim(), &state.selection);
    let mut orders: Vec<(ObjectId, OrderKind)> = Vec::new();
    let mut rally: Option<(Target, Vec<ObjectId>)> = None;
    for (id, object) in capabilities(&state.selection, &modes) {
        match object_right_click(object, target) {
            Some(Resolved::Order(kind)) => orders.push((id, kind)),
            // Every producer rallies on the same target: what was clicked.
            Some(Resolved::Rally(on)) => rally.get_or_insert_with(|| (on, Vec::new())).1.push(id),
            // No right-click uses an ability, nor casts.
            Some(Resolved::Ability(_) | Resolved::Cast(..)) | None => {}
        }
    }
    outbox.give_each(player, orders, queue);
    if let Some((on, producers)) = rally {
        outbox.produce(player, producers, ProductionCommand::SetRally(on));
    }
    true
}
