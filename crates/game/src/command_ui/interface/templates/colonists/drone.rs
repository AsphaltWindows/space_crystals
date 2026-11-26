//! The Drone's panel (`factions/colonists/units/drone.md > Drone ObjectInterfaceState`): Q Move, W Deploy, S Set
//! Drop-off, X Stop ("**W: Deploy** (moved from A, 2026-11-27 [...])", design change 0398). It is unarmed, so its A is
//! empty: the panel shows Attack there only while another selected unit can attack (`systems/controls/interface.md >
//! CommandPanel > Standard Slot Assignments`, design change 0399; `super::super::attack_slot`). Its right-click
//! (`factions/colonists/units/drone.md > Drone ObjectInterfaceState > Right-Click Resolution`: Gather, Drop Off, Move,
//! Follow) is the sim's one rule, as every unit's (`crate::command_ui::resolution::unit_right_click`, slice M3-R).

use super::super::{InterfaceTemplate, PanelCommand, PanelTemplate};

/// The Drone's panel template.
pub const TEMPLATE: PanelTemplate = PanelTemplate::of(InterfaceTemplate::Drone);

/// `factions/colonists/units/drone.md > Drone ObjectInterfaceState > DefaultState commands` ("No Hold Position,
/// Patrol or Attack"): Q Move and X Stop, "as in BasicCombatUnitInterfaceState", W Deploy, and S Set Drop-off, which
/// awaits its drop-off
/// (AwaitingTarget\[SetDropOffLocation\]; design change 0398: "Drone | Q Move, **W Deploy**, S Set Drop-off, X Stop").
/// The section's grid still draws "[A] Deploy" and its Unit Commands say "Deploy — via panel button A": both older than
/// the "**W: Deploy** (moved from A, 2026-11-27 [...])" beside them, which this follows.
#[rustfmt::skip]
pub const GRID: [Option<PanelCommand>; 12] = {
    use PanelCommand::{Deploy, Move, SetDropOff, Stop};
    [
        Some(Move), Some(Deploy),     None, None,
        None,       Some(SetDropOff), None, None,
        None,       Some(Stop),       None, None,
    ]
};
