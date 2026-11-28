//! (M6-GH1) Aircraft that rearm at an Airfield — plan decision T88, `factions/gdo/structures/airfield.md > Aircraft
//! Support`: "GDO aircraft carry limited ammo and **rearm at an Airfield**". A unit whose type has ammo
//! (`UnitDef::ammo`, the Hornet) holds an [`AircraftState`]: the rounds it has left, and the Airfield it is assigned to,
//! its base.
//!
//! - **Ammo**: full as it comes into the game ([`fresh`], `Sim::new_object`). Each shot spends a round as it is fired
//!   or launched (`combat::phases`, phase 5e), and with none left the unit can't attack ([`armed`]), whatever weapon
//!   it holds: its behaviours give up their attacks and it acquires no target (phase 5a's and 5e's `may_attack`, as
//!   behind a raised Riot Shield), it is no threat to the others' scans (`combat::threat`), its right-click attacks
//!   nothing (`orders::resolution::Abilities::may_attack`), and the gate refuses it an Attack or an Attack Ground
//!   (`Reject::OutOfAmmo`). A sequence under way runs on to its end, firing nothing more. It "stays out of ammo until
//!   it returns" (`systems/mechanics/aircraft_landing.md > Assigned base`): slice M6-GH2 rearms it.
//! - **Assignment** (`systems/mechanics/aircraft_landing.md > Assigned base`): "An aircraft is assigned to the Airfield
//!   that built it" (`Sim::release_unit`). "An Airfield holds at most **4 assignments**" (`AirfieldDef::capacity`), the
//!   aircraft in its production queue counting: its counts are derived from the game, never stored ([`taken`],
//!   [`room`]). The production gate refuses an aircraft that would pass them (`Reject::AirfieldFull`: "An Airfield
//!   **can't queue another aircraft** if its assignments plus the aircraft already in its production queue would exceed
//!   4"), and so does a reassignment, a Rebase (`OrderKind::Rebase`), applied as it is given: the aircraft then flies
//!   to its new base (`orders::air_base`). "**No manual unassigning**". When an Airfield goes, the removal path
//!   unassigns its aircraft ("its aircraft become **unassigned** [...] **There is no automatic reassignment**";
//!   `Sim::remove_objects`).
//! - **The return** (`systems/mechanics/aircraft_landing.md > Assigned base`): "when an aircraft runs out of ammo, it
//!   automatically flies back to its assigned Airfield": its last round gone, it gets a Return to Base
//!   (`OrderKind::ReturnToBase`) once phase 5e's dead are removed, which replaces its orders and which any order
//!   replaces in turn ("The automatic return **can be interrupted** by giving the aircraft any other order";
//!   `Sim::send_home`), the orders queued behind the one it ran out in dropped. "An unassigned
//!   aircraft has no base: it doesn't return automatically, and Return to Base is unavailable until it is reassigned."
//! - **(M6-GH2) The landing**: its flight home ends as it is down on a landing slot of its base, where it rearms and
//!   stays ([`AircraftState::landing`], `crate::landing`) unless an order queued behind the flight lifts it off,
//!   holding fire until an order does ([`may_fire`]).
//!
//! No type of the design gains or loses ammo by a change of kind: `Sim::change_kind` keeps the state.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::defs::{AirfieldDef, AmmoDef, ObjectKind};
use crate::hash::{StateHash, StateHasher};
use crate::ids::ObjectId;
use crate::object::Object;
use crate::production::QueueEntry;

/// An aircraft's state (module docs): its ammo and its base. (M6-GH2) Its landing is its object's (`Object::landing`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AircraftState {
    /// The rounds it has left, from none to its type's max (`AmmoDef::max`).
    pub ammo: u32,
    /// The Airfield it is assigned to (`systems/mechanics/aircraft_landing.md > Assigned base`); none while it is
    /// unassigned: one whose Airfield was destroyed, or one that no Airfield built (a debug spawn's). Left out of files
    /// while none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base: Option<ObjectId>,
}

/// The ammo of `kind`'s type, if it has one: an aircraft that rearms at an Airfield.
pub fn def_of(kind: ObjectKind) -> Option<AmmoDef> {
    kind.def().unit.as_ref()?.ammo
}

/// What a unit of `kind` starts with (module docs): its type's full ammo, unassigned; none for a type without ammo.
pub fn fresh(kind: ObjectKind) -> Option<AircraftState> {
    def_of(kind).map(|def| AircraftState {
        ammo: def.max,
        base: None,
    })
}

/// Whether `o` may attack for its ammo: it has a round left, or its weapon never runs out (module docs). The reads of
/// the units' weapons ask it (`combat::weapon`), every tick, for every unit.
#[inline]
pub fn armed(o: &Object) -> bool {
    o.aircraft.is_none_or(|aircraft| aircraft.ammo > 0)
}

/// (M6-GH2) Whether `o` may fire for its ammo and where it is: it is [`armed`], and it isn't on a spot, such as an
/// Airfield's slot,
/// landing or down (`crate::landing`; `systems/mechanics/aircraft_landing.md > Landed aircraft (every archetype)`:
/// "**Landed aircraft hold fire**: they fire only in the air"): on a slot it holds fire, as out of ammo, until an
/// order lifts it off: phase 5e's `may_attack`, the threat it is to the others' scans (`combat::threat`) and its
/// behaviours' attacks (`orders::world::Unit::attrs`) ask it. Which orders it takes doesn't change ([`armed`]): an
/// Attack lifts it off, and it attacks.
#[inline]
pub fn may_fire(o: &Object) -> bool {
    o.aircraft.is_none_or(|aircraft| aircraft.ammo > 0) && o.landing.is_none()
}

/// The Airfield `o` is assigned to, if it is an aircraft with a base.
pub fn base_of(o: &Object) -> Option<ObjectId> {
    o.aircraft.and_then(|aircraft| aircraft.base)
}

/// The Airfield support of `o`'s type (`StructureDef::airfield`), if it is an Airfield.
pub fn airfield_def(o: &Object) -> Option<AirfieldDef> {
    o.def().structure.as_ref()?.airfield
}

/// Whether `o` is an Airfield in operation (`Object::is_operational`: complete), which aircraft are assigned to.
pub fn is_airfield(o: &Object) -> bool {
    airfield_def(o).is_some() && o.is_operational()
}

/// `o` spends a round (phase 5e, `combat::step`): returns whether it was its last, one that leaves it with none. Nothing
/// for a unit without ammo, or with none left.
pub fn spend(o: &mut Object) -> bool {
    let Some(aircraft) = &mut o.aircraft else {
        return false;
    };
    if aircraft.ammo == 0 {
        return false;
    }
    aircraft.ammo -= 1;
    aircraft.ammo == 0
}

/// The places taken at `airfield` (`factions/gdo/structures/airfield.md > Aircraft Support`: "its assignments plus the
/// aircraft already in its production queue"): the aircraft among `objects` whose base it is, and the entries of its
/// production queue that train one, the one in production included. Derived, never stored (plan decision T88).
pub fn taken(objects: &BTreeMap<ObjectId, Object>, airfield: ObjectId) -> u32 {
    let assigned = (objects.values()).filter(|o| base_of(o) == Some(airfield)).count();
    let queued = (objects.get(&airfield).and_then(|o| o.production.as_ref())).map_or(0, |production| {
        production.queue.iter().filter(|entry| takes_place(entry)).count()
    });
    u32::try_from(assigned + queued).unwrap_or(u32::MAX)
}

/// The places left at `airfield` (`systems/mechanics/aircraft_landing.md > Assigned base`: "its assignments plus the
/// aircraft in its production queue must be **fewer than 4**"): its capacity less those [`taken`], none below. None for
/// an object that is no Airfield.
pub fn room(objects: &BTreeMap<ObjectId, Object>, airfield: ObjectId) -> u32 {
    let capacity = (objects.get(&airfield))
        .and_then(airfield_def)
        .map_or(0, |def| def.capacity);
    capacity.saturating_sub(taken(objects, airfield))
}

/// Whether a queue entry trains an aircraft that takes a place at its Airfield ([`taken`]).
pub fn takes_place(entry: &QueueEntry) -> bool {
    entry.unit().is_some_and(|kind| def_of(kind).is_some())
}

impl StateHash for AircraftState {
    fn hash(&self, h: &mut StateHasher) {
        let AircraftState { ammo, base } = self;
        ammo.hash(h);
        base.hash(h);
    }
}

#[cfg(test)]
mod tests;
