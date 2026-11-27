//! (M6-GA) The vehicle deploy order's behaviour (`OrderKind::VehicleDeploy`, plan decision T44: "the behaviour in
//! `orders/vehicle_deploy.rs`, holding still with no auto-targeting"), `factions/gdo/units/artillery_rig.md > Modes`:
//! "switching modes takes time [...]; the unit can neither move nor attack while switching". The model is
//! `crate::vehicle_deploy`'s.
//!
//! - **It stops**: the order's start stops the unit, as every order's but a Move to a point does (`super::run::start`),
//!   and disengages what it attacked. Its state counts the frames of its type's deploy ([`begin`]), which the step
//!   right before phase 5a takes one on each tick and completes (`Sim::step_vehicle_deploys`): behaviours change no
//!   kind, so the order never completes by itself ([`run`]).
//! - **It stands, and attacks nothing**, for every tick of its deploy: nothing gives it a goal, and it can't attack now
//!   (`super::world::Unit::may_attack`), so its turret drops what it locked and picks nothing (`super::turret`), and
//!   phase 5e starts no attack sequence for it. With an order it has no idle auto-targeting, and never yields.

use super::state::Behaviour;
use super::world::Unit;

/// A deploy begins (module docs): none of its type's frames done yet (`crate::vehicle_deploy::begin`). The unit can't
/// attack from its first tick on.
pub(super) fn begin(unit: &mut Unit) -> Behaviour {
    unit.may_attack = false;
    Behaviour::VehicleDeploy(crate::vehicle_deploy::begin(unit.kind))
}

/// One tick of a deploy (module docs): the unit stands, unable to attack; never complete, since its step completes it.
pub(super) fn run(unit: &mut Unit) -> bool {
    unit.may_attack = false;
    false
}
