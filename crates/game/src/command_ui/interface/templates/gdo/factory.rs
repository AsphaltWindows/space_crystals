//! The Factory's panel (`factions/gdo/structures/factory.md > Placeholder stats`: "Production as the Barracks (queue
//! 20, refunds, rally). Panel: **Q** Raptor, **W** Twister, **E** Prospector, **R** Artillery Rig (greyed without
//! Advanced Mechanics), **X** Cancel Production, **C** Set Rally Point"; slice M6-GF): the Production template (plan
//! decision T23, `super::super::production_grid`), as the Barracks' ([`super::barracks`]); its right-click sets the
//! rally point. Q and W are its first recipes, in its `Produces` order: the Raptor's, and the Twister's, greyed while
//! the bank holds less than its Supply (design change 0441: `sc_sim::Reject::InsufficientSupplies`, "Not enough
//! Supplies"). (M6-GA) R is the Artillery Rig's, greyed while its owner has no Advanced Mechanics (the gate's
//! `sc_sim::Reject::Requires`, "Requires an Advanced Mechanics") and while the bank holds less than its 5 Supplies. E
//! holds the Prospector, which it doesn't train yet, not available ([`Planned`], "Not available yet") until slice
//! M6-GP, whose recipe, behind Advanced Mechanics, takes its slot.

use sc_sim::ObjectKind;

use super::super::{InterfaceTemplate, PanelTemplate, Planned};

/// The Factory's panel template: the Production template, the Artillery Rig's Train in R, the Raptor and the Twister
/// taking Q and W, the first slots left, and the Prospector, not in the game yet, in E.
pub const TEMPLATE: PanelTemplate = PanelTemplate {
    trains: &[(3, ObjectKind::ArtilleryRig)],
    planned: &[(2, Planned::Prospector)],
    ..PanelTemplate::of(InterfaceTemplate::Production)
};
