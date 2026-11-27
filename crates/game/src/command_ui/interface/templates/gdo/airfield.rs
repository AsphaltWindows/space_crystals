//! The Airfield's panel (slice M6-GH1; `factions/gdo/structures/airfield.md > Placeholder stats`: "Panel: **Q** Hornet,
//! **W** Dragonfly (greyed without Propulsion Labs), **X** Cancel Production, **C** Set Rally Point"): the Production
//! template (plan decision T23, `super::super::production_grid`), its right-click the rally point. Q is its recipe, the
//! Hornet's, for 150 Space Crystals and 2 Supplies: greyed while the bank holds less (`sc_sim::Reject::InsufficientSupplies`,
//! "Not enough Supplies"; the GDO has no Supply until slice M6-GS) and while the Airfield is full, its assignments and
//! the aircraft in its queue at 4 (`factions/gdo/structures/airfield.md > Production limit`; `sc_sim::Reject::AirfieldFull`,
//! "Airfield full"). W holds the Dragonfly's slot, not available ([`Planned::Dragonfly`]) until slice M6-GL trains it.

use super::super::{InterfaceTemplate, PanelTemplate, Planned};

/// The Airfield's panel template: the Production template, its recipe in Q, the Dragonfly not in the game yet in W.
pub const TEMPLATE: PanelTemplate = PanelTemplate {
    planned: &[(1, Planned::Dragonfly)],
    ..PanelTemplate::of(InterfaceTemplate::Production)
};
