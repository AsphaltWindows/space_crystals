//! The Aerospace Assembly's panel (`factions/syndicate/structures/aerospace_assembly.md > Placeholder stats`: "Panel:
//! **Q** Phantom, **W** Specter, **E** Prism, **X** Cancel Production, **C** Set Rally Point"; slice M6-SD): the
//! Production template (plan decision T23, `super::super::production_grid`), as the Hideout's ([`super::hideout`]); its
//! right-click sets the rally point. Q is its recipe, the Phantom's, trained into the Tunnel Network behind an Optics
//! Lab (plan decision T96; "**Phantom** (requires an Optics Lab)"), which the gate greys it for; W and E hold the units
//! it doesn't train yet, each not available ([`Planned`]) until its slice: M6-SA's Specter and M6-SP's Prism, whose
//! recipes take their slots.

use super::super::{InterfaceTemplate, PanelTemplate, Planned};

/// The Aerospace Assembly's panel template: the Production template, its recipe in Q, its units not in the game yet in
/// W and E.
pub const TEMPLATE: PanelTemplate = PanelTemplate {
    planned: &[(1, Planned::Specter), (2, Planned::Prism)],
    ..PanelTemplate::of(InterfaceTemplate::Production)
};
