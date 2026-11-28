//! The pointer's preview of a click (`systems/controls/hud.md > PointerDisplayType`): in DefaultState the
//! one thing the selected objects would all do on a right-click, or Mixed when they would do different things; while a
//! target is awaited, the pending command's look over a valid target, else Inactive ([`pointer_display`]). Split
//! from `super` (the resolution of the clicks themselves), whose rules it previews.

use std::collections::BTreeSet;

use sc_sim::{Ability, OrderKind};

use super::{MapTarget, Resolved, UnitCapabilities, object_right_click, repaired_by, target_resolution, tower_free};
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
    /// site (design change 0347 point 4); and the Garrison's four kinds' over an own Garrison (slice M5-N2: "Own
    /// Garrison → Enter Garrison"). (M5-S2b) Not the Recruits' over one full and damaged, which they repair (design
    /// change 0378).
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
    /// the selection can repair, and the Agents' over a damaged own Tunnel or Laser Turret (slice M5-N8,
    /// `sc_sim::OrderKind::Repair`), on a right-click and while D awaits its target. On a right-click, over a building
    /// the selection could enter only while it is full (slice M5-S2b; design change 0378: "a damaged enterable
    /// building that is **full at the moment of the click** resolves to **Repair** (the pointer shows Repair)").
    Repair,
    /// (M5-N5) A Shrine's cast (`factions/cults/structures/shrine.md > Placeholder stats`: "Targeting: AwaitingTarget,
    /// left-click a point in vision"), over a point the player sees now, while Q or W awaits it: **Placeholder (not in
    /// the design's Types: plan §N5's "the AwaitingTarget cursor, with a 4 GU ring")**, a look of its own
    /// (`crate::command_ui::cursor`), with the ability's circle on the ground around the point
    /// (`crate::command_ui::indicators::cast`). Over a point out of vision, Inactive (`> Types`: "hovering over an
    /// invalid target for the current context").
    Cast,
}

impl PointerDisplay {
    /// The PointerDisplayType of an order a click gives: Attack for an AttackMove too
    /// (`systems/controls/hud.md > Resolution Rules > AwaitingTarget[CommandType]`: Attack over ground
    /// "will issue AttackMove"); GatherResources for a Gather ("Cursor over resource (resource gatherer selected)"),
    /// ReturnResources for a Drop Off ("Cursor over drop-off point (resource gatherer carrying resources)"), Enter for
    /// an Enter ("Cursor over own Tunnel (Syndicate unit, tier sufficient)"), Assist for an Assist and Repair for a
    /// Repair (design change 0322). (M6-GH1) An aircraft's right-click on an own Airfield, a Rebase onto another or a
    /// Return to Base onto its own (`factions/gdo/structures/airfield.md > Assignment`: "right-click the other
    /// Airfield"), shows Enter: **Placeholder (not in the design: `systems/controls/hud.md > Types` has no row for it;
    /// the aircraft flies there to land, as a unit goes into what it enters)**; and so does a Supply Chopper's
    /// Attach to Tower, the same placeholder, its Pick Up Supplies GatherResources and its Drop Off Supplies
    /// ReturnResources, an Agent's on a station and at its Tunnel (placeholders: no row names them). No click gives
    /// Hold, Stop, (M5-N4b) a Shroud, (M6-GD) a Launch, (M6-GA) a vehicle deploy or (M6-ST1) an Unload All.
    pub fn of_order(kind: OrderKind) -> Self {
        match kind {
            // No click gives a Deploy (slice M3-J1 places it), nor a Build.
            OrderKind::Move(_) | OrderKind::Deploy { .. } | OrderKind::Build { .. } => PointerDisplay::Move,
            // (M5-V2) `systems/controls/hud.md`: "AwaitingTarget[Reverse] over ground: **Move**".
            OrderKind::Reverse(_) => PointerDisplay::Move,
            OrderKind::Enter(_) | OrderKind::Rebase(_) | OrderKind::ReturnToBase => PointerDisplay::Enter,
            OrderKind::AttachToTower(_) => PointerDisplay::Enter,
            OrderKind::Assist(_) => PointerDisplay::Assist,
            OrderKind::Repair(_) => PointerDisplay::Repair,
            OrderKind::Gather(_) | OrderKind::PickUpSupplies(_) => PointerDisplay::GatherResources,
            OrderKind::DropOff(_) | OrderKind::DropOffSupplies(_) => PointerDisplay::ReturnResources,
            // (M5-K) A Target Lock's click, R's (`systems/controls/hud.md` has no row for it): placeholder, Attack.
            OrderKind::Attack(_) | OrderKind::AttackMove(_) | OrderKind::TargetLock(_) => PointerDisplay::Attack,
            OrderKind::Patrol(_) => PointerDisplay::Patrol,
            OrderKind::AttackGround(_) => PointerDisplay::AttackGround,
            OrderKind::Hold | OrderKind::Stop | OrderKind::Shroud | OrderKind::Launch => PointerDisplay::Inactive,
            OrderKind::VehicleDeploy { .. } | OrderKind::UnloadAll => PointerDisplay::Inactive,
        }
    }

    /// The PointerDisplayType of what a click gives an object: its order's, or Move for a rally point
    /// (`systems/controls/hud.md > Resolution Rules > DefaultState`: "Selection is a production building:
    /// Move (rally point)"; `systems/controls/hud.md > Resolution Rules > AwaitingTarget[CommandType]`:
    /// "AwaitingTarget\[SetRallyPoint\] over ground or object: Move"). Set Drop-off over a drop-off shows
    /// ReturnResources: **placeholder**, the rules have no AwaitingTarget\[SetDropOffLocation\] row, and its target is
    /// the drop-off point the loads will return to. (M5-N5) A cast shows Cast. A Schedule Deliveries shows
    /// GatherResources (`systems/controls/hud.md > Resolution Rules > AwaitingTarget[CommandType]`:
    /// "AwaitingTarget\[ScheduleDeliveries\] over valid target: **GatherResources**").
    pub fn of(resolved: Resolved) -> Self {
        match resolved {
            Resolved::Order(kind) => Self::of_order(kind),
            Resolved::Rally(_) => PointerDisplay::Move,
            Resolved::Ability(Ability::SetDropOff(_)) => PointerDisplay::ReturnResources,
            Resolved::Ability(_) => PointerDisplay::Inactive,
            Resolved::Cast(..) => PointerDisplay::Cast,
            Resolved::Schedule(_) => PointerDisplay::GatherResources,
        }
    }
}

/// `systems/controls/hud.md > PointerDisplayType > Resolution Rules` over `target`, for the selected
/// `units` (none: nothing commandable is selected, `systems/controls/interface.md > CommandPanel`). In a build
/// menu, a Tunnel's EjectMenu or ExpandMenu, a pool building's KindMenu or an Altar's AugmentMenus the right-click
/// previewed as in DefaultState, since it does its normal right-click there (design change 0400; **reading**: the
/// rules list no menu). AwaitingPlacement has "No pointer display type — the building ghost preview serves as the
/// cursor" (`crate::command_ui::cursor` shows none): Inactive here.
pub fn pointer_display(interface: InterfaceState, units: &[UnitCapabilities], target: &MapTarget) -> PointerDisplay {
    if units.is_empty() {
        return PointerDisplay::Inactive;
    }
    match interface {
        InterfaceState::DefaultState
        | InterfaceState::BuildMenu(_)
        | InterfaceState::EjectMenu
        | InterfaceState::ExpandMenu
        | InterfaceState::KindMenu(_)
        | InterfaceState::AugmentMenu(_)
        | InterfaceState::AugmentStateMenu(..) => right_click_display(units, target),
        InterfaceState::AwaitingTarget { pending, .. } => awaiting_display(pending, units, target),
        InterfaceState::AwaitingPlacement(_) => PointerDisplay::Inactive,
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
/// over a target it can use, else Inactive ((M5-V2) Reverse's over the ground: "AwaitingTarget[Reverse] over ground:
/// **Move**"; "over non-ground: **Inactive**"). (M5-N8)
/// A Repair's over a damaged own object that one of the selected `units` repairs ([`repaired_by`]): over one that none
/// of them repairs, an Agent's over a Storage, the gate would refuse the click. A Supply Chopper's Drop Off
/// Supplies' and Attach to Tower's over an own Supply Tower free for one of them ([`tower_free`]): over one another
/// chopper is attached to, the gate would refuse the click (`sc_sim::Reject::TowerTaken`).
fn awaiting_display(pending: PanelCommand, units: &[UnitCapabilities], target: &MapTarget) -> PointerDisplay {
    match (pending, target.target) {
        (PanelCommand::Repair, _) if !repaired_by(units, target) => PointerDisplay::Inactive,
        (PanelCommand::DropOffSupplies | PanelCommand::AttachToTower, _) if !tower_free(units, target) => {
            PointerDisplay::Inactive
        }
        _ => target_resolution(pending, target).map_or(PointerDisplay::Inactive, PointerDisplay::of),
    }
}
