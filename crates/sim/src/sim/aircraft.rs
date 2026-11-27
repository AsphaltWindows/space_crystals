//! (M6-GH1) The aircraft that rearm at an Airfield, in the game (`crate::aircraft`, plan decision T88): an Airfield's
//! new aircraft, over it and assigned to it ([`Sim::release_over_pads`]); a Rebase, applied as it is given
//! ([`Sim::rebase`]); and the return of the aircraft whose last round went in phase 5e ([`Sim::send_home`]).

use crate::aircraft;
use crate::events::SimEvent;
use crate::ids::ObjectId;
use crate::math;
use crate::orders::OrderKind;
use crate::production::QueueEntry;
use crate::sim::Sim;

impl Sim {
    /// `ExitRule::Pad` (**Placeholder (D34 for slice M6-GH1)**: "in GH1, a new Hornet appears over its Airfield"):
    /// `producer`'s finished aircraft comes into the game over the centre of its footprint, facing its rally point if
    /// it has one (east if not, or if the rally point is the centre), with the order its rally point gives
    /// (`factions/gdo/structures/airfield.md > Returning and rearming`: "they take off for the Airfield's rally point,
    /// and with no rally point they stay"), hovering there without one. It uses no doorway (`core/entities.md >
    /// Structure Type > Exit Doorway`: "Aircraft [...] are not covered by this rule") and never waits: the air units'
    /// separation spreads the ones over the Airfield (`movement::air`). It is assigned to the Airfield that built it
    /// (`factions/gdo/structures/airfield.md > Assignment`: "An aircraft is assigned to the Airfield that built it").
    /// Slice M6-GH2 lands it on a free pad instead.
    pub(super) fn release_over_pads(&mut self, producer: ObjectId) {
        let Some(building) = self.objects.get(&producer) else {
            return;
        };
        let (Some(owner), Some(production)) = (building.owner, &building.production) else {
            return;
        };
        let (Some(kind), rally) = (production.current().and_then(QueueEntry::unit), production.rally) else {
            return;
        };
        if kind.def().unit.is_none() || self.next_id == u32::MAX {
            return;
        }
        let pos = building.pos;
        let assigns = aircraft::airfield_def(building).is_some();
        let toward = rally
            .map(|rally| rally.point() - pos)
            .filter(|toward| toward.length_squared() > 0.0);
        if let Some(production) = self.objects.get_mut(&producer).and_then(|o| o.production.as_mut()) {
            production.release();
        }
        let id = self.insert_object(kind, Some(owner), pos, None, None);
        if let Some(o) = self.objects.get_mut(&id) {
            o.facing = toward.map_or(0.0, math::vector_angle);
            o.prev_facing = o.facing;
            if let Some(state) = o.aircraft.as_mut().filter(|_| assigns) {
                state.base = Some(producer);
            }
        }
        self.events.push(SimEvent::UnitProduced { producer, unit: id });
        if let Some(kind) = rally.and_then(|rally| self.rally_order(id, rally)) {
            self.give_order(&[id], kind, false);
        }
    }

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
    /// (`factions/gdo/structures/airfield.md > Returning and rearming`: "**Automatic return**: when an aircraft runs
    /// out of ammo, it automatically flies back to its assigned Airfield"): each that is still in the game with a base
    /// gets a Return to Base in place of its orders, which any order replaces in turn ("The automatic return **can be
    /// interrupted** by giving the aircraft any other order"). **Placeholder (not in the design: the orders queued
    /// behind the one it ran out in)**: dropped with it, as the Specter's (`factions/syndicate/units/specter.md >
    /// Attack`: "it **ignores the orders queued after the Attack** and flies home, where they are wiped"). One killed
    /// in the tick, or unassigned by its Airfield's fall in it, gets nothing: "An unassigned aircraft has no base: it
    /// doesn't return automatically". After the removal of the dead.
    pub(super) fn send_home(&mut self, emptied: &[ObjectId]) {
        for &id in emptied {
            if self.objects.get(&id).and_then(aircraft::base_of).is_some() {
                self.give_order(&[id], OrderKind::ReturnToBase, false);
            }
        }
    }
}
