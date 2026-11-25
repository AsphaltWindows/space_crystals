//! The Hideout's panel (`factions/syndicate/structures/hideout.md > ObjectInterfaceState[Hideout]`, slice M4-J2): the
//! Production template (plan decision T23, `super::super::production_grid`), "Q: Build Agent", "W: Build Guard", "E:
//! Build Grenadier" (design change 0260), "X: Cancel Production" and "C: Set Rally Point"; its right-click sets the
//! rally point. Q and W are its recipes, the Guard's greyed while the player owns no Optics Lab (slice M4-K: "Only
//! available if the player owns at least one Optics Lab", the gate's `sc_sim::Reject::Requires`). The Grenadier's E
//! holds its slot, greyed with its prerequisite ([`Planned::Grenadier`]), until slice M4-U brings the Grenadier, whose
//! recipe then takes the slot.

use super::super::{InterfaceTemplate, PanelTemplate, Planned};

/// The Hideout's panel template: its Grenadier in E, which it doesn't train yet (`> Produces`: "Grenadier [...] Unit
/// Prerequisite: player owns at least one Chemical Plant").
pub const TEMPLATE: PanelTemplate = PanelTemplate {
    planned: &[(2, Planned::Grenadier)],
    ..PanelTemplate::of(InterfaceTemplate::Production)
};
