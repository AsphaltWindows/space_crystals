//! Gathering — `factions/colonists/units.md > Drone > Gathering`, `> Patch Occupancy (one Drone per patch)`, `>
//! Assigned Drop-off`; `factions/colonists/economy.md > Egress Node > Drop Off` (slices M3-H and M3-H3, plan
//! decision T17).
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
//!   ends for any reason takes its unfinished work with it (design change 0130, `> Interrupted mining`: "collects
//!   nothing, and the patch is freed immediately") and occupancy can't outlive the order that holds it.
//! - **The load** leaves the worker only at a drop-off, here, or with the worker itself when it deploys (design
//!   change 0129, `> Other orders keep the load`; 0114): no order clears it.
//! - **The timed work** runs here, in the economy phase ([`step`], after the plates' mining, plan decision T5): a
//!   Mining phase counts its frames and, at MiningDuration, takes `min(CarryCapacity, remaining)` from the patch
//!   (design change 0138, `> Last load of a patch`: the full 32 frames, and then "what is left"); a DroppingOff
//!   phase counts its frames and, at DropOffDuration, credits the owner's bank (`SimEvent::ResourcesDelivered`).
//!   All bank changes stay in the economy phase.
//! - **Timing**, by the Director's convention for recipes (`ARCHITECTURE.md > Economy > Recipe timing`): the frame
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

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::defs::{ObjectKind, WorkerDef};
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
    /// `> Assigned Drop-off`: "the Egress Node or refinery it delivers its current and future loads to". `None`
    /// until its first Gather assigns one, and again when that one is gone, but for a worker on its way back to it,
    /// which takes the nearest other at once (design change 0170, `Sim::reassign_drop_offs`).
    pub drop_off: Option<ObjectId>,
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
        let Worker { load, drop_off } = self;
        load.hash(h);
        drop_off.hash(h);
    }
}

/// How a unit of `kind` gathers, if it does (`UnitDef::worker`).
pub fn worker_def(kind: ObjectKind) -> Option<WorkerDef> {
    kind.def().unit.as_ref()?.worker
}

/// Whether workers deliver to a structure of `kind` (`StructureDef::drop_off`: the Egress Node).
pub fn is_drop_off(kind: ObjectKind) -> bool {
    kind.def()
        .structure
        .as_ref()
        .is_some_and(|structure| structure.drop_off)
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

/// The workers' timed work (module docs), for the workers from before this tick, in id order: the frames first,
/// then the loads taken from the patches. Returns the patches that the loads emptied and that leave the map, in id
/// order.
pub(super) fn step(
    objects: &mut BTreeMap<ObjectId, Object>,
    players: &mut [Player],
    first_new: ObjectId,
    events: &mut Vec<SimEvent>,
) -> Vec<ObjectId> {
    // (worker, patch, CarryCapacity) of the mining that ended this tick.
    let mut mined: Vec<(ObjectId, ObjectId, u32)> = Vec::new();
    for o in objects.range_mut(..first_new).map(|(_, o)| o) {
        let (Some(worker), Some(orders)) = (&mut o.worker, &mut o.orders) else {
            continue;
        };
        let (Some(def), Some((kind, gather))) = (worker_def(o.kind), job(orders)) else {
            continue;
        };
        match &mut gather.phase {
            // `> Gathering`: MiningDuration.
            GatherPhase::Mining { done } => {
                *done = done.saturating_add(1);
                if *done >= def.mining_ticks {
                    if let OrderKind::Gather(patch) = kind {
                        mined.push((o.id, patch, def.carry_capacity));
                    }
                    gather.enter(GatherPhase::ToDropOff);
                }
            }
            // `> Gathering`: DropOffDuration. A drop-off that went away in the middle takes nothing: the worker
            // looks for another (`orders::gather`).
            GatherPhase::DroppingOff { done } => match gather.node {
                None => gather.enter(GatherPhase::ToDropOff),
                Some(node) => {
                    *done = done.saturating_add(1);
                    if *done >= def.drop_off_ticks {
                        // `factions/colonists/economy.md > Egress Node > Drop Off`: the load enters the pool.
                        let amount = std::mem::take(&mut worker.load);
                        if let Some(player) = players.iter_mut().find(|p| Some(p.id) == o.owner) {
                            player.credit_crystals(amount);
                        }
                        events.push(SimEvent::ResourcesDelivered {
                            worker: o.id,
                            drop_off: node,
                            amount,
                        });
                        gather.enter(GatherPhase::Delivered);
                    }
                }
            },
            GatherPhase::ToPatch | GatherPhase::Waiting { .. } | GatherPhase::ToDropOff | GatherPhase::Delivered => {}
        }
    }
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
        if let Some(worker) = objects.get_mut(&id).and_then(|o| o.worker.as_mut()) {
            worker.load = amount;
        }
        // `core/map.md > SpaceCrystalsPatch`: a depleted patch disappears, unless a GDO plate stands on it.
        if emptied && !covered(objects, patch) {
            depleted.push(patch);
        }
    }
    depleted.sort_unstable();
    depleted.dedup();
    depleted
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash_of(worker: &Worker) -> u64 {
        let mut h = StateHasher::new();
        worker.hash(&mut h);
        h.finish()
    }

    /// Every field of a worker is state: each one changes the hash.
    #[test]
    fn the_hash_covers_every_field() {
        let worker = Worker {
            load: 8,
            drop_off: Some(ObjectId(2)),
        };
        let changed = [
            Worker { load: 7, ..worker },
            Worker {
                drop_off: Some(ObjectId(3)),
                ..worker
            },
            Worker {
                drop_off: None,
                ..worker
            },
        ];
        for other in changed {
            assert_ne!(hash_of(&other), hash_of(&worker), "{other:?}");
        }
    }

    /// The removal path: a worker forgets the drop-off that is gone, and keeps its load. A worker saved with the
    /// patch slice M3-H kept loads as it was, without it.
    #[test]
    fn a_worker_forgets_what_is_gone() {
        let mut worker = Worker {
            load: 8,
            drop_off: Some(ObjectId(2)),
        };
        worker.forget(&|id| id == ObjectId(9));
        assert_eq!(worker.drop_off, Some(ObjectId(2)));
        worker.forget(&|id| id == ObjectId(2));
        let emptied = Worker {
            load: 8,
            ..Worker::default()
        };
        assert_eq!(worker, emptied);
        let saved: Worker = serde_json::from_str(r#"{"load":8,"drop_off":null,"patch":3}"#).unwrap();
        assert_eq!(saved, emptied);
    }

    /// `factions/colonists/units.md > Drone > Gathering`: the Drone is the one worker, and the Egress Node the one
    /// drop-off.
    #[test]
    fn the_drone_is_the_worker_and_the_egress_node_the_drop_off() {
        for kind in ObjectKind::ALL.iter().copied() {
            assert_eq!(worker_def(kind).is_some(), kind == ObjectKind::Drone, "{kind:?}");
            assert_eq!(is_drop_off(kind), kind == ObjectKind::EgressNode, "{kind:?}");
        }
        let def = worker_def(ObjectKind::Drone).expect("a worker");
        assert_eq!((def.carry_capacity, def.mining_ticks, def.drop_off_ticks), (8, 32, 2));
        assert_eq!((def.mining_reach, def.drop_off_reach), (8.0, 8.0));
        assert_eq!(
            def.search_radius, 6.0,
            "> Patch Occupancy > Search radius (design change 0123)"
        );
    }
}
