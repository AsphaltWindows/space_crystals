//! (M6-GH2) Landing at an Airfield, in the game (`crate::landing`, plan decision T88): the step right after phase 5a
//! that sends the aircraft flying home on to their slots, (M6-GL) a Glider by its landing approach, lands them there
//! and counts their landings ([`Sim::step_landings`]); an Airfield's new aircraft, landed on a free slot
//! ([`Sim::release_onto_slot`]); and the aircraft that go with an Airfield destroyed under them ([`Sim::landed_on`]).

use std::collections::BTreeSet;

use glam::Vec2;

use super::Sim;
use super::drones::died;
use crate::aircraft;
use crate::economy::RATIO_ONE;
use crate::events::SimEvent;
use crate::ids::ObjectId;
use crate::landing::{self, Landing, Spot, User};
use crate::math;
use crate::mover::{Approach, MoveGoal, MoveStatus, Mover};
use crate::production::QueueEntry;
use crate::progress::Progress;
use crate::ruleset::rules;
use crate::units::Tick;

impl Sim {
    /// The landing step, right after phase 5a (`crate::landing`), for the aircraft before `first_new`, as 5a's orders
    /// leave them:
    /// - **Each landing under way** takes a frame (`AirfieldDef::landing_ticks`); one whose frames are all done is
    ///   down, a ground target from here on, and begins to rearm in this tick's economy phase (`landing::rearm`).
    /// - **Each aircraft flying home** (`orders::air_base`), Airfield by Airfield in id order, flies on to the slot it
    ///   takes (`landing::claims`), or over the Airfield's centre while none is free, and back to it once it has come
    ///   to rest off it. Once it has come to rest over its slot (`Air::landing_reach_su`; plan decision T88: "a
    ///   HoverCraft brakes to a stop over its pad") it lands there: its landing begins, none of its frames done, and it
    ///   holds still (`Mover::stop`), an air target until it is down.
    /// - (M6-GL) **A Glider flying home**, which can't come to rest by itself (`movement::glider`), flies its landing
    ///   approach to its slot instead (`Mover::approach_to`, `movement::approach`): it slows to IdleSpeed for the last
    ///   of the way and stops over the slot, where it lands as above, held still from then on (`movement::approach`).
    ///   With no slot free it flies to the Airfield's centre and circles there (`movement::glider`), until one is.
    ///
    /// Here, as the entries (`Sim::step_entries`): after the orders, which start the flights and lift the aircraft off,
    /// and before 5b to 5f, so that the tick's movement flies each to its slot, and its combat sees each as the step
    /// leaves it.
    pub(super) fn step_landings(&mut self, first_new: ObjectId) {
        let users = landing::users(self.objects.range(..first_new).map(|(_, o)| o));
        if users.is_empty() {
            return;
        }
        for user in users.iter().filter(|user| user.on.is_some()) {
            self.land_a_frame(user.id);
        }
        let homes: BTreeSet<ObjectId> = (users.iter())
            .filter(|user| user.on.is_none())
            .filter_map(|user| user.home)
            .collect();
        for home in homes {
            self.fly_home(home, &users);
        }
    }

    /// One frame of aircraft `id`'s landing, if it isn't down yet (module docs).
    fn land_a_frame(&mut self, id: ObjectId) {
        let landing = (self.objects.get_mut(&id))
            .filter(|o| o.aircraft.is_some())
            .and_then(|o| o.landing.as_mut());
        if let Some(landing) = landing
            && !landing.descent.done()
        {
            landing.descent.step(RATIO_ONE);
        }
    }

    /// The flights home to Airfield `home` of `users` (module docs, [`Sim::step_landings`]).
    fn fly_home(&mut self, home: ObjectId, users: &[User]) {
        let Some(airfield) = self.objects.get(&home).filter(|o| aircraft::is_airfield(o)) else {
            return;
        };
        let centre = airfield.pos;
        let slots: Vec<Vec2> = landing::slots(airfield).collect();
        let claims = landing::claims(airfield, users);
        let ticks = aircraft::airfield_def(airfield).map_or(0, |def| def.landing_ticks);
        let reach = rules().tuning.air.landing_reach_su;
        let tick = self.tick;
        let flying = users.iter().filter(|user| user.home == Some(home) && user.on.is_none());
        for user in flying {
            let slot = (claims.iter().position(|claim| *claim == Some(user.id))).and_then(|i| slots.get(i).copied());
            let Some(o) = self.objects.get_mut(&user.id) else {
                continue;
            };
            let glides = o.def().unit.as_ref().is_some_and(|unit| unit.glides());
            let (Some(mover), true) = (o.mover.as_mut(), o.aircraft.is_some()) else {
                continue;
            };
            let to = slot.unwrap_or(centre);
            // Its move over: an arrival holds it still from its next tick on (`movement::drag`'s braking approach).
            // (M6-GL) A Glider's, once it has stopped over its slot at the end of its landing approach
            // (`movement::approach`).
            let at_rest = if glides {
                mover.is_over()
            } else {
                mover.status != MoveStatus::Moving
            };
            let near = o.pos.distance_squared(to) <= reach * reach;
            if slot.is_some() && at_rest && near {
                // Its frames are positive (the sign's rule, `Ruleset::check`): it is down from a later step on.
                mover.stop();
                o.landing = Some(Landing {
                    at: Spot::Structure(home),
                    descent: Progress::new(ticks),
                    rearm: None,
                });
                continue;
            }
            if glides {
                glide_home(mover, slot, centre, tick);
                continue;
            }
            // On its way there it flies on; come to rest over the centre while no slot is free, it waits there.
            if mover.goal != MoveGoal::Point(to) && !(at_rest && near) {
                mover.go_to(MoveGoal::Point(to), None, tick);
            }
        }
    }

    /// `ExitRule::Pad`: `producer`'s finished aircraft comes into the game landed on the first slot of the Airfield
    /// that no aircraft takes (`landing::claims`; design change 0428 point 2: "New aircraft appear **landed on a free
    /// slot**; they take off for the Airfield's rally point, and with no rally point they stay landed"), down and full,
    /// facing its rally point if it has one (east if not, or if the rally point is on its slot), with the order its
    /// rally point gives, which lifts it off in its first tick (`landing::take_off`). It uses no doorway
    /// (`core/entities.md > Structure Type > Exit Doorway`: "Aircraft [...] are not covered by this rule"). While no
    /// slot is free (an aircraft rebased elsewhere still on one, or one rising from it: `landing::claims`), it stays
    /// inside, finished, as a unit behind an occupied doorway does (`Sim::release_unit`;
    /// `systems/mechanics/aircraft_landing.md > Assigned base`: "No free spot: the finished aircraft waits inside"). It
    /// is assigned to the Airfield that built it (`systems/mechanics/aircraft_landing.md > Assigned base`: "An aircraft
    /// is assigned to the Airfield that built it"). A producer with no landing slots (no Airfield) lets it out over its
    /// centre, in the air. It replaced slice M6-GH1's release over the Airfield's centre (D34).
    pub(super) fn release_onto_slot(&mut self, producer: ObjectId) {
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
        let support = aircraft::airfield_def(building);
        let pos = match support {
            Some(_) => {
                let users = landing::users(self.objects.values());
                let claims = landing::claims(building, &users);
                let free = landing::slots(building).zip(claims).find(|(_, claim)| claim.is_none());
                let Some((slot, _)) = free else {
                    return;
                };
                slot
            }
            None => building.pos,
        };
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
            if let (Some(state), Some(support)) = (o.aircraft.as_mut(), support) {
                state.base = Some(producer);
                o.landing = Some(landing::down_on(producer, support.landing_ticks));
            }
        }
        self.events.push(SimEvent::UnitProduced { producer, unit: id });
        if let Some(kind) = rally.and_then(|rally| self.rally_order(id, rally)) {
            self.give_order(&[id], kind, false);
        }
    }

    /// (M6-GH2) The removal path (`Sim::remove_objects`), for the objects `ids` about to leave the game: the aircraft
    /// landed on the Airfields among them, landing or down, those not among `ids`, in id order, each with a
    /// `SimEvent::Died` (design change 0428 point 3, `systems/mechanics/aircraft_landing.md > Assigned base`:
    /// "**Aircraft landed on it are destroyed with it**: they don't take off by themselves. Designer's reading: an
    /// aircraft still **landing** (not yet down) counts as landed and is destroyed; one already **taking off**
    /// escapes"). One taking off has no landing any more (`landing::take_off`): it flies on, unassigned as the
    /// Airfield's other aircraft are.
    pub(super) fn landed_on(&mut self, ids: &[ObjectId]) -> Vec<ObjectId> {
        let landed: Vec<ObjectId> = (self.objects.values())
            .filter(|o| landing::at(o).is_some_and(|at| ids.contains(&at)) && !ids.contains(&o.id))
            .map(|o| o.id)
            .collect();
        for id in &landed {
            if let Some(o) = self.objects.get(id) {
                self.events.push(died(o));
            }
        }
        landed
    }
}

/// (M6-GL) A Glider's flight home to an Airfield centred on `centre`, as the landing step leaves it in `tick` (module
/// docs, [`Sim::step_landings`]): its landing approach to `slot`, the slot it takes, unless it flies it already (a new
/// one from wherever it is, over another slot or its approach's way); with none free, on to the centre, to circle there
/// as a Glider does at its goal (`movement::glider`), unless it circles there already. **Placeholder (Q0192 point
/// 2)**, the designer's (`systems/mechanics/aircraft_landing.md > Vertical`: "**No free spot** at its base: a Glider
/// circles over the structure's centre until one frees").
fn glide_home(mover: &mut Mover, slot: Option<Vec2>, centre: Vec2, tick: Tick) {
    match slot {
        Some(slot) => {
            if mover.goal != MoveGoal::Point(slot) || mover.approach != Some(Approach::Flying) {
                mover.approach_to(slot, tick);
            }
        }
        None => {
            if mover.goal != MoveGoal::Point(centre) && mover.orbit != Some(centre) {
                mover.go_to(MoveGoal::Point(centre), None, tick);
            }
        }
    }
}
