//! (M6-GH1) The aircraft that rearm at an Airfield, in the game (`crate::aircraft`, plan decision T88): a Rebase,
//! applied as it is given ([`Sim::rebase`]); and the return of the aircraft whose last round went in phase 5e
//! ([`Sim::send_home`]). (M6-GH2) An Airfield's new aircraft, landed on a free slot and assigned to it, and the
//! landings are `sim/landing.rs`'s (`Sim::release_onto_slot`, `Sim::step_landings`).

use crate::aircraft;
use crate::ids::ObjectId;
use crate::orders::OrderKind;
use crate::sim::Sim;

impl Sim {
    /// A Rebase of aircraft `id` to `airfield`, applied as it is given (`Sim::give_order`; plan decision T88:
    /// "applied when issued: the aircraft is then flying to its new base"): whether the aircraft takes the order. One
    /// assigned to it already does; another does if `airfield` is an Airfield of its owner's in operation with room
    /// for it (`aircraft::room`), and is assigned to it now. The gate let through as many as it had room for; this
    /// checks again for a rally point's order, which no gate judges, and for the units of a command after which the
    /// Airfield has filled.
    pub(super) fn rebase(&mut self, id: ObjectId, airfield: ObjectId) -> bool {
        let Some(o) = self.objects.get(&id).filter(|o| o.aircraft.is_some()) else {
            return false;
        };
        if aircraft::base_of(o) == Some(airfield) {
            return true;
        }
        let owner = o.owner;
        let fits = (self.objects.get(&airfield)).is_some_and(|a| a.owner == owner && aircraft::is_airfield(a))
            && aircraft::room(&self.objects, airfield) > 0;
        if let Some(state) = (self.objects.get_mut(&id))
            .and_then(|o| o.aircraft.as_mut())
            .filter(|_| fits)
        {
            state.base = Some(airfield);
        }
        fits
    }

    /// The aircraft whose last round went in this tick's phase 5e (`emptied`, `combat::step`) fly home by themselves
    /// (`systems/mechanics/aircraft_landing.md > Assigned base`: "**Automatic return** when out of ammo: it flies to
    /// its base, lands and rearms"): each that is still in the game with a base gets a Return to Base in place of its
    /// orders, which any order replaces in turn ("Any other order interrupts it (it stays out of ammo)"). **Placeholder
    /// (Q0195)**: the orders queued behind the one it ran out in are dropped with it, as the Specter's
    /// (`systems/mechanics/aircraft_landing.md > Network base`: "it ignores later queued orders and goes home to
    /// rearm"); an order Shift-queued behind the return itself starts as it lands ("**Shift-queued orders run as soon
    /// as it has landed**", `orders::air_base`). One killed in the tick, or unassigned by its Airfield's fall in it,
    /// gets nothing ("**Base destroyed**: [...] an unassigned aircraft can't rearm or return until reassigned"). After
    /// the removal of the dead.
    pub(super) fn send_home(&mut self, emptied: &[ObjectId]) {
        for &id in emptied {
            if self.objects.get(&id).and_then(aircraft::base_of).is_some() {
                self.give_order(&[id], OrderKind::ReturnToBase, false);
            }
        }
    }
}
