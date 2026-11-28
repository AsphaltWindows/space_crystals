//! The Barracks' panel (`factions/gdo/structures/barracks.md > ObjectInterfaceState[Barracks]`): the Production
//! template, its units from its `ProductionDef` (plan decision T23, `super::super::production_grid`): "**Q: Build
//! Peacekeeper**", "**W: Build Rocketman** (hotkey tentative)" (slice M5-K), greyed while its owner has no Arsenal (the
//! gate's `Reject::Requires`, "Requires an Arsenal"), "**E: Build Field Engineer** (placeholder letter)" (slice M5-N9),
//! greyed while its owner has no Career Campus ("Requires a Career Campus") and while the bank holds less than
//! its Supply, 1 (`factions/gdo/units/field_engineer.md > Placeholder stats`), (M6-GD) R: Build Drone Operator
//! (`factions/gdo/units/drone_operator.md > Placeholder stats`: "trained at the Barracks on **R** (Q0155)"), greyed
//! while its owner has no Career Campus, and (design change 0441: "greyed without it, with a feedback line that names
//! Supplies") while the bank holds less than its Supply (`sc_sim::Reject::InsufficientSupplies`, "Not enough
//! Supplies"), X Cancel Production and C Set Rally Point.

use sc_sim::ObjectKind;

use super::super::{InterfaceTemplate, PanelTemplate};

/// The Barracks' panel template: the Rocketman's Train in W, the Field Engineer's in E and the Drone Operator's in R, the
/// Peacekeeper taking Q, the first slot left.
pub const TEMPLATE: PanelTemplate = PanelTemplate {
    trains: &[
        (1, ObjectKind::Rocketman),
        (2, ObjectKind::FieldEngineer),
        (3, ObjectKind::DroneOperator),
    ],
    ..PanelTemplate::of(InterfaceTemplate::Production)
};
