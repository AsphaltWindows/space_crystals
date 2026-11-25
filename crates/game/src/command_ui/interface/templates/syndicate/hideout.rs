//! The Hideout's panel (`factions/syndicate/structures/hideout.md > ObjectInterfaceState[Hideout]`, slice M4-J2): the
//! Production template (plan decision T23, `super::super::production_grid`), "Q: Build Agent", "W: Build Guard", "E:
//! Build Grenadier" (design change 0260), "X: Cancel Production" and "C: Set Rally Point"; its right-click sets the
//! rally point. The Hideout trains only the Agent yet: the Guard's W and the Grenadier's E hold their slots, greyed
//! with their prerequisites ([`Planned::Guard`], [`Planned::Grenadier`]), until slice M4-K gives the Hideout the Guard
//! and slice M4-U the Grenadier, whose recipes then take the slots.

use super::super::{InterfaceTemplate, PanelTemplate, Planned};

/// The Hideout's panel template: its Guard in W and its Grenadier in E, which it doesn't train yet (`> Produces`:
/// "Guard [...] Unit Prerequisite: player owns at least one Optics Lab", "Grenadier [...] Unit Prerequisite: player
/// owns at least one Chemical Plant").
pub const TEMPLATE: PanelTemplate = PanelTemplate {
    planned: &[(1, Planned::Guard), (2, Planned::Grenadier)],
    ..PanelTemplate::of(InterfaceTemplate::Production)
};
