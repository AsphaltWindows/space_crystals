//! (M6-GD) The Launch order's behaviour (`OrderKind::Launch`, plan decision T91), `factions/gdo/units/drone_operator.md
//! > Launching and piloting`: "The Drone Operator launches a drone from its stock".
//!
//! - **It stops**: the order's start stops the unit, as every order's but a Move to a point does (`super::run::start`).
//! - **The launch** is the step right after phase 5a's (`Sim::step_launches`, `crate::tether`), since behaviours bring
//!   no object into the game: the order waits for it in its first tick ([`run`]). The step launches the unit's drone,
//!   if it has one in stock and none out, and ends the order either way; its queued orders go on from the next tick,
//!   as after any order that completed (`systems/controls/commands.md > Unit Command`).
//! - **A queued Launch** that starts with nothing to launch, its drone out or its stock empty by then, ends with no
//!   launch: the gate judged it as it was given (`Reject::DroneOut`, `Reject::StockEmpty`), as a channel's energy.

use super::OrderKind;
use crate::object::Object;

/// One tick of a Launch (module docs), `first` in the tick it started: whether it is complete. In its first tick it
/// waits for the step after the phase, which ends it; a Launch still current in a later tick (a hand-edited save's)
/// completes then.
pub(super) fn run(first: bool) -> bool {
    !first
}

/// Whether `o`'s current order is a Launch, for the step right after phase 5a (module docs): one 5a started in this
/// tick, since that step ends every one.
pub(crate) fn launching(o: &Object) -> bool {
    (o.orders.as_ref())
        .and_then(|orders| orders.current.as_ref())
        .is_some_and(|active| active.order.kind == OrderKind::Launch)
}
