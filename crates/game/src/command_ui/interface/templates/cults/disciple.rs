//! The Disciple's panel (`factions/cults/units/disciple.md > Placeholder stats`: "ObjectInterfaceState:
//! BasicCombatUnitInterfaceState, plus **R: Shroud** and **F: Mirage**", slice M5-N4a): the BasicCombatUnit template,
//! whose attack commands it has no weapon for ("AttackAttributes - None") are greyed (Attack:
//! `super::super::super::Conditions`, as the Modes have it; Attack Ground: `super::super::can_target_ground`), and its
//! spells: in R, Shroud, "hotkey **R** (R starts the channel; Stop or another order cancels it; Q0112: R, as the other
//! units that add an ability to the combat panel; Q stays Move)", which held its slot from slice M5-N4a until slice
//! M5-N4b brought it, and (M6-CM) in F, Mirage (`> Mirage numbers`: "hotkey **F** (designer placeholders, 2026-11-27; W
//! is the panel's Reverse slot)"). The grid puts each in its slot for a unit that casts it, as it does the units' other
//! abilities (`super::super::grid`, `crate::command_ui::interface::PanelCommand::Shroud`,
//! `crate::command_ui::interface::PanelCommand::Mirage`). The Disciple is Ungroupable ("Groupable - false (user,
//! 2026-11-26; Q0112: each R press starts one Disciple's channel; Tab to the next)"), its type's data
//! (`sc_sim::defs::ObjectDef::groupable`), so that R and F go to the one Disciple of the ActiveGroup. An illusion
//! (`sc_sim::mirage`) is a Disciple and has its panel, R and F greyed ("**No spells**: illusions can't cast",
//! `sc_sim::Reject::IllusionsCantCast`).

use super::super::{InterfaceTemplate, PanelTemplate};

/// The Disciple's panel template: the BasicCombatUnit's, its R and F added by the grid.
pub const TEMPLATE: PanelTemplate = PanelTemplate::of(InterfaceTemplate::BasicCombatUnit);
/// `systems/combat.md > AttackType > CanTargetGround`, derived from the AttackType: the sim's rule, which its gate
/// takes an Attack Ground by (slice M4-O).
pub use sc_sim::combat::can_target_ground;
