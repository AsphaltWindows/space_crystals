//! The Drone's panel (`factions/colonists/units/drone.md > Drone ObjectInterfaceState`): Q Move, A Deploy, S Set
//! Drop-off, X Stop. Its right-click (`> Right-Click Resolution`: Gather, Drop Off, Move, Follow) is the sim's one
//! rule, as every unit's (`crate::command_ui::resolution::unit_right_click`, slice M3-R).

use super::super::{InterfaceTemplate, PanelCommand, PanelTemplate};

/// The Drone's panel template.
pub const TEMPLATE: PanelTemplate = PanelTemplate::of(InterfaceTemplate::Drone);

/// `> Drone ObjectInterfaceState > DefaultState commands` ("No Hold Position, Patrol or Attack"): Q Move and X Stop,
/// "as in BasicCombatUnitInterfaceState", A Deploy, and S Set Drop-off, which awaits its drop-off
/// (AwaitingTarget\[SetDropOffLocation\]; design change 0124: "Drone: Q Move, A Deploy, S Set Drop-off, X Stop").
#[rustfmt::skip]
pub const GRID: [Option<PanelCommand>; 12] = {
    use PanelCommand::{Deploy, Move, SetDropOff, Stop};
    [
        Some(Move),   None,             None, None,
        Some(Deploy), Some(SetDropOff), None, None,
        None,         Some(Stop),       None, None,
    ]
};
