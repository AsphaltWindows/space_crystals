//! The Artillery Rig's two panels (`factions/gdo/units/artillery_rig.md > Placeholder stats`: "Panel:
//! BasicCombatUnitInterfaceState plus **R: Deploy** (mobile) / **R: Undeploy** (deployed); the mobile Rig has no attack
//! commands"; slice M6-GA): the BasicCombatUnit template, without its attack commands for the mobile Rig
//! ([`MOBILE`], the Prayer-mobile's and the Drone Operator's: its A the selection's, `super::super::attack_slot`), with
//! them for the deployed one ([`DEPLOYED`], its Attack Ground lit by its CanTargetGround), and R, which the grid adds for
//! a type that deploys (`sc_sim::defs::UnitDef::vehicle_deploy`; `super::super::grid`), as it adds the units' other
//! abilities:
//! - **R, Deploy** on the mobile Rig and **Undeploy** on the deployed one (`super::super::PanelCommand::VehicleDeploy`,
//!   the type it becomes): a vehicle deploy order (`sc_sim::OrderKind::VehicleDeploy`), at once and where each unit
//!   stands, to every unit of the ActiveGroup (the Rig is Groupable, "Groupable true"), Shift queuing it ("**Queueable**:
//!   Shift-Deploy waits until the earlier orders are done"). It is greyed while the deploy runs ("**Undeploy is only
//!   available once deployment is complete**", `sc_sim::Reject::Deploying`), its bar showing the deploy's progress
//!   (`crate::command_ui::interface::PanelSlot::progress`), and the InfoPanel says it ("Deploying: 50%").
//! - **While it deploys** the Rig takes the orders its new type will carry out ("the **last order given** [...] is
//!   **held**"): the panel and the right-click judge it as the type it becomes (`crate::command_ui::interface::Modes::
//!   becomes`), so that the mobile Rig's panel shows A: Attack while it deploys, and a right-click on an enemy holds an
//!   Attack. An order the new type can't carry out is dropped as the deploy ends, "with the usual unavailable-command
//!   feedback" (`sc_sim::SimEvent::OrderDropped`, said as the gate's refusal of it: `crate::command_ui::feedback`).
//! - **Deployed**, the Rig "**cannot move**": its Move, Reverse and Patrol are greyed while every one of their
//!   recipients is of a type that can't move (`sc_sim::Reject::CannotMove`, `sc_sim::vehicle_deploy::immobile`), and a
//!   right-click on the ground that moves none of them says so.
//!
//! The switch of type moves each Rig to its new type's SelectionGroup ("**Two unit types** [...], with separate
//! SelectionGroups"), as a Riot Shield's toggle does (`SimEvent::KindChanged`, `crate::selection`): "When the switch
//! makes a group disappear, the group the units joined becomes active".

use super::super::{InterfaceTemplate, PanelTemplate};

/// R: Deploy, or Undeploy (`super::super::PanelCommand::VehicleDeploy`): the first slot the BasicCombatUnit grid leaves
/// free, where the other units have their own command (the Riot Shield's, a Convert, the Disciple's Shroud).
pub const DEPLOY_SLOT: usize = 3;

/// The mobile Rig's panel template: the BasicCombatUnit's without its attack commands ("the mobile Rig has no attack
/// commands"), R added by the grid.
pub const MOBILE: PanelTemplate = PanelTemplate {
    without_attacks: true,
    ..PanelTemplate::of(InterfaceTemplate::BasicCombatUnit)
};

/// The deployed Rig's panel template: the BasicCombatUnit's, R added by the grid.
pub const DEPLOYED: PanelTemplate = PanelTemplate::of(InterfaceTemplate::BasicCombatUnit);
