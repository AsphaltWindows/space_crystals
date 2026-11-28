//! (M6-GH1) An aircraft's flight to its Airfield (`OrderKind::Rebase`, `OrderKind::ReturnToBase`; plan decision T88,
//! `crate::aircraft`): `systems/mechanics/aircraft_landing.md > Assigned base`: "the aircraft is reassigned and flies
//! there"; "**Return to Base** [...]: fly there, land and rearm", which an aircraft out of ammo also gets by itself
//! ("**Automatic return**").
//!
//! - **Its start**: the unit flies to the centre of its base ([`destination`]): for a Rebase, the Airfield that
//!   `Sim::give_order` assigned it to as the order was given. (M6-GH2) The landing step right after this phase sends
//!   it on to its landing slot, and lands it there (`Sim::step_landings`, `crate::landing`). One on a slot of its base
//!   already stays there: the order lifts it off no more than it flies it anywhere (`crate::landing::take_off`).
//! - **Its end** ([`run`]): (M6-GH2) once it is down on its slot, as its landing's frames are all done
//!   (`systems/mechanics/aircraft_landing.md > Assigned base`: "**Shift-queued orders run as soon as it has
//!   landed**", "taking off with the ammo rearmed so far"): an order queued behind the flight starts in the next tick
//!   and lifts it off, its rearm cut short, as an order given during rearming does (`crate::landing::take_off`). With
//!   none, it stays landed, idle, and rearms there (`crate::landing::rearm`, whatever its order): "**After rearming it
//!   stays landed**". It replaced slice M6-GH1's hover over the Airfield's centre (D34), whose flight ended there once
//!   its move was no longer under way.
//! - **No base**: its Airfield destroyed on the way, which unassigns it (`Sim::remove_objects`; "An unassigned aircraft
//!   has no base: it doesn't return automatically"), or a unit that got the order unassigned (a hand-edited save's:
//!   the gate refuses it the order). The order ends where the unit is, at once.

use glam::Vec2;

use super::world::{Unit, World};

/// Where the unit's flight goes: the centre of its base, if it has one (`Unit::base`) and the Airfield still stands.
/// (M6-GH2) Nowhere for an aircraft on a slot of its base already, which stays there (module docs).
pub(super) fn destination(unit: &Unit, world: &World) -> Option<Vec2> {
    if unit.on_slot() {
        return None;
    }
    let base = world.get(unit.base()?)?;
    Some(base.pos)
}

/// One tick of the flight (module docs): whether it is complete. (M6-GH2) Once the aircraft is down on its slot,
/// rearmed or not; the landing step lands it, and the economy phase rearms it.
pub(super) fn run(unit: &mut Unit) -> bool {
    if unit.base().is_none() {
        unit.mover.stop();
        return true;
    }
    unit.landing().is_some_and(|landing| landing.descent.done())
}
