//! The Barracks' panel (`factions/gdo/structures/barracks.md > ObjectInterfaceState[Barracks]`): the Production
//! template, its units from its `ProductionDef` (plan decision T23, `super::super::production_grid`): "**Q: Build
//! Peacekeeper**", "**E: Build Field Engineer** (placeholder letter)" (slice M5-N9), greyed while its owner has no
//! Career Campus (the gate's `Reject::Requires`, "Requires a Career Campus"), X Cancel Production and C Set Rally
//! Point. "**W: Build Rocketman** (hotkey tentative)" stays hidden: the Rocketman isn't in the game yet.

use sc_sim::ObjectKind;

use super::super::{InterfaceTemplate, PanelTemplate};

/// The Barracks' panel template: the Field Engineer's Train in E, so that W stays the Rocketman's.
pub const TEMPLATE: PanelTemplate = PanelTemplate {
    trains: &[(2, ObjectKind::FieldEngineer)],
    ..PanelTemplate::of(InterfaceTemplate::Production)
};
