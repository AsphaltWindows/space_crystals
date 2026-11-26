//! The Disciple's panel (`factions/cults/units/disciple.md > Placeholder stats`: "ObjectInterfaceState:
//! BasicCombatUnitInterfaceState, plus **R: Shroud**", slice M5-N4a): the BasicCombatUnit template, whose attack
//! commands it has no weapon for ("AttackAttributes - None") are greyed (Attack: `super::super::super::Conditions`,
//! as the Modes have it; Attack Ground: `super::super::can_target_ground`), and its spell in R, "hotkey **R** (R starts
//! the channel; Stop or another order cancels it; Q0112: R, as the other units that add an ability to the combat
//! panel; Q stays Move)", which held its slot from slice M5-N4a until slice M5-N4b brought it: the grid puts it in R
//! for a unit that casts Shroud, as it does the units' other abilities (`super::super::grid`,
//! `crate::command_ui::interface::PanelCommand::Shroud`). The Disciple is Ungroupable ("Groupable - false (user,
//! 2026-11-26; Q0112: each R press starts one Disciple's channel; Tab to the next)"), its type's data
//! (`sc_sim::defs::ObjectDef::groupable`), so that R goes to the one Disciple of the ActiveGroup.

use super::super::{InterfaceTemplate, PanelTemplate};

/// The Disciple's panel template: the BasicCombatUnit's, its R added by the grid.
pub const TEMPLATE: PanelTemplate = PanelTemplate::of(InterfaceTemplate::BasicCombatUnit);
