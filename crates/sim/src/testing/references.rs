//! What an object refers to, and the references the removal path must not leave behind (`Sim::remove_objects`).

use crate::combat::AttackPhase;
use crate::ids::ObjectId;
use crate::mover::MoveGoal;
use crate::object::Object;
use crate::production::Rally;
use crate::sim::Sim;

/// The objects `o` refers to, in no particular order: those its orders (current and queued) name, those its
/// behaviours engaged by themselves, its move's goal object, its attack's target and the target of its attack
/// phase, the patch a plate mines, the object a producer's rally point is on, the building whose exit doorway a
/// unit is leaving, and a worker's drop-off.
pub fn referenced_ids(o: &Object) -> Vec<ObjectId> {
    let mut ids = Vec::new();
    if let Some(orders) = &o.orders {
        ids.extend(orders.current.as_ref().and_then(|active| active.order.target()));
        ids.extend(orders.queue.iter().filter_map(|order| order.target()));
        ids.extend(orders.behaviour_targets());
    }
    if let Some(MoveGoal::Object { id, .. }) = o.mover.as_ref().map(|m| m.goal) {
        ids.push(id);
    }
    if let Some(attack) = &o.attack {
        ids.extend(attack.target);
        if let Some(AttackPhase::Aiming { target } | AttackPhase::Firing { target: Some(target) }) = attack.phase {
            ids.push(target);
        }
    }
    ids.extend(o.extractor.map(|extractor| extractor.patch));
    if let Some(Rally::Object { id, .. }) = o.production.as_ref().and_then(|production| production.rally) {
        ids.push(id);
    }
    ids.extend(o.mover.as_ref().and_then(|mover| mover.exit));
    if let Some(worker) = &o.worker {
        ids.extend(worker.drop_off);
    }
    ids
}

/// Every reference to an object that no longer exists, as `(holder, missing)` pairs ([`referenced_ids`]). The
/// removal path (`Sim::remove_objects`) leaves none behind, but for the orders that outlive their targets
/// (`orders::Order::outlives_target`, design changes 0134 and 0139) and a rally point on an object the holder's
/// owner still has a record of: they go on until the owner sees its place.
pub fn dangling_references(sim: &Sim) -> Vec<(ObjectId, ObjectId)> {
    sim.objects()
        .flat_map(|o| {
            let mut ids = referenced_ids(o);
            let remembered = |id| {
                o.owner
                    .and_then(|p| sim.vision().player(p))
                    .is_some_and(|v| v.record(id).is_some())
            };
            let mut outliving = Vec::new();
            if let Some(orders) = &o.orders {
                let current = (orders.current.iter()).map(|active| (&active.order, Some(&active.state)));
                let queued = orders.queue.iter().map(|order| (order, None));
                for (order, state) in current.chain(queued) {
                    outliving.extend(
                        order
                            .target()
                            .filter(|&id| order.outlives_target(state, remembered(id))),
                    );
                }
            }
            if let Some(Rally::Object { id, .. }) = o.production.as_ref().and_then(|p| p.rally)
                && remembered(id)
            {
                outliving.push(id);
            }
            // One reference each, so that a move or an attack still naming the object is caught.
            for id in outliving {
                if let Some(i) = ids.iter().position(|&r| r == id) {
                    ids.swap_remove(i);
                }
            }
            ids.into_iter().map(move |id| (o.id, id))
        })
        .filter(|&(_, id)| sim.object(id).is_none())
        .collect()
}
