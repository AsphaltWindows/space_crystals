//! Buildings that hold units in a game (`crate::holding`, plan decision T60, slice M5-F1): their pools as the sim shows
//! them, the gate's rule for their commands, Eject All, and the step of the economy phase that lets the ejected units
//! out. The way in is the Enter order's (`sim/network/entries.rs`); the way out when the building is destroyed is
//! `holding/evacuation.rs`'s.
//!
//! **Eject All** (`factions/cults/structures/armory.md > Eject All`: "Immediate command that ejects all stored Recruits
//! from the exit side in a rapid stream, one after another"): the command marks every unit of the pool
//! (`Held::ejected`), and the building lets them out through its exit doorway, longest inside first, one at a time:
//! each "only once the doorway is free" (`core/entities.md > Structure Type > Exit Doorway` 3), so the stream is as
//! fast as each walks clear of the doorway. A unit that enters meanwhile stays: it wasn't stored when the command came.
//! An ejected unit comes out as it went in, "unchanged", with no order: it walks straight out of the exit side and
//! stops (`core/entities.md > Structure Type > Exit Doorway` 2: "straight out of the exit side if there is none"; the
//! RallyPoint is slice M5-F2's).

mod evacuation;

use std::collections::BTreeMap;

use super::Sim;
use super::gate::Checked;
use crate::command::HoldCommand;
use crate::events::SimEvent;
use crate::holding;
use crate::ids::{ObjectId, PlayerId};
use crate::network::{Held, Place};
use crate::production::ExitDoor;
use crate::sim::Reject;
use crate::units::Tick;

impl Sim {
    /// The units inside `building` (`network::Place::Building`), longest inside first: by `Held::since`, ties by id.
    /// Those ejected and not out yet are among them. None for anything but a building that holds units.
    pub fn held_in(&self, building: ObjectId) -> Vec<&Held> {
        let mut inside: Vec<&Held> = (self.held.values())
            .filter(|held| held.place == Place::Building(building))
            .collect();
        inside.sort_by_key(|held| (held.since, held.object.id));
        inside
    }

    /// `building`'s pool, longest inside first: the Armory's "StoredRecruits" (`factions/cults/structures/armory.md >
    /// ArmoryInstanceState`), which Eject All empties and whose size the capacity bounds ([`Sim::room`]). Every unit
    /// inside, for now: slice M5-F2's training orders will reserve Recruits, "removing it from the internal pool for
    /// the duration it sits in the queue" (`factions/cults/structures/armory.md > Training`), and leave them out of it
    /// here.
    pub fn pool(&self, building: ObjectId) -> Vec<&Held> {
        self.held_in(building)
    }

    /// How many more units `building` takes in now: its capacity (`HoldsDef::capacity`: "The Armory can hold up to 10
    /// Recruits in its internal pool at a time", `factions/cults/structures/armory.md > InternalRecruitCapacity - 10`)
    /// less its pool (Placeholder (Q0104 point 5): "The pool only"). 0 for anything but a building that holds units.
    pub fn room(&self, building: ObjectId) -> u32 {
        let Some(holds) = self.objects.get(&building).and_then(holding::holds) else {
            return 0;
        };
        let pool = u32::try_from(self.pool(building).len()).unwrap_or(u32::MAX);
        holds.capacity.saturating_sub(pool)
    }

    /// The gate for `Command::Holding`: the player's buildings among `objects` that hold units take `command`
    /// (`systems/controls/selection_and_hud.md > SelectionGroup > No smart-casting`: every one of the selection), the
    /// player's other objects ignore it ([`Reject::NoHolders`] if none is left). **Eject All**: "Only available if
    /// StoredRecruits is not empty" (`factions/cults/structures/armory.md > ObjectInterfaceState[Armory]`): the
    /// buildings whose pool has a unit take it, [`Reject::NothingInside`] if none has.
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
    /// pool ejected (module docs); [`Sim::step_holding`] lets them out.
    pub(in crate::sim) fn apply_holding(&mut self, buildings: &[ObjectId], command: HoldCommand) {
        match command {
            HoldCommand::EjectAll => {
                for &building in buildings {
                    let pool: Vec<ObjectId> = self.pool(building).iter().map(|held| held.object.id).collect();
                    for unit in pool {
                        if let Some(held) = self.held.get_mut(&unit) {
                            held.ejected = true;
                        }
                    }
                }
            }
        }
    }

    /// The buildings' step of the economy phase (`sim/phase.rs`): each building with ejected units inside, in id
    /// order, lets the one longest inside out into its exit doorway if the doorway is free ([`Sim::release_held`]).
    /// After the producers, whose finished units go first through a doorway they share (slice M5-F2), and after 5d, so
    /// that a unit never moves in the tick it appears; before 5f, so that it is seen in it.
    pub(in crate::sim) fn step_holding(&mut self) {
        if self.held.is_empty() {
            return;
        }
        let mut first: BTreeMap<ObjectId, (Tick, ObjectId)> = BTreeMap::new();
        for held in self.held.values().filter(|held| held.ejected) {
            if let Place::Building(building) = held.place {
                let key = (held.since, held.object.id);
                first.entry(building).and_modify(|k| *k = (*k).min(key)).or_insert(key);
            }
        }
        for (building, (_, unit)) in first {
            self.release_held(building, unit);
        }
    }

    /// Unit `unit`, held in `building`, comes out into the building's exit doorway
    /// (`core/entities.md > Structure Type > Exit Doorway` 1: it "appears half inside the building, at the middle of
    /// the exit side"), facing out, as a producer's finished unit does (`Sim::release_unit`): only once the doorway is
    /// free (3: "the next unit can only appear once the doorway is free"; a unit the building let out, `Mover::exit`,
    /// still stands in it), at the spot the doorway has room for (`Sim::doorway_spot`). It is the object it was, with
    /// no order, and walks straight out (2: "or straight out of the exit side if there is none"), with a
    /// `SimEvent::LeftBuilding`. Returns whether it came out. Slice M5-F2 releases its trained units by it too.
    pub(in crate::sim) fn release_held(&mut self, building: ObjectId, unit: ObjectId) -> bool {
        let Some(ExitDoor::Side(doorway)) = self.objects.get(&building).and_then(ExitDoor::of) else {
            return false;
        };
        let occupied =
            (self.objects.values()).any(|o| o.mover.as_ref().is_some_and(|mover| mover.exit == Some(building)));
        if occupied {
            return false;
        }
        let Some(held) = (self.held.get(&unit)).filter(|held| held.place == Place::Building(building)) else {
            return false;
        };
        let def = held.object.kind.def();
        let (Some(unit_def), Some(half)) = (&def.unit, def.half_extents()) else {
            return false;
        };
        let door = doorway.door();
        let Some(pos) = self.doorway_spot(&door, unit_def, half) else {
            return false;
        };
        if !self.put_on_map(unit, pos, door.outward()) {
            return false;
        }
        if let Some(mover) = self.objects.get_mut(&unit).and_then(|o| o.mover.as_mut()) {
            mover.exit = doorway.holds(pos, half).then_some(building);
        }
        self.events.push(SimEvent::LeftBuilding { unit, building });
        true
    }
}

#[cfg(test)]
mod tests;
