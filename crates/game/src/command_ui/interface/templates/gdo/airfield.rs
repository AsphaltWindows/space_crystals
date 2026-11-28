//! The Airfield's panel (slice M6-GH1; `factions/gdo/structures/airfield.md > Placeholder stats`: "Panel: **Q** Hornet,
//! **W** Dragonfly (greyed without Propulsion Labs), **X** Cancel Production, **C** Set Rally Point"): the Production
//! template (plan decision T23, `super::super::production_grid`), its right-click the rally point. Q is its recipe, the
//! Hornet's, for 150 Space Crystals and 2 Supplies: greyed while the bank holds less (`sc_sim::Reject::InsufficientSupplies`,
//! "Not enough Supplies"; the GDO starts with 1) and while the Airfield is full, its assignments and
//! the aircraft in its queue at 4 (`factions/gdo/structures/airfield.md > Production limit`; `sc_sim::Reject::AirfieldFull`,
//! "Airfield full"). (M6-GL) W is its second recipe, the Dragonfly's, for 175 Space Crystals and 4 Supplies, greyed so
//! too, and while the player owns no Propulsion Labs, its Unit Prerequisite (`sc_sim::Reject::Requires`, "Requires a
//! Propulsion Labs", as `crate::command_ui::feedback::requires_text` words every one). It held the Dragonfly's slot,
//! not available, from slice M6-GH1 until slice M6-GL trained it.

use super::super::{InterfaceTemplate, PanelTemplate};

/// The Airfield's panel template: the Production template, its recipes in Q and W.
pub const TEMPLATE: PanelTemplate = PanelTemplate::of(InterfaceTemplate::Production);
