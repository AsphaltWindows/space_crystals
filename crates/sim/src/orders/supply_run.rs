//! A Supply Chopper's flight to a spot (`OrderKind::PickUpSupplies`, `OrderKind::DropOffSupplies`,
//! `OrderKind::AttachToTower`; `crate::supply_run`) — `factions/gdo/units/supply_chopper.md`.
//!
//! - **Its start** ([`destination`]): the chopper flies to the spot of the order's object, as its owner knows it: a
//!   station's or a tower's that it sees (`supply_run::spot_of`), a remembered station's centre. One on that object's
//!   spot already stays there (`supply_run::take_off`). The step right after this phase lands it there, or holds it
//!   off while another takes the spot (`Sim::step_supply_runs`); the economy phase picks up and drops off
//!   (`economy::supply_run`), which ends a Pick Up Supplies and a Drop Off Supplies.
//! - **Its end** ([`run`]): an Attach to Tower once the chopper is down on its tower (`> ObjectInterfaceState
//!   [SupplyChopper]`: "fly to tower, land, attach"); a Drop Off Supplies once it carries nothing; any of the three at
//!   once for a unit that is no chopper, and once its owner knows nothing of the order's object any more (a tower
//!   destroyed, `Known::Gone`).

use glam::Vec2;

use super::OrderKind;
use super::world::{Known, Unit, World};
use crate::landing::Spot;
use crate::supply_run;

/// Where the flight goes (module docs): the spot of the object the order lands the chopper on, as its owner knows it.
/// Nowhere for a chopper on that spot already, landing or down, which stays there, and for a unit that is no chopper.
pub(super) fn destination(kind: OrderKind, unit: &Unit, world: &World) -> Option<Vec2> {
    let at = supply_run::lands_for(kind)?;
    unit.chopper.as_deref()?;
    if unit.landing.is_some_and(|landing| landing.at == Spot::Structure(at)) {
        return None;
    }
    match world.known(unit.owner, at) {
        Known::Seen(view) => supply_run::spot_of(view.kind, view.pos),
        Known::Remembered { position } => Some(position),
        Known::Gone => None,
    }
}

/// One tick of the order (module docs): whether it is complete.
pub(super) fn run(kind: OrderKind, unit: &mut Unit, world: &World) -> bool {
    let (Some(at), Some(chopper)) = (supply_run::lands_for(kind), unit.chopper.as_deref()) else {
        unit.mover.stop();
        return true;
    };
    let down_on = (unit.landing).is_some_and(|landing| landing.at == Spot::Structure(at) && landing.descent.done());
    let empty = chopper.cargo.is_none();
    if matches!(world.known(unit.owner, at), Known::Gone) {
        if unit.landing.is_none() {
            unit.mover.stop();
        }
        return true;
    }
    match kind {
        OrderKind::AttachToTower(_) => down_on,
        OrderKind::DropOffSupplies(_) => empty,
        _ => false,
    }
}
