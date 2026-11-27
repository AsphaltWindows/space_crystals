//! The Factory's panel (`factions/gdo/structures/factory.md > Placeholder stats`: "Production as the Barracks (queue
//! 20, refunds, rally). Panel: **Q** Raptor, **W** Twister, **E** Prospector, **R** Artillery Rig (greyed without
//! Advanced Mechanics), **X** Cancel Production, **C** Set Rally Point"; slice M6-GF): the Production template (plan
//! decision T23, `super::super::production_grid`), as the Barracks' ([`super::barracks`]); its right-click sets the
//! rally point. Q and W are its recipes, in its `Produces` order: the Raptor's, and the Twister's, greyed while the bank
//! holds less than its Supply (design change 0441: `sc_sim::Reject::InsufficientSupplies`, "Not enough Supplies"). E
//! and R hold the units it doesn't train yet, each not available ([`Planned`], "Not available yet") until its
//! slice: M6-GP's Prospector and M6-GA's Artillery Rig, whose recipes, behind Advanced Mechanics, take their slots.

use super::super::{InterfaceTemplate, PanelTemplate, Planned};

/// The Factory's panel template: the Production template, its two recipes in Q and W, its units not in the game yet in
/// E and R.
pub const TEMPLATE: PanelTemplate = PanelTemplate {
    planned: &[(2, Planned::Prospector), (3, Planned::ArtilleryRig)],
    ..PanelTemplate::of(InterfaceTemplate::Production)
};
