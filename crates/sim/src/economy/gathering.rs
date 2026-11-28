//! Gathering — `factions/colonists/units/drone.md > Gathering`, `factions/colonists/units/drone.md > Patch Occupancy
//! (one Drone per patch)`, `factions/colonists/units/drone.md > Assigned Drop-off`; `factions/colonists/economy.md >
//! Egress Node > Drop Off` (slices M3-H and M3-H3, plan decision T17).
//!
//! "Every Drone job is pick up at X, deliver to Y": a worker walks to a Space Crystals patch, mines it for
//! MiningDuration, carries up to CarryCapacity to its AssignedDropOff, unloads for DropOffDuration, and goes again.
//! "Resources only enter the player's resource pool when delivered to an Egress Node."
//!
//! - **The capability** ([`Worker`], `Object::worker`): the load and the AssignedDropOff. A unit type with a
//!   `WorkerDef` (`UnitDef::worker`: the Drone) has one. The patch it gathers is its Gather order's.
//! - **The job** is an order, `OrderKind::Gather(patch)` or `OrderKind::DropOff(node)`, and its behaviour
//!   (`orders::gather`, phase 5a) owns the legs, the claims and every decision; its state
//!   (`orders::state::Gather`) holds the phase and the frames of timed work done. **Deviation from the plan's
//!   sketch**, which put the timed work into the `Worker`: it is part of the order's state, so that an order that
//!   ends for any reason takes its unfinished work with it (design change 0130, `systems/mechanics/gathering.md >
//!   Interrupted mining`: "collects nothing, and the patch is freed immediately") and occupancy can't outlive the order
//!   that holds it.
//! - **The load** leaves the worker only at a drop-off, here, or with the worker itself when it deploys (design
//!   change 0129, `systems/mechanics/gathering.md > Other orders keep the load`; 0114): no order clears it.
//! - **The timed work** runs here, in the economy phase ([`step`], after the plates' mining, plan decision T5): a
//!   Mining phase counts its frames and, at MiningDuration, takes `min(CarryCapacity, remaining)` from the patch
//!   (design change 0138, `systems/mechanics/gathering.md > Last load of a patch`: the full 32 frames, and then
//!   "what is left"); a DroppingOff phase counts its frames and, at DropOffDuration, credits the owner's bank
//!   (`SimEvent::ResourcesDelivered`). All bank changes stay in the economy phase.
//! - **Timing**, by the Director's convention for recipes (`docs/architecture/economy.md > Economy > Recipe timing`): the frame
//!   in which the behaviour starts the work counts. A worker that reaches its patch in phase 5a of tick `t` holds
//!   its load at the end of tick `t + 31` and sets off in `t + 32`, when the patch is free again; one that reaches
//!   its drop-off in tick `t` is paid at the end of `t + 1` and sets off in `t + 2`.
//! - **Depletion** (`core/map.md > SpaceCrystalsPatch`: "When a patch is fully depleted, it disappears from the
//!   map"): [`step`] returns the patches its loads emptied, and `Sim::step` takes them out through the removal
//!   path, which opens their tiles (design change 0038) and leaves the workers that gathered them heading for where
//!   they were (`Sim::keep_gatherers_on_spots`, design change 0134). A patch under a GDO plate stays
//!   (`factions/gdo/economy.md > ExtractionPlate`); for workers it is gone already (design change 0137).
//!
//! **Trip times with DragMovement** (slice M3-G, plan decision T25; pinned by `tests/gathering/`): with the patch 2
//! grid units from the Egress Node, edge to edge (design change 0107), one Drone alone delivers a load every
//! **73 ticks** (4.56 s) with its numbers of design change 0205 (slice M3-K3): 31 of mining after the tick it
//! begins, 2 of unloading and two legs of 20 ticks each. A leg is about 85 su and not the 128 of the 2 grid units:
//! the Drone's own 36 su come off, and it begins its work when it comes within reach, 8 su short, while it brakes
//! off the last of its speed. It never reaches its top speed on a leg (it peaks near 7 su a tick). That is 1.75
//! Space Crystals per second for one Drone; the design's table has 1.5 for each of the four starting Drones (6.1 in
//! all). Before 0205 a trip took 81 ticks (legs of 24, peaking near 6 su a tick: 1.58 per second), and on the
//! stand-in mover the Drone had before (top speed at once) 60 ticks.
//!
//! **Crowds**: `tests/gathering_soak.rs`, and the note on collision in `orders::gather`.
//!
//! **Supplies** (slice M4-I; `factions/syndicate/units/agent.md > Gathering > Supplies`, design change 0256): a
//! worker whose type gathers them (`WorkerDef::supplies`: the Agent) picks one up at a Supply Delivery Station
//! (`super::stations`): its timed work there is a pick-up, PickUpDuration, which takes the Supply from the station as
//! it ends, and its delivery pays the Supplies bank, DropOffDuration after it began at the Tunnel's Supplies hatch.
//! What a load is of is the worker's [`Worker::resource`]. **Switching resources** (0256 point 4): "an Agent carrying
//! Space Crystals that is ordered to gather Supplies goes straight to the station; its crystals are lost when it picks
//! up the Supply. (Assumed the same the other way: a carried Supply is lost when it mines crystals.)" The old load
//! goes when the new pick-up or mining ends, with the new load: one interrupted before that keeps it (design change
//! 0292 point 3: "the old load is lost when the new pick-up or mining ends, both ways").

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::stations::is_station;
use crate::defs::{ObjectKind, Resource, WorkerDef};
use crate::events::SimEvent;
use crate::hash::{StateHash, StateHasher};
use crate::ids::ObjectId;
use crate::object::{Object, ResourceState};
use crate::orders::state::{Behaviour, GatherPhase};
use crate::orders::{OrderKind, Orders};
use crate::player::Player;

/// A worker's own state (module docs): what it keeps whatever its orders are.
///
/// It no longer keeps the patch it gathers (slice M3-H had it, for question Q0037 point 1's proposal: back to the
/// patch after a Drop Off order). Design change 0127 dropped that, and the patch a Gather goes back to after a
/// delivery, "the patch where it last successfully mined" (0132), is the one its order names. A save that has
/// the field still loads: serde passes over it.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Worker {
    /// The Space Crystals it carries, at most its CarryCapacity. It keeps them through other orders and delivers
    /// them at its next drop-off (design change 0129).
    pub load: u32,
    /// `factions/colonists/units/drone.md > Assigned Drop-off`: "the Egress Node or refinery it delivers its current
    /// and future loads to". `None` until its first Gather assigns one, and again when that one is gone, but for a
    /// worker on its way back to it, which takes the nearest other at once (design change 0170,
    /// `Sim::reassign_drop_offs`).
    pub drop_off: Option<ObjectId>,
    /// What its load is of (slice M4-I): Space Crystals, or Supplies for a worker of a type that gathers them
    /// (module docs). Space Crystals while it carries nothing. Optional in files and left out of them while Space
    /// Crystals, so that states saved before it existed load, and hashed only while Supplies.
    #[serde(default, skip_serializing_if = "is_crystals")]
    pub resource: Resource,
}

/// Whether `resource` is Space Crystals, which a worker's file leaves out ([`Worker::resource`]).
fn is_crystals(resource: &Resource) -> bool {
    *resource == Resource::Crystals
}

impl Worker {
    /// The removal path (`Sim::remove_objects`): the worker drops its references to the objects that are gone.
    pub(crate) fn forget(&mut self, is_gone: &impl Fn(ObjectId) -> bool) {
        if self.drop_off.is_some_and(is_gone) {
            self.drop_off = None;
        }
    }
}

impl StateHash for Worker {
    fn hash(&self, h: &mut StateHasher) {
        let Worker {
            load,
            drop_off,
            resource,
        } = self;
        load.hash(h);
        drop_off.hash(h);
        // (M4-I) Only while it carries Supplies, under a tag of its own: the states of games in which no worker
        // carries any hash as they did before the field existed, so the goldens stand.
        match resource {
            Resource::Crystals => {}
            Resource::Supplies => h.tag(1),
        }
    }
}

/// How a unit of `kind` gathers, if it does (`UnitDef::worker`).
pub fn worker_def(kind: ObjectKind) -> Option<WorkerDef> {
    kind.def().unit.as_ref()?.worker
}

/// Whether workers deliver to a structure of `kind` (`StructureDef::drop_off`: the Egress Node, and the Tunnel at its
/// hatches, `super::drop_off`).
pub fn is_drop_off(kind: ObjectKind) -> bool {
    super::drop_off::rule(kind).is_some()
}

/// Whether `id` is a Space Crystals patch that workers can gather: one with no mining structure on it (question
/// Q0037 point 11: "Drones can't gather" a patch under a GDO Extraction Plate).
pub fn gatherable(objects: &BTreeMap<ObjectId, Object>, id: ObjectId) -> bool {
    let is_patch = objects
        .get(&id)
        .is_some_and(|o| matches!(o.resource, Some(ResourceState::CrystalPatch { .. })));
    is_patch && !covered(objects, id)
}

/// Whether a mining structure stands on `patch`.
fn covered(objects: &BTreeMap<ObjectId, Object>, patch: ObjectId) -> bool {
    objects
        .values()
        .any(|o| o.extractor.is_some_and(|extractor| extractor.patch == patch))
}

/// The gathering job of `orders`' current order, if it has one: the order's kind and its state.
fn job(orders: &mut Orders) -> Option<(OrderKind, &mut crate::orders::state::Gather)> {
    let active = orders.current.as_mut()?;
    match &mut active.state {
        Behaviour::Gather(gather) => Some((active.order.kind, gather)),
        _ => None,
    }
}

/// Pays `amount` of `resource` into `player`'s bank. Income saturates (`Player::credit_crystals`). A Supply
/// Chopper's drop-off pays by it too (`super::supply_run`).
pub(super) fn credit(player: &mut Player, resource: Resource, amount: u32) {
    match resource {
        Resource::Crystals => player.credit_crystals(amount),
        Resource::Supplies => player.supplies = player.supplies.saturating_add(amount),
    }
}

/// The workers' timed work (module docs), for the workers from before this tick, in id order: the frames first,
/// then the loads taken from the patches and the stations. Returns the patches that the loads emptied and that leave
/// the map, in id order.
pub(super) fn step(
    objects: &mut BTreeMap<ObjectId, Object>,
    players: &mut [Player],
    first_new: ObjectId,
    events: &mut Vec<SimEvent>,
) -> Vec<ObjectId> {
    // (worker, patch, CarryCapacity) of the mining that ended this tick, and (worker, station, CarryCapacity) of the
    // pick-ups.
    let mut mined: Vec<(ObjectId, ObjectId, u32)> = Vec::new();
    let mut picked: Vec<(ObjectId, ObjectId, u32)> = Vec::new();
    // The timed work of the workers of a type that gathers Supplies too, as (worker, patch or station, frames done):
    // how long it lasts depends on where it is done, a station's being a pick-up of Supplies (slice M4-I), which the
    // pass over the workers can't look up. It is told after the pass, from their targets alone, so that no tick looks
    // for the stations among all the objects.
    let mut either: Vec<(ObjectId, ObjectId, u32)> = Vec::new();
    for o in objects.range_mut(..first_new).map(|(_, o)| o) {
        let (Some(worker), Some(orders)) = (&mut o.worker, &mut o.orders) else {
            continue;
        };
        let (Some(def), Some((kind, gather))) = (worker_def(o.kind), job(orders)) else {
            continue;
        };
        match &mut gather.phase {
            // `factions/colonists/units/drone.md > Gathering`: MiningDuration; `factions/syndicate/units/agent.md >
            // Gathering > Supplies`: PickUpDuration at a station.
            GatherPhase::Mining { done } => {
                let source = match kind {
                    OrderKind::Gather(source) => Some(source),
                    _ => None,
                };
                *done = done.saturating_add(1);
                if def.supplies.is_some()
                    && let Some(source) = source
                {
                    either.push((o.id, source, *done));
                } else if *done >= def.crystals.pick_up_ticks {
                    if let Some(source) = source {
                        mined.push((o.id, source, def.crystals.carry_capacity));
                    }
                    gather.enter(GatherPhase::ToDropOff);
                }
            }
            // `factions/colonists/units/drone.md > Gathering`: DropOffDuration, of the resource it carries. A drop-off
            // that went away in the middle takes nothing: the worker looks for another (`orders::gather`).
            GatherPhase::DroppingOff { done } => match gather.node {
                None => gather.enter(GatherPhase::ToDropOff),
                Some(node) => {
                    let resource = worker.resource;
                    *done = done.saturating_add(1);
                    if *done >= def.load(resource).unwrap_or(def.crystals).drop_off_ticks {
                        // `factions/colonists/economy.md > Egress Node > Drop Off`: the load enters the pool.
                        let amount = std::mem::take(&mut worker.load);
                        worker.resource = Resource::Crystals;
                        if let Some(player) = players.iter_mut().find(|p| Some(p.id) == o.owner) {
                            credit(player, resource, amount);
                        }
                        events.push(SimEvent::ResourcesDelivered {
                            worker: o.id,
                            drop_off: node,
                            amount,
                            resource,
                        });
                        gather.enter(GatherPhase::Delivered);
                    }
                }
            },
            GatherPhase::ToPatch
            | GatherPhase::Waiting { .. }
            | GatherPhase::ToDropOff
            | GatherPhase::AtHatch { .. }
            | GatherPhase::Delivered => {}
        }
    }
    for (id, source, done) in either {
        let at_station = objects.get(&source).is_some_and(|o| is_station(o.kind));
        let Some(o) = objects.get_mut(&id) else {
            continue;
        };
        let resource = if at_station {
            Resource::Supplies
        } else {
            Resource::Crystals
        };
        let Some(load) = worker_def(o.kind).map(|def| def.load(resource).unwrap_or(def.crystals)) else {
            continue;
        };
        if done < load.pick_up_ticks {
            continue;
        }
        if let Some((_, gather)) = o.orders.as_mut().and_then(job) {
            gather.enter(GatherPhase::ToDropOff);
        }
        let ended = (id, source, load.carry_capacity);
        if at_station {
            picked.push(ended);
        } else {
            mined.push(ended);
        }
    }
    // In id order, as the pass found them: of two workers that take a patch's last Space Crystals in one tick, the
    // lower id gets them (question Q0037 point 13).
    mined.sort_unstable_by_key(|&(id, ..)| id);
    let mut depleted = Vec::new();
    for (id, patch, capacity) in mined {
        let Some(ResourceState::CrystalPatch { remaining }) = objects.get_mut(&patch).and_then(|o| o.resource.as_mut())
        else {
            continue;
        };
        // Question Q0037 point 13: the last load of a nearly empty patch is what is left.
        let amount = (*remaining).min(capacity);
        *remaining -= amount;
        let emptied = *remaining == 0;
        // A Supply it carried is lost as the mining ends (0256 point 4; design change 0292 point 3).
        if let Some(worker) = objects.get_mut(&id).and_then(|o| o.worker.as_mut()) {
            worker.load = amount;
            worker.resource = Resource::Crystals;
        }
        // `core/map.md > SpaceCrystalsPatch`: a depleted patch disappears, unless a GDO plate stands on it.
        if emptied && !covered(objects, patch) {
            depleted.push(patch);
        }
    }
    // 0256 point 2: "A Supply leaves the station when the pick-up ends"; the Space Crystals the worker carried are
    // lost as it takes the Supply (point 4; design change 0292 point 3). Point 3: the pick-up that empties a station
    // notifies the owners of the workers there.
    for (id, station, capacity) in picked {
        let Some((amount, emptied)) = super::stations::take(objects, station, capacity) else {
            continue;
        };
        if amount > 0
            && let Some(worker) = objects.get_mut(&id).and_then(|o| o.worker.as_mut())
        {
            worker.load = amount;
            worker.resource = Resource::Supplies;
        }
        if emptied {
            super::stations::notify_empty(objects, station, events);
        }
    }
    depleted.sort_unstable();
    depleted.dedup();
    depleted
}

#[cfg(test)]
mod tests;
