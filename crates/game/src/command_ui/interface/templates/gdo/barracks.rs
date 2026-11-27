//! The Barracks' panel (`factions/gdo/structures/barracks.md > ObjectInterfaceState[Barracks]`): the Production
//! template, its units from its `ProductionDef` (plan decision T23, `super::super::production_grid`): "**Q: Build
//! Peacekeeper**", "**W: Build Rocketman** (hotkey tentative)" (slice M5-K), greyed while its owner has no Arsenal (the
//! gate's `Reject::Requires`, "Requires an Arsenal"), "**E: Build Field Engineer** (placeholder letter)" (slice
//! M5-N9), greyed while its owner has no Career Campus ("Requires a Career Campus"), X Cancel Production and C Set
//! Rally Point.

use sc_sim::ObjectKind;

use super::super::{InterfaceTemplate, PanelTemplate};

/// The Barracks' panel template: the Rocketman's Train in W and the Field Engineer's in E, the Peacekeeper taking Q,
/// the first slot left.
pub const TEMPLATE: PanelTemplate = PanelTemplate {
    trains: &[(1, ObjectKind::Rocketman), (2, ObjectKind::FieldEngineer)],
    ..PanelTemplate::of(InterfaceTemplate::Production)
};
