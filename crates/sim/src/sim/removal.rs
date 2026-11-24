//! The one way objects leave the game: destroyed in combat (phase 5e) or removed by a debug command. Whatever the
//! cause, every reference other objects hold to them goes too, so no state ever names an object that is gone.

use super::Sim;
use crate::economy;
use crate::events::SimEvent;
use crate::ids::ObjectId;
use crate::mover::{MoveGoal, MoveStatus};
use crate::object::Object;
use crate::orders::Order;

impl Sim {
    /// Removes the objects `ids` (those that exist), each with a `SimEvent::Removed`, in id order. Then, for every
    /// other object: an order naming one of them completes if it is the current order and is dropped if queued
    /// (`systems/controls/behaviors.md > AttackingObject` 5: the target destroyed, the behaviour is complete; `>
    /// MovingToObject > Completion`: a Follow ends when its target dies); a behaviour that engaged one by itself
    /// drops it (`Orders::forget`); a move towards one ends as `MoveStatus::TargetLost`; an attack on one stops
    /// (`combat::AttackState::forget`).
    ///
    /// A plate takes its patch along if that is depleted, in the same tick (`factions/gdo/economy.md >
    /// ExtractionPlate > On destruction`: "the patch is removed from the map"; a debug removal stands for a
    /// destruction), which opens its tile (design change 0038); a patch with crystals left stays, uncovered. A plate
    /// whose patch is removed from under it (a debug command) mines nothing from then on.
    ///
    /// A producer's whole queue is refunded as if cancelled, a finished unit waiting inside for the doorway included
    /// (design changes 0084 and 0085, `Sim::refund_production`); of the army cap it frees what its unit in production
    /// took, the only one of the queue that took any (design change 0143).
    /// A rally point on a removed object resets to None, and a unit in the doorway of a removed building is an
    /// ordinary unit from then on.
    ///
    /// A constructor refunds its construction or its ready building to its owner as if cancelled (design change 0084,
    /// `Sim::refund_constructors`).
    ///
    /// A worker gathering at a patch that goes is sent to another first, or its order ends
    /// (`Sim::redirect_gatherers`, question Q0037 point 7); a worker forgets a drop-off and a patch that are gone
    /// (`Worker::forget`).
    ///
    /// The grids aren't touched: phase 5b catches up with a footprint that went away, and a removal after it must
    /// sync them (`paths::sync`).
    pub(super) fn remove_objects(&mut self, ids: &[ObjectId]) {
        let mut gone: Vec<ObjectId> = ids.to_vec();
        gone.extend(
            ids.iter()
                .filter_map(|&id| economy::mining::depleted_patch(&self.objects, id)),
        );
        gone.sort_unstable();
        gone.dedup();
        // The workers gathering at a patch that goes: on to another one, while this one is still there to measure
        // from (`Sim::redirect_gatherers`).
        self.redirect_gatherers(&gone);
        for &id in &gone {
            self.refund_production(id);
        }
        self.refund_constructors(&gone);
        gone.retain(|id| self.objects.remove(id).is_some());
        if gone.is_empty() {
            return;
        }
        self.events.extend(gone.iter().map(|&id| SimEvent::Removed(id)));
        for o in self.objects.values_mut() {
            forget(o, &gone);
        }
    }
}

/// `o` drops every reference to the objects `gone` (sorted by id).
fn forget(o: &mut Object, gone: &[ObjectId]) {
    let is_gone = |id: ObjectId| gone.binary_search(&id).is_ok();
    let names_gone = |order: &Order| order.target().is_some_and(is_gone);
    if let Some(orders) = &mut o.orders {
        if orders.current.as_ref().is_some_and(|active| names_gone(&active.order)) {
            orders.current = None;
        }
        orders.queue.retain(|order| !names_gone(order));
        // The enemies its behaviours engaged by themselves (`orders::state`).
        orders.forget(&is_gone);
    }
    if let Some(mover) = &mut o.mover
        && let MoveGoal::Object { id, .. } = mover.goal
        && is_gone(id)
    {
        mover.finish(MoveStatus::TargetLost);
    }
    if let Some(attack) = &mut o.attack {
        attack.forget(gone);
    }
    if o.extractor.is_some_and(|extractor| is_gone(extractor.patch)) {
        o.extractor = None;
    }
    // A rally point on an object that is gone resets to None (`factions/gdo/structures.md > Barracks > RallyPoint
    // behavior`; for an enemy only while its owner saw it, `production::rally::follow`).
    if let Some(production) = &mut o.production {
        production.forget(is_gone);
    }
    if let Some(worker) = &mut o.worker {
        worker.forget(&is_gone);
    }
    // The building whose doorway a unit stood in: nothing is left to overlap.
    if let Some(mover) = &mut o.mover
        && mover.exit.is_some_and(is_gone)
    {
        mover.exit = None;
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use super::*;
    use crate::combat::{AttackPhase, AttackState};
    use crate::defs::ObjectKind;
    use crate::ids::PlayerId;
    use crate::orders::state::Pursuit;
    use crate::orders::{ActiveOrder, Auto, Behaviour, OrderGroup, OrderKind, Target};
    use crate::scenarios;

    /// Every kind of reference: orders, a move towards an object, an attack, and what the behaviours engaged by
    /// themselves.
    #[test]
    fn removal_clears_every_reference_to_the_removed() {
        let mut sim = Sim::new(&scenarios::empty(8, 6));
        let mut unit = |x| {
            sim.insert_object(
                ObjectKind::Peacekeeper,
                Some(PlayerId(1)),
                Vec2::new(x, 100.0),
                None,
                None,
            )
        };
        let (a, b, c) = (unit(100.0), unit(200.0), unit(300.0));
        let order = |kind| Order {
            kind,
            group: OrderGroup(0),
            slot: None,
        };
        let o = sim.objects.get_mut(&a).unwrap();
        let orders = o.orders.as_mut().unwrap();
        orders.current = Some(ActiveOrder {
            order: order(OrderKind::Move(Target::Object(b))),
            since: 1,
            state: Behaviour::default(),
        });
        let point = order(OrderKind::Move(Target::Point(Vec2::ONE)));
        orders
            .queue
            .extend([order(OrderKind::Attack(b)), order(OrderKind::Attack(c)), point.clone()]);
        let mover = o.mover.as_mut().unwrap();
        mover.go_to(MoveGoal::Object { id: b, reach: 10.0 }, None, 1);
        o.attack = Some(AttackState {
            target: Some(b),
            phase: Some(AttackPhase::Aiming { target: b }),
            elapsed: 1,
        });
        // Unit c holds its position and engaged b by itself; the idle unit d went for b on its own.
        let d = sim.insert_object(
            ObjectKind::Peacekeeper,
            Some(PlayerId(1)),
            Vec2::new(400.0, 100.0),
            None,
            None,
        );
        sim.objects.get_mut(&c).unwrap().orders.as_mut().unwrap().current = Some(ActiveOrder {
            order: order(OrderKind::Hold),
            since: 1,
            state: Behaviour::Hold { target: Some(b) },
        });
        sim.objects.get_mut(&d).unwrap().orders.as_mut().unwrap().auto = Auto::Engaging {
            pursuit: Pursuit {
                target: b,
                chasing: true,
            },
            origin: Vec2::ZERO,
        };
        assert_eq!(crate::testing::referenced_ids(&sim.objects[&c]), [b]);
        assert_eq!(crate::testing::referenced_ids(&sim.objects[&d]), [b]);
        sim.events.clear();

        sim.remove_objects(&[b, ObjectId(99), b]);
        assert_eq!(sim.events, [SimEvent::Removed(b)], "once, and only what existed");
        assert!(!sim.objects.contains_key(&b));
        let o = &sim.objects[&a];
        let orders = o.orders.as_ref().unwrap();
        assert_eq!(orders.current, None, "the current order completed");
        assert_eq!(orders.queue, [order(OrderKind::Attack(c)), point]);
        let mover = o.mover.as_ref().unwrap();
        assert_eq!((mover.goal, mover.status), (MoveGoal::None, MoveStatus::TargetLost));
        assert_eq!(o.attack, Some(AttackState::default()));
        let held = sim.objects[&c].orders.as_ref().unwrap();
        assert!(matches!(
            held.current.as_ref().map(|active| &active.state),
            Some(Behaviour::Hold { target: None })
        ));
        assert_eq!(sim.objects[&d].orders.as_ref().unwrap().auto, Auto::default());
        assert_eq!(crate::testing::dangling_references(&sim), []);
    }
}
