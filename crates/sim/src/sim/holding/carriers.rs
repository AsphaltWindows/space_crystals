//! (M6-ST1) Carriers in a game (`crate::carry`, plan decision T94): their passengers as the sim shows them, the places
//! they take ([`Sim::room`] counts them), the gate's rule for an Unload All, its step in the economy phase, and the
//! passengers' part of the removal path. The way aboard is the Enter order's (`Sim::board`, `sim/network/entries.rs`).
//!
//! - **Unloading** ([`Sim::step_unloads`]): a passenger comes back onto the map as every held unit does, the object it
//!   was (plan decision T35), in the economy phase, after 5d, so that it never moves in the tick it appears, and before
//!   5f, which sees it (`sim/network.rs`, *Old ids come back*).
//! - **Death** ([`Sim::passengers_of`], [`Sim::passengers_die`]): `Sim::remove_objects` and `Sim::remove_held` take a
//!   carrier's passengers along.

use glam::Vec2;

use crate::carry;
use crate::defs::{CarrierDef, ObjectKind};
use crate::events::SimEvent;
use crate::ids::ObjectId;
use crate::math;
use crate::network::{Held, Place};
use crate::object::Object;
use crate::orders::unload;
use crate::ruleset::rules;
use crate::sim::{Reject, Sim};
use crate::vision::Hiding;

impl Sim {
    /// The passengers aboard `carrier` (`network::Place::Carrier`), longest aboard first: by `Held::since`, ties by id.
    /// None for anything but a carrier, wherever it is: on the map, or held in its owner's Tunnel Network.
    pub fn passengers(&self, carrier: ObjectId) -> Vec<&Held> {
        let mut aboard: Vec<&Held> = (self.held.values())
            .filter(|held| held.place == Place::Carrier(carrier))
            .collect();
        aboard.sort_by_key(|held| (held.since, held.object.id));
        aboard
    }

    /// The places `carrier`'s passengers take aboard its hold `def` (`carry::places`): one for a passenger of a base
    /// the hold doesn't list (a ruleset that changed the hold under it).
    pub(in crate::sim) fn places_taken(&self, carrier: ObjectId, def: &CarrierDef) -> u32 {
        (self.held.values())
            .filter(|held| held.place == Place::Carrier(carrier))
            .map(|held| carry::places(def, held.object.kind).unwrap_or(1))
            .fold(0, u32::saturating_add)
    }

    /// The passengers of the objects `ids`, and theirs in turn (a passenger that carries: none can yet), each once, in
    /// id order: the units that leave the game with them (`crate::carry`, *Death*). None without a carrier among
    /// `ids`, on the map or held, which is all the removal path asks of most removals.
    pub(in crate::sim) fn passengers_of(&self, ids: &[ObjectId]) -> Vec<ObjectId> {
        let carries = |id: &ObjectId| {
            let o = (self.objects.get(id)).or_else(|| self.held.get(id).map(|held| &held.object));
            o.is_some_and(|o| carry::def_of(o).is_some())
        };
        if self.held.is_empty() || !ids.iter().any(carries) {
            return Vec::new();
        }
        let mut found: Vec<ObjectId> = Vec::new();
        let mut carriers: Vec<ObjectId> = ids.to_vec();
        carriers.sort_unstable();
        while !carriers.is_empty() {
            let aboard: Vec<ObjectId> = (self.held.values())
                .filter(|held| matches!(held.place, Place::Carrier(c) if carriers.binary_search(&c).is_ok()))
                .map(|held| held.object.id)
                .filter(|id| !found.contains(id))
                .collect();
            found.extend(&aboard);
            carriers = aboard;
        }
        found.sort_unstable();
        found
    }

    /// The passengers `passengers` die with their carriers, which are leaving the game ([`Sim::passengers_of`]; the
    /// carriers' `SimEvent::Died` came before): each with a `SimEvent::Died` where the carrier it is in, on the map,
    /// was, hidden from whom the carrier was hidden from (`Hiding::of`). A passenger whose carrier is held has none: it
    /// leaves the game with it (`Sim::remove_held`).
    pub(in crate::sim) fn passengers_die(&mut self, passengers: &[ObjectId]) {
        for &id in passengers {
            let Some(held) = self.held.get(&id) else {
                continue;
            };
            let Some(carrier) = self.outermost_carrier(id) else {
                continue;
            };
            self.events.push(SimEvent::Died {
                id,
                kind: held.object.kind,
                owner: held.object.owner,
                pos: carrier.pos,
                hiding: Hiding::of(carrier),
            });
        }
    }

    /// The carrier on the map that held unit `id` is aboard of, directly or in a carrier aboard it: `None` for a unit
    /// held elsewhere, or aboard a carrier that is held itself.
    fn outermost_carrier(&self, id: ObjectId) -> Option<&Object> {
        let mut place = self.held.get(&id)?.place;
        // One step for each carrier, at most as many as there are held units.
        for _ in 0..=self.held.len() {
            let Place::Carrier(carrier) = place else {
                return None;
            };
            if let Some(o) = self.objects.get(&carrier) {
                return Some(o);
            }
            place = self.held.get(&carrier)?.place;
        }
        None
    }

    /// The gate for an Unload All (`crate::carry`, `orders::unload`): the carriers among `units` take it
    /// (`UnitDef::carries`), else [`Reject::LacksAbility`]; of them those with a passenger aboard, else
    /// [`Reject::NothingInside`], as an Eject All with no one inside
    /// (`factions/cults/structures/armory.md > ObjectInterfaceState[Armory]`: "Only available if StoredRecruits is not
    /// empty"). Whether one unloads now, above ground (`carry::loads`), is its type's rule, checked last
    /// (`vehicle_deploy::can_take`). A queued Unload All is judged as it is given: one that starts with no one aboard
    /// ends at once.
    pub(in crate::sim) fn check_unload(&self, units: Vec<ObjectId>) -> Result<Vec<ObjectId>, Reject> {
        let carriers: Vec<ObjectId> = (units.into_iter())
            .filter(|id| self.objects.get(id).is_some_and(|o| carry::def_of(o).is_some()))
            .collect();
        if carriers.is_empty() {
            return Err(Reject::LacksAbility);
        }
        let loaded: Vec<ObjectId> = (carriers.into_iter())
            .filter(|&id| self.held.values().any(|held| held.place == Place::Carrier(id)))
            .collect();
        if loaded.is_empty() {
            return Err(Reject::NothingInside);
        }
        Ok(loaded)
    }

    /// The unloads (`orders::unload`), in the economy phase (module docs): each carrier before `first_new` whose
    /// current order is an Unload All, in id order, once it is at rest (`Mover::momentum`: it brakes off its speed from
    /// the order's start, which stopped it), ends it and unloads ([`Sim::unload`]); one that doesn't load now
    /// (`carry::loads`: a hand-edited save's) ends it with no unload.
    pub(in crate::sim) fn step_unloads(&mut self, first_new: ObjectId) {
        let unloading: Vec<ObjectId> = (self.objects.range(..first_new))
            .filter(|(_, o)| unload::unloading(o))
            .map(|(&id, _)| id)
            .collect();
        for carrier in unloading {
            let Some(o) = self.objects.get_mut(&carrier) else {
                continue;
            };
            let loads = carry::loads(o);
            let at_rest = o.mover.as_ref().is_none_or(|mover| mover.momentum == Vec2::ZERO);
            if loads && !at_rest {
                continue;
            }
            if let Some(orders) = &mut o.orders {
                orders.current = None;
            }
            if loads {
                self.unload(carrier);
            }
        }
    }

    /// `carrier` unloads (`crate::carry`, *Unload All*): its passengers, longest aboard first ([`Sim::passengers`]),
    /// each at the first free spot of the rings round it, nearest first (`carry::ring_spots`, `Holding::unload_rings`
    /// of them, spaced for the largest of them): of `n` passengers the `i`th tries a ring's spots from its
    /// `i × spots / n`th on, round the ring, so that they spread round the carrier. A spot is free where the unit fits
    /// facing as the carrier does, on the map, by the rule of every unit put on it (`Sim::check_unit_fits_facing`),
    /// those out before it included. Each comes out so, idle, with a `SimEvent::Unloaded`; one for whom no ring has a
    /// free spot stays aboard.
    fn unload(&mut self, carrier: ObjectId) {
        let passengers: Vec<(ObjectId, ObjectKind)> = (self.passengers(carrier).into_iter())
            .map(|held| (held.object.id, held.object.kind))
            .collect();
        let Some(o) = self.objects.get(&carrier) else {
            return;
        };
        if passengers.is_empty() {
            return;
        }
        // The circle round a passenger's centre that holds its Silhouette however it lies.
        let radius = (passengers.iter())
            .filter_map(|&(_, kind)| Some((kind.def().silhouette()? / 2.0).length()))
            .fold(0.0, f32::max);
        let facing = o.facing;
        let rings: Vec<Vec<Vec2>> = (0..rules().tuning.holding.unload_rings)
            .map(|ring| carry::ring_spots(o, radius, ring))
            .collect();
        let count = passengers.len();
        for (i, &(unit, kind)) in passengers.iter().enumerate() {
            let fits =
                |p: Vec2| self.map.contains_point(p) && self.check_unit_fits_facing(kind, p, facing, None).is_ok();
            let spot = rings.iter().find_map(|spots| {
                let n = spots.len();
                (0..n).map(|k| spots[(i * n / count + k) % n]).find(|&p| fits(p))
            });
            let Some(pos) = spot else {
                continue;
            };
            if !self.put_on_map(unit, pos, math::facing_vector(facing)) {
                continue;
            }
            // Its facing to the bit, as its spot was judged.
            if let Some(out) = self.objects.get_mut(&unit) {
                out.facing = facing;
                out.prev_facing = facing;
            }
            self.events.push(SimEvent::Unloaded { unit, carrier });
        }
    }
}

#[cfg(test)]
mod tests;
