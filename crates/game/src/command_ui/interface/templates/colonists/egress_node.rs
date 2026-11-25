//! The Egress Node's panel (`factions/colonists/economy.md > Egress Node > Egress Node ObjectInterfaceState`, slice
//! M3-J2): "F: Select Returning Drones" (design change 0124: "Egress Node: F"), its one command. No right-click.

use super::super::{InterfaceTemplate, PanelCommand, PanelTemplate};

/// The Egress Node's panel template.
pub const TEMPLATE: PanelTemplate = PanelTemplate::of(InterfaceTemplate::EgressNode);

/// Its DefaultState: F, Select Returning Drones.
#[rustfmt::skip]
pub const GRID: [Option<PanelCommand>; 12] = [
    None, None, None, None,
    None, None, None, Some(PanelCommand::SelectReturningDrones),
    None, None, None, None,
];
