//! The Extraction Facility's panel (`factions/gdo/economy.md > ExtractionFacility >
//! ObjectInterfaceState[ExtractionFacility]`): its DefaultState builds the Extraction Plate, then places it
//! (`super::super::state_grid`). No right-click.

use sc_sim::ObjectKind;

use super::super::{InterfaceTemplate, PanelCommand, PanelTemplate};

/// The Extraction Facility's panel template: its X while it holds the plate ready is "X: Cancel Ready Plate".
pub const TEMPLATE: PanelTemplate = PanelTemplate {
    names: &[(PanelCommand::CancelReady, "Cancel Ready Plate")],
    ..PanelTemplate::of(InterfaceTemplate::ExtractionFacility)
};

/// `> ObjectInterfaceState[ExtractionFacility] > DefaultState`: "Q: Build Extraction Plate", which becomes "Q: Place
/// Plate" once the plate is ready; its X, Cancel Construction or Cancel Ready Plate, shows only while it constructs
/// or holds the plate ready (`super::super::state_grid`).
#[rustfmt::skip]
pub const GRID: [Option<PanelCommand>; 12] = [
    Some(PanelCommand::Construct(ObjectKind::ExtractionPlate)), None, None, None,
    None,                                                       None, None, None,
    None,                                                       None, None, None,
];
