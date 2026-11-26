//! Buildings that hold units in a game (`crate::holding`, plan decision T60, slice M5-F1): their pools as the sim shows
//! them, the gate's rule for their commands, Eject All, and the step of the economy phase that lets the units ready to
//! leave out. The way in is the Enter order's (`sim/network/entries.rs`); the way out when the building is destroyed
//! is `holding/evacuation.rs`'s; a producer that trains its units inside is `sim/producers/pool.rs`'s (slice M5-F2).
//!
//! - **The pool** ([`Sim::pool`]): the units inside that no entry of the building's queue reserves and that aren't on
//!   their way out; **the room** ([`Sim::room`]) counts every unit inside.
//! - **Eject All** (`factions/cults/structures/armory.md > Eject All`: "Immediate command that ejects all stored
//!   Recruits from the exit side in a rapid stream, one after another"): the command marks every unit of the pool
//!   ready to leave (`Held::leaving`); "Recruits already reserved by a queued training order are not ejected and
//!   remain committed to that order" (slice M5-F2). A unit that enters meanwhile stays: it wasn't stored when the
//!   command came.
//! - **Leaving** ([`Sim::step_holding`]; design change 0348, `factions/cults/structures/README.md > Choosing Recruits,
//!   Release and Eject All`: "units sharing the exit doorway come out **in the order they are ready**"): the building
//!   lets its units ready to leave out through its exit doorway, one at a time, each "only once the doorway is free"
//!   (`core/entities.md > Structure Type > Exit Doorway` 3), so a stream is as fast as each walks clear of the
//!   doorway. Ejected and released units come out as they went in, "unchanged", a trained one as its new type
//!   (`Sim::train`), and each goes to the building's RallyPoint ("ejected and released units walk **to the
//!   RallyPoint**, like trained units"), or straight out of the exit side without one (`core/entities.md > Structure
//!   Type > Exit Doorway` 2).
//! - **At once** (slice M5-N2; `factions/cults/structures/garrison.md > Placeholder stats > Leaving`: "units leave
//!   **through the door**; on **Eject All** they all come out **at once**, ignoring collision with each other until
//!   they are clear of the footprint"): a building with one door (`ExitRule::AtOnce`, the Garrison's) lets every unit
//!   ready to leave out together, once its doorway is free ([`Sim::let_all_out`]).

mod evacuation;

use std::collections::BTreeMap;

use glam::Vec2;

use super::Sim;
use super::gate::Checked;
use crate::command::HoldCommand;
use crate::defs::{ExitRule, ObjectKind};
use crate::events::SimEvent;
use crate::holding;
use crate::ids::{ObjectId, PlayerId};
use crate::network::{Held, Place};
use crate::production::{Door, ExitDoor, Production, QueueEntry, Rally};
use crate::sim::Reject;
use crate::units::Tick;

impl Sim {
    /// The units inside `building` (`network::Place::Building`), longest inside first: by `Held::since`, ties by id.
    /// Those reserved, in training, or ready to leave and not out yet are among them. None for anything but a building
    /// that holds units.
    pub fn held_in(&self, building: ObjectId) -> Vec<&Held> {
        let mut inside: Vec<&Held> = (self.held.values())
            .filter(|held| held.place == Place::Building(building))
            .collect();
        inside.sort_by_key(|held| (held.since, held.object.id));
        inside
    }

    /// `building`'s pool, longest inside first: the Armory's "StoredRecruits" (`factions/cults/structures/armory.md >
    /// ArmoryInstanceState`), which Eject All empties and the orders draw on: the units inside that no entry of its
    /// queue reserves (slice M5-F2, `Production::reserved`: an order "reserves one Recruit from StoredRecruits
    /// (removing it from the internal pool for the duration it sits in the queue)",
    /// `factions/cults/structures/armory.md > Training`) and that aren't on their way out (`Held::leaving`). None for
    /// anything but a building that holds units.
    pub fn pool(&self, building: ObjectId) -> Vec<&Held> {
        let production = self.objects.get(&building).and_then(|o| o.production.as_ref());
        let reserved = |unit: ObjectId| production.is_some_and(|production| production.reserves(unit));
        (self.held_in(building).into_iter())
            .filter(|held| held.leaving.is_none() && !reserved(held.object.id))
            .collect()
    }

    /// How many more units `building` takes in now: its capacity (`HoldsDef::capacity`: "The Armory can hold up to 10
    /// Recruits in its internal pool at a time", `factions/cults/structures/armory.md > InternalRecruitCapacity - 10`)
    /// less every unit inside, those its orders reserved and the one in training included (design change 0347 point
    /// 5: "Capacity counts reserved units: the queue can never exceed the capacity"). 0 for anything but a building
    /// that holds units.
    pub fn room(&self, building: ObjectId) -> u32 {
        let Some(holds) = self.objects.get(&building).and_then(holding::holds) else {
            return 0;
        };
        let inside = u32::try_from(self.held_in(building).len()).unwrap_or(u32::MAX);
        holds.capacity.saturating_sub(inside)
    }

    /// The gate for `Command::Holding`: the player's buildings among `objects` that hold units take `command`
    /// (`systems/controls/selection.md > SelectionGroup > No smart-casting`: every one of the selection), the
    /// player's other objects ignore it ([`Reject::NoHolders`] if none is left). **Eject All**: "Only available if
    /// StoredRecruits is not empty" (`factions/cults/structures/armory.md > ObjectInterfaceState[Armory]`): the
    /// buildings whose pool has a unit take it, [`Reject::NothingInside`] if none has: units reserved by an order, or
    /// on their way out already, are none of the pool's.
    pub(in crate::sim) fn check_holding(
        &self,
        player: PlayerId,
        objects: &[ObjectId],
        command: HoldCommand,
    ) -> Result<Checked, Reject> {
        let buildings = self.own_units(player, objects, |o| holding::holds(o).is_some())?;
        if buildings.is_empty() {
            return Err(Reject::NoHolders);
        }
        match command {
            HoldCommand::EjectAll => {
                let stored: Vec<ObjectId> = (buildings.into_iter())
                    .filter(|&building| !self.pool(building).is_empty())
                    .collect();
                if stored.is_empty() {
                    return Err(Reject::NothingInside);
                }
                Ok(Checked::Holding {
                    buildings: stored,
                    command,
                })
            }
        }
    }

    /// `command` to `buildings`, which passed the gate ([`Sim::check_holding`]). Eject All marks every unit of each
    /// pool ready to leave from this tick (module docs); [`Sim::step_holding`] lets them out.
    pub(in crate::sim) fn apply_holding(&mut self, buildings: &[ObjectId], command: HoldCommand) {
        match command {
            HoldCommand::EjectAll => {
                let tick = self.tick;
                for &building in buildings {
                    let pool: Vec<ObjectId> = self.pool(building).iter().map(|held| held.object.id).collect();
                    for unit in pool {
                        if let Some(held) = self.held.get_mut(&unit) {
                            held.leaving = Some(tick);
                        }
                    }
                }
            }
        }
    }

    /// The buildings' step of the economy phase (`sim/phase.rs`): each building with units ready to leave inside
    /// (`Held::leaving`: ejected, or the unit of a finished entry, `Sim::ready_to_leave`), in id order, lets the one
    /// ready first out into its exit doorway if the doorway is free ([`Sim::let_out`]): "in the order they are ready"
    /// (design change 0348), ties by the longest inside, then by id; a building with one door lets them all out at
    /// once ([`Sim::let_all_out`]). After the producers, whose finished entries make their units ready in the same
    /// tick, and after 5d, so that a unit never moves in the tick it appears; before 5f, so that it is seen in it.
    pub(in crate::sim) fn step_holding(&mut self) {
        if self.held.is_empty() {
            return;
        }
        let mut first: BTreeMap<ObjectId, (Tick, Tick, ObjectId)> = BTreeMap::new();
        for held in self.held.values() {
            if let (Place::Building(building), Some(ready)) = (held.place, held.leaving) {
                let key = (ready, held.since, held.object.id);
                first.entry(building).and_modify(|k| *k = (*k).min(key)).or_insert(key);
            }
        }
        for (building, (.., unit)) in first {
            // A building with one door produces nothing (`defs_are_consistent`): its exit is its holding's.
            match self.objects.get(&building).and_then(holding::holds) {
                Some(holds) if matches!(holds.exit, ExitRule::AtOnce(_)) => self.let_all_out(building),
                _ => self.let_out(building, unit),
            }
        }
    }

    /// (M5-N2) Every unit ready to leave `building`, a building with one door (`ExitRule::AtOnce`, the Garrison's),
    /// comes out at once (module docs), as it is, each with a `SimEvent::LeftBuilding`, in the order of the pool
    /// (`Sim::held_in`): once the doorway is free, each at the spot the doorway has for it, as one unit let out alone
    /// would be ([`Sim::release_held`]), all spots found before any of them appears, so that they don't see each
    /// other. They stand there together, overlapping, and pass through each other as they walk straight out of the
    /// door (`systems/units.md > UnitCollision > Units that already overlap`: they ignore each other until
    /// separated; `core/entities.md > Structure Type > Exit Doorway` 2, with no rally point: the Garrison has none),
    /// idle; out of the footprint, they part as idle units that overlap do (`movement::separate`). A unit the doorway
    /// has no spot for stays inside, ready, until the doorway is free again. **Reading of the design**: "through the
    /// door" is the one spot an exit doorway puts a unit at, "half inside the building, at the middle of the exit side"
    /// (`core/entities.md > Structure Type > Exit Doorway` 1), for all of them.
    fn let_all_out(&mut self, building: ObjectId) {
        let ready: Vec<ObjectId> = (self.held_in(building).into_iter())
            .filter(|held| held.leaving.is_some())
            .map(|held| held.object.id)
            .collect();
        let spots: Vec<(ObjectId, Spot)> = (ready.into_iter())
            .filter_map(|unit| Some((unit, self.held_spot(building, unit, None, None)?)))
            .collect();
        for (unit, spot) in spots {
            if self.come_out(building, unit, spot) {
                self.events.push(SimEvent::LeftBuilding { unit, building });
            }
        }
    }

    /// Unit `unit`, ready to leave `building`, comes out if the doorway lets it ([`Sim::release_held`]), with what it
    /// leaves for:
    /// - the first unit the front entry of the building's queue reserved, that entry finished (`sim/producers/pool.rs`):
    ///   a training's comes out as the unit it trains (`Sim::train`, a `SimEvent::UnitProduced`), an augmentation
    ///   order's augmented (`Sim::augment`, slice M5-N1b; a `SimEvent::UnitProduced` too), a Release's as it is
    ///   (`SimEvent::LeftBuilding`), and the entry leaves the queue (`Production::release`);
    /// - any other, ejected: as it is (`SimEvent::LeftBuilding`).
    ///
    /// Then it goes to the building's RallyPoint, with the order the rally gives (`Sim::rally_order`), as a producer's
    /// unit does (design change 0348, `factions/cults/structures/README.md > Choosing Recruits, Release and Eject All`:
    /// "ejected and released units walk **to the RallyPoint**, like trained units").
    fn let_out(&mut self, building: ObjectId, unit: ObjectId) {
        let production = self.objects.get(&building).and_then(|o| o.production.as_ref());
        let rally = production.and_then(|production| production.rally);
        let entry = (production.filter(|production| production.reserved_by_current().first() == Some(&unit)))
            .and_then(Production::current);
        if !self.release_held(building, unit, entry.and_then(QueueEntry::unit), rally) {
            return;
        }
        let released = entry.and_then(|_| {
            let production = self.objects.get_mut(&building)?.production.as_mut()?;
            production.release()
        });
        match released {
            Some((entry @ (QueueEntry::Unit(_) | QueueEntry::Augment { .. }), reserved)) => {
                self.complete(unit, entry, reserved.get(1..).unwrap_or_default());
                self.events.push(SimEvent::UnitProduced {
                    producer: building,
                    unit,
                });
            }
            Some((QueueEntry::Release, _)) | None => self.events.push(SimEvent::LeftBuilding { unit, building }),
        }
        if let Some(kind) = rally.and_then(|rally| self.rally_order(unit, rally)) {
            self.give_order(&[unit], kind, false);
        }
    }

    /// Unit `unit`, held in `building`, comes out into the building's exit doorway
    /// (`core/entities.md > Structure Type > Exit Doorway` 1: it "appears half inside the building, at the middle of
    /// the exit side"), facing out, as a producer's finished unit does (`Sim::release_unit`): only once the doorway is
    /// free (3: "the next unit can only appear once the doorway is free"; a unit the building let out, `Mover::exit`,
    /// still stands in it), at the spot the doorway has room for (`Sim::doorway_spot`), a spot for a unit of kind
    /// `becomes` if it comes out as one (a trained unit, slice M5-F2), else for its own. Of several exit sides, the
    /// first door in their order toward `rally` that is free and has a spot (`production::Doorway::order`, slice
    /// M5-C's rule). It is the object it was, with no order: the caller gives it the rally's ([`Sim::let_out`]), and
    /// without one it walks straight out (2: "or straight out of the exit side if there is none"). Returns whether
    /// it came out.
    pub(in crate::sim) fn release_held(
        &mut self,
        building: ObjectId,
        unit: ObjectId,
        becomes: Option<ObjectKind>,
        rally: Option<Rally>,
    ) -> bool {
        self.held_spot(building, unit, becomes, rally)
            .is_some_and(|spot| self.come_out(building, unit, spot))
    }

    /// Where unit `unit`, held in `building`, would come out now ([`Sim::release_held`]): the spot of the first door
    /// in their order toward `rally` that is free and has one, for a unit of kind `becomes` if it comes out as one.
    /// `None` while there is none: it waits inside.
    fn held_spot(
        &self,
        building: ObjectId,
        unit: ObjectId,
        becomes: Option<ObjectKind>,
        rally: Option<Rally>,
    ) -> Option<Spot> {
        let o = self.objects.get(&building)?;
        let (Some(ExitDoor::Side(doorway)), centre) = (ExitDoor::of(o), o.pos) else {
            return None;
        };
        let held = (self.held.get(&unit)).filter(|held| held.place == Place::Building(building))?;
        let def = becomes.unwrap_or(held.object.kind).def();
        let (Some(unit_def), Some(half)) = (&def.unit, def.half_extents()) else {
            return None;
        };
        // The units this building let out that are still in its doorway, each keeping its door occupied.
        let released: Vec<Vec2> = (self.objects.values())
            .filter(|o| o.mover.as_ref().is_some_and(|mover| mover.exit == Some(building)))
            .map(|o| o.pos)
            .collect();
        let free = |door: &Door| released.iter().all(|&pos| doorway.door_at(pos) != *door);
        let toward = rally.map(|rally| rally.point() - centre);
        let mut doors = doorway.order(toward).filter(free);
        let (door, pos) = doors.find_map(|door| Some((door, self.doorway_spot(&door, unit_def, half)?)))?;
        Some(Spot {
            pos,
            outward: door.outward(),
            in_doorway: doorway.holds(pos, half),
        })
    }

    /// Unit `unit`, held in `building`, comes out at `spot`, facing out of its door, in the doorway's exception while it
    /// overlaps the footprint (`Mover::exit`). Returns whether it came out.
    fn come_out(&mut self, building: ObjectId, unit: ObjectId, spot: Spot) -> bool {
        if !self.put_on_map(unit, spot.pos, spot.outward) {
            return false;
        }
        if let Some(mover) = self.objects.get_mut(&unit).and_then(|o| o.mover.as_mut()) {
            mover.exit = spot.in_doorway.then_some(building);
        }
        true
    }
}

/// Where a held unit comes out of its building's exit doorway ([`Sim::release_held`]).
#[derive(Copy, Clone, Debug, PartialEq)]
struct Spot {
    pos: Vec2,
    /// Out of its door, which it faces.
    outward: Vec2,
    /// Its box overlaps the footprint there: it is in the doorway.
    in_doorway: bool,
}

#[cfg(test)]
mod tests;
