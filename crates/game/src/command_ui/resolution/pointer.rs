//! The pointer's preview of a click (`systems/controls/hud.md > PointerDisplayType`): in DefaultState the
//! one thing the selected objects would all do on a right-click, or Mixed when they would do different things; while a
//! target is awaited, the pending command's look over a valid target, else Inactive ([`pointer_display`]). Split
//! from `super` (the resolution of the clicks themselves), whose rules it previews.

use std::collections::BTreeSet;

use sc_sim::{Ability, OrderKind};

use super::{CursorTarget, MapTarget, Resolved, UnitCapabilities, object_right_click, target_resolution};
use crate::command_ui::interface::{InterfaceState, PanelCommand};

/// `systems/controls/hud.md > PointerDisplayType > Types` that the game can show so far.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PointerDisplay {
    /// Nothing will happen on a click.
    Inactive,
    Move,
    Attack,
    AttackGround,
    Patrol,
    /// "Unit will gather from this resource" (slice M3-J2).
    GatherResources,
    /// "Unit will return carried resources to this drop-off point" (slice M3-J2).
    ReturnResources,
    /// "Unit will enter this object" (slice M4-J1): "Cursor over own Tunnel (Syndicate unit, tier sufficient)", the
    /// sim's Enter (`sc_sim::OrderKind::Enter`): an Agent that carries nothing, or any unit the Tunnel's tier lets
    /// into the network, over an own Tunnel in operation; never over a site, which takes no Enter (design change 0237).
    /// The Recruits' too, over an own building in operation that takes them in, the Armory (slice M5-J1;
    /// `factions/cults/units/recruit.md > Right-Click Resolution`: "Own Armory → Enter Armory"), and never over its
    /// site (design change 0347 point 4).
    Enter,
    /// `systems/controls/hud.md > Mixed Right-Click Preview` (design change 0063): the selected units
    /// would do different things on a right-click.
    Mixed,
    /// (M5-J1) The Recruits' Assist Construction (design change 0322, a placeholder look: `crate::command_ui::cursor`):
    /// over an own building under construction that the selection builds from outside, on a right-click
    /// (`factions/cults/units/recruit.md > Right-Click Resolution`: "In-progress own building → Assist Construction")
    /// and while S awaits its target.
    Assist,
    /// (M5-J1) The Recruits' Repair (design change 0322, a placeholder look): over a damaged own building or vehicle
    /// the selection can repair. Its hook: no order gives it until slice N8 brings `OrderKind`'s Repair, which
    /// [`PointerDisplay::of_order`] then shows so, on a right-click and while D awaits its target.
    Repair,
}

impl PointerDisplay {
    /// The PointerDisplayType of an order a click gives: Attack for an AttackMove too
    /// (`systems/controls/hud.md > Resolution Rules > AwaitingTarget[CommandType]`: Attack over ground
    /// "will issue AttackMove"); GatherResources for a Gather ("Cursor over resource (resource gatherer selected)"),
    /// ReturnResources for a Drop Off ("Cursor over drop-off point (resource gatherer carrying resources)"), Enter for
    /// an Enter ("Cursor over own Tunnel (Syndicate unit, tier sufficient)"), Assist for an Assist (design change
    /// 0322). No click gives Hold or Stop.
    pub fn of_order(kind: OrderKind) -> Self {
        match kind {
            // No click gives a Deploy (slice M3-J1 places it), nor a Build.
            OrderKind::Move(_) | OrderKind::Deploy { .. } | OrderKind::Build { .. } => PointerDisplay::Move,
            OrderKind::Enter(_) => PointerDisplay::Enter,
            OrderKind::Assist(_) => PointerDisplay::Assist,
            OrderKind::Gather(_) => PointerDisplay::GatherResources,
            OrderKind::DropOff(_) => PointerDisplay::ReturnResources,
            OrderKind::Attack(_) | OrderKind::AttackMove(_) => PointerDisplay::Attack,
            OrderKind::Patrol(_) => PointerDisplay::Patrol,
            OrderKind::AttackGround(_) => PointerDisplay::AttackGround,
            OrderKind::Hold | OrderKind::Stop => PointerDisplay::Inactive,
        }
    }

    /// The PointerDisplayType of what a click gives an object: its order's, or Move for a rally point
    /// (`systems/controls/hud.md > Resolution Rules > DefaultState`: "Selection is a production building:
    /// Move (rally point)"; `systems/controls/hud.md > Resolution Rules > AwaitingTarget[CommandType]`:
    /// "AwaitingTarget\[SetRallyPoint\] over ground or object: Move"). Set Drop-off over a drop-off shows
    /// ReturnResources: **placeholder**, the rules have no AwaitingTarget\[SetDropOffLocation\] row, and its target is
    /// the drop-off point the loads will return to.
    pub fn of(resolved: Resolved) -> Self {
        match resolved {
            Resolved::Order(kind) => Self::of_order(kind),
            Resolved::Rally(_) => PointerDisplay::Move,
            Resolved::Ability(Ability::SetDropOff(_)) => PointerDisplay::ReturnResources,
            Resolved::Ability(_) => PointerDisplay::Inactive,
        }
    }
}

/// `systems/controls/hud.md > PointerDisplayType > Resolution Rules` over `target`, for the selected
/// `units` (none: nothing commandable is selected, `systems/controls/interface.md > CommandPanel`). In a build
/// menu, or a Tunnel's EjectMenu or ExpandMenu, a right-click only goes back: Inactive (placeholder: the rules list no
/// menu). AwaitingPlacement has "No pointer display type — the building ghost preview serves as the cursor"
/// (`crate::command_ui::cursor` shows none): Inactive here.
pub fn pointer_display(interface: InterfaceState, units: &[UnitCapabilities], target: &MapTarget) -> PointerDisplay {
    if units.is_empty() {
        return PointerDisplay::Inactive;
    }
    match interface {
        InterfaceState::DefaultState => right_click_display(units, target),
        InterfaceState::AwaitingTarget { pending, .. } => awaiting_display(pending, target),
        InterfaceState::BuildMenu(_)
        | InterfaceState::EjectMenu
        | InterfaceState::ExpandMenu
        | InterfaceState::AwaitingPlacement(_) => PointerDisplay::Inactive,
    }
}

/// `systems/controls/hud.md > Resolution Rules > DefaultState` and
/// `systems/controls/hud.md > Mixed Right-Click Preview` (design change 0063): the right-click preview,
/// from the action each of `units` would take ([`object_right_click`]). One action shows its pointer, more than one
/// Mixed, none Inactive. An action is a PointerDisplayType, so a Move to the location, a Follow and a rally point are
/// all Move.
pub fn right_click_display(units: &[UnitCapabilities], target: &MapTarget) -> PointerDisplay {
    let actions: BTreeSet<PointerDisplay> = units
        .iter()
        .filter_map(|&unit| object_right_click(unit, target))
        .map(PointerDisplay::of)
        .collect();
    let mut actions = actions.into_iter();
    match (actions.next(), actions.next()) {
        (None, _) => PointerDisplay::Inactive,
        (Some(action), None) => action,
        (Some(_), Some(_)) => PointerDisplay::Mixed,
    }
}

/// `systems/controls/hud.md > Resolution Rules > AwaitingTarget[CommandType]`: the pending command's look
/// over a target it can use, else Inactive. Reverse, which has no order yet, by the design's row for the ground.
fn awaiting_display(pending: PanelCommand, target: &MapTarget) -> PointerDisplay {
    match (pending, target.target) {
        (PanelCommand::Reverse, CursorTarget::Ground(_)) => PointerDisplay::Move,
        _ => target_resolution(pending, target).map_or(PointerDisplay::Inactive, PointerDisplay::of),
    }
}
