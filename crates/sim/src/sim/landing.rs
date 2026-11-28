//! Landing in the game (`crate::landing`): the one step right after phase 5a that flies every aircraft that lands on
//! to its spot, on a structure or on open ground, a Glider by its landing approach, lands it there and counts its
//! landing's frames ([`Sim::step_landings`]); an Airfield's new aircraft, landed on a free slot
//! ([`Sim::release_onto_slot`]); and the aircraft that go with a structure destroyed under them ([`Sim::landed_on`]).

use std::collections::BTreeSet;

use glam::Vec2;

use super::Sim;
use super::drones::died;
use crate::aircraft;
use crate::defs::Domain;
use crate::economy::RATIO_ONE;
use crate::events::SimEvent;
use crate::ids::ObjectId;
use crate::landing::{self, Landing, Queue, Spot, User};
use crate::math;
use crate::mover::{Approach, MoveGoal, MoveStatus, Mover};
use crate::object::Object;
use crate::paths::{blocking, box_overlaps_rect};
use crate::production::QueueEntry;
use crate::progress::Progress;
use crate::ruleset::rules;
use crate::shape::Shape;
use crate::units::{SU_PER_GU, Tick, TilePos};

impl Sim {
    /// The landing step, right after phase 5a (`crate::landing`), for the units before `first_new` as 5a's orders and
    /// the supply loop's attachments (`Sim::step_supply_runs`) leave them:
    /// - **Each landing under way** takes a frame; one whose frames are all done is down, a ground target from here on
    ///   (`landing::down`); on an Airfield it begins to rearm in this tick's economy phase (`landing::rearm`).
    /// - **Each aircraft flying to a structure to land** (`landing::users`, `landing::flies_to`: an aircraft flying
    ///   home, `orders::air_base`; a chopper's supply run, `orders::supply_run`), structure by structure in id order,
    ///   flies on to the spot it takes ([`Sim::fly_to_structure`]).
    /// - **Each unit asked to land on open ground** in this tick (`Mover::land_at`), in id order, flies on to its spot
    ///   there ([`Sim::fly_to_field`]).
    ///
    /// Here, as the entries (`Sim::step_entries`): after the orders, which start the flights and lift the aircraft off,
    /// and before 5b to 5f, so that the tick's movement flies each to its spot, and its combat sees each as the step
    /// leaves it.
    pub(super) fn step_landings(&mut self, first_new: ObjectId) {
        let fields: Vec<(ObjectId, Vec2)> = (self.objects.range_mut(..first_new))
            .filter_map(|(&id, o)| Some((id, o.mover.as_mut()?.field.take()?)))
            .collect();
        for (_, o) in self.objects.range_mut(..first_new) {
            if let Some(landing) = &mut o.landing
                && !landing.descent.done()
            {
                landing.descent.step(RATIO_ONE);
            }
        }
        let users = landing::users(self.objects.range(..first_new).map(|(_, o)| o));
        let homes: BTreeSet<ObjectId> = (users.iter())
            .filter(|user| user.on.is_none())
            .filter_map(|user| user.home)
            .collect();
        for home in homes {
            self.fly_to_structure(home, &users, first_new);
        }
        let mut taken = Vec::new();
        for (id, point) in fields {
            self.fly_to_field(id, point, &mut taken);
        }
    }

    /// The flights of `users` to structure `at` to land on it ([`Sim::step_landings`]), each to the spot it takes
    /// (`landing::claim`, of the spots its def gives: `landing::pads`), or to where the structure has it wait while it
    /// takes none (an Airfield's centre, the one spot of a tower or a station), and back to it once it has come to rest
    /// off it. Once it has come to rest over its spot (`Air::landing_reach_su`; plan decision T88: "a HoverCraft
    /// brakes to a stop over its pad") it lands there: its landing begins, none of its frames done, in its own landing
    /// time (`LandingDef::landing_ticks`) or else the structure's (`AirfieldDef::landing_ticks`), and it holds still
    /// (`Mover::stop`), an air target until it is down. **A Glider**, which can't come to rest by itself
    /// (`movement::glider`), flies its landing approach to its spot instead (`Mover::approach_to`,
    /// `movement::approach`): it slows to IdleSpeed for the last of the way and stops over the spot, where it lands as
    /// above, held still from then on; with no spot free it flies to where it waits and circles there
    /// (`movement::glider`), until one is. The supply loop's rules on its spots (`Sim::spot_rules`): only a tower's
    /// attached chopper takes its spot, the others' flights there ending; a chopper flying to a station lands only
    /// once its Supplies are due by the time it is down, and waits over it till then.
    fn fly_to_structure(&mut self, at: ObjectId, users: &[User], first_new: ObjectId) {
        let Some(structure) = self.objects.get(&at) else {
            return;
        };
        let Some(pads) = landing::pads(structure) else {
            return;
        };
        let (only, due) = self.spot_rules(structure, first_new);
        let claims = landing::claim(&pads, users, only);
        let reach = rules().tuning.air.landing_reach_su;
        let tick = self.tick;
        let flying = users.iter().filter(|user| user.home == Some(at) && user.on.is_none());
        for user in flying {
            if only.is_some_and(|only| only != user.id) {
                self.end_flight(user.id);
                continue;
            }
            let spot =
                (claims.iter().position(|claim| *claim == Some(user.id))).and_then(|i| pads.spots.get(i).copied());
            let Some(o) = self.objects.get_mut(&user.id) else {
                continue;
            };
            let own = landing::def_of(o.kind).map(|def| def.landing_ticks);
            let Some(ticks) = own.or(pads.landing_ticks) else {
                continue;
            };
            let glides = o.def().unit.as_ref().is_some_and(|unit| unit.glides());
            let Some(mover) = o.mover.as_mut() else {
                continue;
            };
            let to = spot.unwrap_or(pads.wait);
            // Its move over: an arrival holds it still from its next tick on (`movement::drag`'s braking approach).
            // (M6-GL) A Glider's, once it has stopped over its spot at the end of its landing approach
            // (`movement::approach`).
            let at_rest = if glides {
                mover.is_over()
            } else {
                mover.status != MoveStatus::Moving
            };
            let near = o.pos.distance_squared(to) <= reach * reach;
            let waits = due.is_some_and(|due| due > ticks);
            if spot.is_some() && at_rest && near && !waits {
                // Its frames are positive (the sign's rule, `Ruleset::check`): it is down from a later step on.
                mover.stop();
                o.landing = Some(Landing {
                    at: Spot::Structure(at),
                    descent: Progress::new(ticks),
                    rearm: None,
                });
                continue;
            }
            if glides {
                glide_home(mover, spot, pads.wait, tick);
                continue;
            }
            // On its way there it flies on; come to rest where it waits, it waits there; the one that takes a tower's
            // or a station's spot, pushed off it by the separation of one waiting beside it (`movement::air`), flies
            // back.
            let pushed_off = pads.queue == Queue::One && spot.is_some() && at_rest && !near;
            if (mover.goal != MoveGoal::Point(to) && !(at_rest && near)) || pushed_off {
                mover.go_to(MoveGoal::Point(to), None, tick);
            }
        }
    }

    /// Unit `id`'s Field landing at `point`, which its order asked for in this tick (`Mover::land_at`;
    /// `systems/mechanics/aircraft_landing.md > Field landing`), `taken` the spots of those before it in the tick: it
    /// flies to the free spot nearest `point` ([`Sim::field_spot`]), a Glider by its landing approach
    /// (`Mover::approach_to`), and lands there once it has come to rest over it, a Glider stopped over it, in its own
    /// landing time (`LandingDef::landing_ticks`), as on a structure's spot ([`Sim::fly_to_structure`]). One that has
    /// landed already, anywhere, flies no more: its order learns it by its landing (`landing::down`). One with no
    /// landing time of its own, or no free spot within the search's reach, does nothing.
    fn fly_to_field(&mut self, id: ObjectId, point: Vec2, taken: &mut Vec<Shape>) {
        let Some(o) = self.objects.get(&id).filter(|o| o.landing.is_none()) else {
            return;
        };
        let Some(def) = landing::def_of(o.kind) else {
            return;
        };
        let Some(spot) = self.field_spot(o, point, taken) else {
            return;
        };
        taken.push(Shape::silhouette(o.kind, spot, o.facing));
        let (reach, tick) = (rules().tuning.air.landing_reach_su, self.tick);
        let Some(o) = self.objects.get_mut(&id) else {
            return;
        };
        let glides = o.def().unit.as_ref().is_some_and(|unit| unit.glides());
        let Some(mover) = o.mover.as_mut() else {
            return;
        };
        let at_rest = if glides {
            mover.is_over()
        } else {
            mover.status != MoveStatus::Moving
        };
        if at_rest && o.pos.distance_squared(spot) <= reach * reach {
            mover.stop();
            o.landing = Some(Landing {
                at: Spot::Field(spot),
                descent: Progress::new(def.landing_ticks),
                rearm: None,
            });
            return;
        }
        if glides {
            if mover.goal != MoveGoal::Point(spot) || mover.approach != Some(Approach::Flying) {
                mover.approach_to(spot, tick);
            }
        } else if mover.goal != MoveGoal::Point(spot) || at_rest {
            mover.go_to(MoveGoal::Point(spot), None, tick);
        }
    }

    /// The free spot nearest `point` for unit `o` to land on, if one is within the search's reach
    /// (`landing::field_points`; `systems/mechanics/aircraft_landing.md > Field landing`: "only on ground infantry can
    /// walk on, with no building or unit in the way"): its Silhouette there, facing as it does now, on the map, over
    /// tiles infantry walk on (Traversible: `NavClass::GroundRugged`, the grid of the infantry bases), clear of every
    /// blocking footprint, of every ground unit, of every unit landed on open ground or taking off, and of `taken`.
    /// **Placeholder (not in the design: what blocks a Field landing)**: those, the ground grid's rules for infantry.
    pub(super) fn field_spot(&self, o: &Object, point: Vec2, taken: &[Shape]) -> Option<Vec2> {
        let (kind, facing) = (o.kind, o.facing);
        let far = landing::FIELD_SEARCH_SU + 4.0 * SU_PER_GU;
        let nav = self.nav.nav().rules();
        let footprints: Vec<_> = self.objects.values().filter_map(|other| blocking(other, nav)).collect();
        let in_the_way = |other: &&Object| {
            let ground = (other.def().unit.as_ref()).is_some_and(|unit| unit.domain() == Domain::Ground);
            let afield = other.landing.is_some_and(|landing| landing.at.field().is_some());
            let rising = other.mover.as_ref().is_some_and(|mover| mover.rise.is_some());
            other.id != o.id && other.def().silhouette().is_some() && (ground || afield || rising)
        };
        let shapes: Vec<Shape> = (self.objects.values())
            .filter(|other| other.pos.distance(point) <= far)
            .filter(in_the_way)
            .map(Shape::of)
            .chain(taken.iter().copied())
            .collect();
        landing::field_points(point).find(|&spot| {
            let shape = Shape::silhouette(kind, spot, facing);
            let half = shape.bound_half();
            let (lo, hi) = (TilePos::containing(spot - half), TilePos::containing(spot + half));
            let walkable = (lo.y..=hi.y).all(|y| {
                (lo.x..=hi.x).all(|x| (self.map.tile_preset(TilePos::new(x, y))).is_some_and(|tile| tile.traversible))
            });
            self.map.clamp_box(spot, half) == spot
                && walkable
                && !footprints.iter().any(|&rect| box_overlaps_rect(spot, half, rect))
                && !shapes.iter().any(|other| shape.overlaps(other))
        })
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

#[cfg(test)]
mod tests;
