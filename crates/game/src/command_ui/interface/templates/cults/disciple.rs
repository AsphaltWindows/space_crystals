//! The Disciple's panel (`factions/cults/units/disciple.md > Placeholder stats`: "ObjectInterfaceState:
//! BasicCombatUnitInterfaceState, plus **R: Shroud**", slice M5-N4a): the BasicCombatUnit template, whose attack
//! commands it has no weapon for ("AttackAttributes - None") are greyed (Attack: `super::super::super::Conditions`,
//! as the Modes have it; Attack Ground: `super::super::can_target_ground`), and its spell in R, "hotkey **R** (R starts
//! the channel; Stop or another order cancels it; Q0112: R, as the other units that add an ability to the combat
//! panel; Q stays Move)", holding its slot until slice M5-N4b brings it ([`Planned::Shroud`]). The Disciple is
//! Ungroupable ("Groupable - false (user, 2026-11-26; Q0112: each R press starts one Disciple's channel; Tab to the
//! next)"), its type's data (`sc_sim::defs::ObjectDef::groupable`).

use super::super::{InterfaceTemplate, PanelTemplate, Planned};

/// R's slot (`CommandSlot::all` order), the first the BasicCombatUnit grid leaves free, where the other units add
/// their own (the Peacekeeper's Riot Shield, the Legionnaires' Convert: `super::super::grid`).
const SHROUD_SLOT: usize = 3;

/// The Disciple's panel template: the BasicCombatUnit's, with Shroud in R.
pub const TEMPLATE: PanelTemplate = PanelTemplate {
    planned: &[(SHROUD_SLOT, Planned::Shroud)],
    ..PanelTemplate::of(InterfaceTemplate::BasicCombatUnit)
};
