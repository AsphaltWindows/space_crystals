//! (M6-GH1) An aircraft's flight to its Airfield (`OrderKind::Rebase`, `OrderKind::ReturnToBase`; plan decision T88,
//! `crate::aircraft`): `factions/gdo/structures/airfield.md > Assignment`: "The aircraft is reassigned and flies there
//! to land"; `> Returning and rearming`: "**Return to Base** command: sends the aircraft to its assigned Airfield to
//! land and rearm", which an aircraft out of ammo also gets by itself ("**Automatic return**").
//!
//! - **Its start**: the unit flies to the centre of its base ([`destination`]): for a Rebase, the Airfield that
//!   `Sim::give_order` assigned it to as the order was given.
//! - **Its end** ([`run`]): over the Airfield, once its move is no longer under way, arrived or stuck, as a Move's.
//!   **Placeholder (D34 for slice M6-GH1)**: it hovers there, idle; slice M6-GH2 lands it on a pad and rearms it.
//! - **No base**: its Airfield destroyed on the way, which unassigns it (`Sim::remove_objects`; "An unassigned aircraft
//!   has no base: it doesn't return automatically"), or a unit that got the order unassigned (a hand-edited save's:
//!   the gate refuses it the order). The order ends where the unit is, at once.

use glam::Vec2;

use super::world::{Unit, World};
use crate::mover::MoveStatus;

/// Where the unit's flight goes: the centre of its base, if it has one (`Unit::base`) and the Airfield still stands.
pub(super) fn destination(unit: &Unit, world: &World) -> Option<Vec2> {
    let base = world.get(unit.base?)?;
    Some(base.pos)
}

/// One tick of the flight (module docs): whether it is complete.
pub(super) fn run(unit: &mut Unit) -> bool {
    if unit.base.is_none() {
        unit.mover.stop();
        return true;
    }
    unit.mover.status != MoveStatus::Moving
}
