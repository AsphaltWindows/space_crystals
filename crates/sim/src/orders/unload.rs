//! (M6-ST1) The Unload All order's behaviour (`OrderKind::UnloadAll`, plan decision T94),
//! `factions/syndicate/units/subterranean_transport.md > Placeholder stats`: "**Unloading**: **F: Unload All** (above
//! ground only; they come out around it)" (`crate::carry`).
//!
//! - **It stops**: the order's start stops the carrier, as every order's but a Move to a point does
//!   (`super::run::start`).
//! - **The unload** is a step of the economy phase (`Sim::step_unloads`, beside the buildings' that let units out),
//!   since behaviours bring no object onto the map: it waits for the carrier to come to rest (`Mover::momentum`: a
//!   tracked vehicle brakes off its speed over a few ticks), then puts its passengers on the map round it and ends the
//!   order; its queued orders go on from the next tick, as after any order that completed
//!   (`systems/controls/commands.md > Unit Command`). So the behaviour itself never completes ([`run`]).
//!   `systems/mechanics/carrying.md > Unloading`: "**On the move, it stops first**: the passengers come out once the
//!   carrier is at rest".
//! - **A queued Unload All** that starts with no one aboard, or with the carrier no longer one that loads, ends with no
//!   unload: the gate judged it as it was given.

use super::OrderKind;
use crate::object::Object;

/// One tick of an Unload All (module docs): never complete, the economy phase's step ending it.
pub(super) fn run() -> bool {
    false
}

/// Whether `o`'s current order is an Unload All, for the economy phase's step (module docs).
pub(crate) fn unloading(o: &Object) -> bool {
    (o.orders.as_ref())
        .and_then(|orders| orders.current.as_ref())
        .is_some_and(|active| active.order.kind == OrderKind::UnloadAll)
}
