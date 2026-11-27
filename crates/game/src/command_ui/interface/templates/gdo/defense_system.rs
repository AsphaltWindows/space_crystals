//! The Defense System's panel (`factions/gdo/structures/defense_system.md > Placeholder stats`, design change 0392,
//! slice M6-GR): "**Works like the Extraction Facility** [...] one slot, Q, no build menu": its DefaultState builds a
//! Missile Turret, then places it (`super::super::state_grid`), as the Extraction Facility's its plate
//! (`super::extraction_facility`). No right-click.
//!
//! It is Groupable ("Groupable true"): the Defense Systems selected are one ActiveGroup, whose panel follows the first
//! one's construction (`super::super::Conditions::constructor`). Q's Build and X's Cancels go to every one of them, as
//! a Groupable type's commands do (`systems/controls/selection.md > SelectionGroup > No smart-casting`), each that can
//! taking it; Q's Place places the first one's turret, the one the panel shows (**Placeholder (not in the design: a
//! group of Defense Systems' placement goes to the first one, as a group of Drones' Deploy to one Drone,
//! `systems/controls/selection.md > Groupable workers used up by a build`)**).

use sc_sim::ObjectKind;

use super::super::{InterfaceTemplate, PanelCommand, PanelTemplate};

/// The Defense System's panel template: its X while it holds a turret ready is "X: Cancel Ready Turret".
pub const TEMPLATE: PanelTemplate = PanelTemplate {
    names: &[(PanelCommand::CancelReady, "Cancel Ready Turret")],
    ..PanelTemplate::of(InterfaceTemplate::DefenseSystem)
};

/// `factions/gdo/structures/defense_system.md > Placeholder stats`: "**Idle**: **Q: Build Missile Turret** (100 Space
/// Crystals, starts construction)", which becomes "**Q: Place Missile Turret**, greyed out, with a **running progress
/// bar** on the button" while it constructs, and enters AwaitingPlacement once the turret is ready; its X, "**X: Cancel
/// Construction** (full refund)" or "**X: Cancel Ready Turret** (75% refund, rounded down)", shows only while it
/// constructs or holds the turret ready (`super::super::state_grid`).
#[rustfmt::skip]
pub const GRID: [Option<PanelCommand>; 12] = [
    Some(PanelCommand::Construct(ObjectKind::MissileTurret)), None, None, None,
    None,                                                     None, None, None,
    None,                                                     None, None, None,
];
