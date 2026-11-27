//! The Drone Operator's panel (slice M6-GD; `factions/gdo/units/drone_operator.md > Placeholder stats`: "**Build
//! Drone** (hotkey **F**)", "**Launch Drone** (hotkey **R**)"; plan §GD: "the operator's panel: R Launch, and F Build
//! Drone with a progress bar"): the BasicCombatUnit template without its attack commands, the Prayer-mobile's (the
//! operator "has **no weapon** of its own"), its A the selection's ([`super::super::attack_slot`], design change 0399),
//! and its stock's two commands, which the grid adds for a unit whose stock holds drones (`sc_sim::defs::UnitDef::stock`
//! whose item is on a leash, `sc_sim::defs::UnitDef::tether`; [`super::super::grid`]), as it adds the units' other
//! abilities:
//! - **R, Launch Drone** ([`PanelCommand::Launch`]): the drone comes out over the operator, which then "**cannot
//!   move**": its Move and Patrol are greyed while every one of their recipients holds still for its drone
//!   (`sc_sim::Reject::DroneOut`, `crate::command_ui::interface::Conditions::unavailable`), and a right-click that
//!   moves none of them says so (`crate::command_ui::feedback`). R is greyed while its drone is out or none is in
//!   stock.
//! - **F, Build Drone** ([`PanelCommand::Buy`]): greyed while the stock is full, while a drone is out ("Build Drone can
//!   start only once the operator's drone has **ended**", design change 0421) and while one is bought, its bar showing
//!   the purchase's progress then (`crate::command_ui::interface::PanelSlot::progress`), its hint its price.
//!
//! The operator is Groupable ("Groupable true"): R and F go to every operator of the ActiveGroup. Its right-click is
//! the sim's one rule, as every unit's (`crate::command_ui::resolution::unit_right_click`).

use super::super::{InterfaceTemplate, PanelTemplate};

/// R: Launch Drone.
pub const LAUNCH_SLOT: usize = 3;
/// F: Build Drone.
pub const BUY_SLOT: usize = 7;

/// The Drone Operator's panel template: the BasicCombatUnit's without its attack commands, R and F added by the grid.
pub const TEMPLATE: PanelTemplate = PanelTemplate {
    without_attacks: true,
    ..PanelTemplate::of(InterfaceTemplate::BasicCombatUnit)
};
