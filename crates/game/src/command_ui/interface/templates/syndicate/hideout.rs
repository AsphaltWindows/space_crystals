//! The Hideout's panel (`factions/syndicate/structures/hideout.md > ObjectInterfaceState[Hideout]`, slice M4-J2): the
//! Production template (plan decision T23, `super::super::production_grid`), "Q: Build Agent", "W: Build Guard", "E:
//! Build Grenadier" (design change 0260), "X: Cancel Production" and "C: Set Rally Point"; its right-click sets the
//! rally point. Q, W and E are its recipes, in their order, each greyed while the gate refuses it, with the gate's
//! reason: the Guard's while the player owns no Optics Lab (slice M4-K: "Only available if the player owns at least
//! one Optics Lab", `sc_sim::Reject::Requires`), the Grenadier's while they own no Chemical Plant (slice M4-U: "Only
//! available if the player owns at least one Chemical Plant"), whose recipe took the slot its placeholder held.

use super::super::{InterfaceTemplate, PanelTemplate};

/// The Hideout's panel template: the Production template, its recipes in Q, W and E (`> Produces`).
pub const TEMPLATE: PanelTemplate = PanelTemplate::of(InterfaceTemplate::Production);
