//! (M6-GA) The vehicle deploys' step (plan decision T44: "Progress and completion in a Sim step right after
//! `step_modes`, before 5a"; the model is `crate::vehicle_deploy`'s): each deploy under way takes its frame, and one
//! whose frames are done completes, through `Sim::change_kind`, its unit's held and queued orders checked against the
//! new type.

use super::Sim;
use crate::defs::Domain;
use crate::events::SimEvent;
use crate::ids::ObjectId;
use crate::orders::OrderKind;
use crate::vehicle_deploy;

impl Sim {
    /// The step right after the Modes step (`ARCHITECTURE.md > Tick pipeline`), for the units before `first_new`, in
    /// id order: each one that deploys takes one frame of its deploy (`vehicle_deploy::step`), and each whose last frame
    /// that was completes its deploy ([`Sim::complete_vehicle_deploy`]). Before phase 5a, so that the unit acts as its
    /// new type from the tick's first phase, its held order first; a deploy that 5a starts takes its first frame at the
    /// next tick's step (`vehicle_deploy`'s timing).
    pub(super) fn step_vehicle_deploys(&mut self, first_new: ObjectId) {
        let done: Vec<(ObjectId, crate::defs::ObjectKind)> = (self.objects.range_mut(..first_new))
            .filter_map(|(&id, o)| vehicle_deploy::step(o).map(|into| (id, into)))
            .collect();
        for (id, into) in done {
            self.complete_vehicle_deploy(id, into);
        }
    }

    /// Unit `id`'s deploy is done (`factions/gdo/units/artillery_rig.md > Modes`: "The unit becomes Artillery Rig
    /// (Deployed) (type and SelectionGroup) when deploying finishes"): its deploy order ends, and it becomes a unit of
    /// type `into` (`Sim::change_kind`, with its `SimEvent::KindChanged`: the same object, its orders and its move kept,
    /// its attack the new type's). Then "each order is checked against the deployed unit: one it can't carry out (e.g.
    /// **Move**, since a deployed unit can't move) is **dropped** with the usual unavailable-command feedback [...], and
    /// the next one is tried": each of its queue, in order, by the gate's rule (`vehicle_deploy::can_take`), the target
    /// of an Attack as its owner knows it; each it drops with a `SimEvent::OrderDropped`. Phase 5a starts the first
    /// one it keeps, the held order if it holds one.
    ///
    /// GP's site arm (plan decision T87) goes here: a deploy whose type it becomes is a structure would place it on
    /// its site (a placement for `change_kind`) and drop the unit's orders, which a structure doesn't take.
    fn complete_vehicle_deploy(&mut self, id: ObjectId, into: crate::defs::ObjectKind) {
        if let Some(orders) = self.objects.get_mut(&id).and_then(|o| o.orders.as_mut()) {
            orders.current = None;
        }
        if !self.change_kind(id, into, None) {
            return;
        }
        let Some(o) = self.objects.get(&id) else {
            return;
        };
        let Some(orders) = &o.orders else {
            return;
        };
        let research = self.research();
        let owner = o.owner;
        let domain = |kind: OrderKind| -> Option<Domain> {
            let OrderKind::Attack(target) = kind else {
                return None;
            };
            self.known_domain(owner?, target).ok()
        };
        let dropped: Vec<usize> = (orders.queue.iter().enumerate())
            .filter(|(_, order)| vehicle_deploy::can_take(o, order.kind, domain(order.kind), research).is_err())
            .map(|(i, _)| i)
            .collect();
        if dropped.is_empty() {
            return;
        }
        let Some(orders) = self.objects.get_mut(&id).and_then(|o| o.orders.as_mut()) else {
            return;
        };
        let mut index = 0;
        let mut gone = Vec::with_capacity(dropped.len());
        orders.queue.retain(|order| {
            let keep = !dropped.contains(&index);
            index += 1;
            if !keep {
                gone.push(order.kind);
            }
            keep
        });
        for order in gone {
            self.events.push(SimEvent::OrderDropped { unit: id, order });
        }
    }
}
