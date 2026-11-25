//! The one way objects leave the game: destroyed in combat (phase 5e) or removed by a debug command. Whatever the
//! cause, every reference other objects hold to them goes too, so no state ever names an object that is gone, but
//! for the orders that outlive their targets (`orders::Order::outlives_target`, the one rule of design changes 0134
//! and 0139): an order on an object its holder's owner remembers out of its sight goes on as if the object stood,
//! until the owner sees its place ([`Sim::follow_targets`], at the end of the tick), and so does a rally point on it;
//! a Gather on its way to its patch's spot goes on there.

use super::Sim;
use crate::economy;
use crate::events::SimEvent;
use crate::ids::ObjectId;
use crate::mover::{MoveGoal, MoveStatus};
use crate::object::Object;
use crate::orders::{Behaviour, Order};
use crate::production;

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
    /// A worker gathering at a patch that goes keeps its Gather, and heads for where the patch was
    /// (`Sim::keep_gatherers_on_spots`, design change 0134). A worker on its way back to a drop-off that goes takes
    /// the nearest other at once (`Sim::reassign_drop_offs`, design change 0170); any other forgets it
    /// (`Worker::forget`).
    ///
    /// Design change 0139 (`core/vision.md > Explored > Targeting remembered objects`: "Nothing about the real state
    /// leaks"): the orders that outlive their targets stay (`orders::Order::outlives_target`): an Attack, a Move to an
    /// object and a Gather, current or queued, on an object their holder's owner remembers out of its sight
    /// (`PlayerVision::remembered_record`, by the vision the last tick left), and a Gather on its way to its spot.
    /// The behaviours go on to the last-known position, and end once the owner sees the place
    /// ([`Sim::follow_targets`]). A rally point on a remembered object stays too. Everything else lets go of the
    /// object, the move and the attack included, which never name a remembered object
    /// (`orders::travel::walk_to_remembered`, the Gather's legs).
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
        // The workers gathering at a patch that goes, while it is still there to say where it was
        // (`Sim::keep_gatherers_on_spots`), and those on their way back to a drop-off that goes
        // (`Sim::reassign_drop_offs`).
        self.keep_gatherers_on_spots(&gone);
        self.reassign_drop_offs(&gone);
        for &id in &gone {
            self.refund_production(id);
        }
        self.refund_constructors(&gone);
        gone.retain(|id| self.objects.remove(id).is_some());
        if gone.is_empty() {
            return;
        }
        self.events.extend(gone.iter().map(|&id| SimEvent::Removed(id)));
        let vision = &self.vision;
        for o in self.objects.values_mut() {
            let owner_vision = o.owner.and_then(|owner| vision.player(owner));
            let remembered = |id: ObjectId| owner_vision.is_some_and(|v| v.remembered_record(id).is_some());
            forget(o, &gone, remembered);
        }
    }

    /// The end of the tick, on the vision it leaves (`Sim::step`, after `Vision::remember`): the orders and rally
    /// points that name an object follow what their owner knows of it.
    /// - Design change 0139: an object a player remembered, and whose place it now sees without it
    ///   (`PlayerVision::seen_gone`), is gone to that player: its orders that name it end "the way it would for a
    ///   vanished target" ("the unit stops there and takes its next queued order or idles": the current one completes
    ///   where the unit is, a queued one is dropped), and its rally points on it reset to None. A current Gather goes
    ///   on to its spot instead, and looks round there ("Gather: the redirect rules", `orders::Order::outlives_target`).
    ///   An object that still exists elsewhere (its footprint changed) is left to the view of it (`TargetView`).
    /// - Design change 0088: a rally point on an enemy unit turns into a ground rally point once the unit leaves its
    ///   owner's sight (`production::rally::follow`).
    pub(super) fn follow_targets(&mut self) {
        for vision in self.vision.players() {
            let gone: Vec<ObjectId> = vision
                .seen_gone()
                .iter()
                .copied()
                .filter(|id| !self.objects.contains_key(id))
                .collect();
            if gone.is_empty() {
                continue;
            }
            let player = vision.player();
            for o in self.objects.values_mut().filter(|o| o.owner == Some(player)) {
                forget_seen_gone(o, &gone);
            }
        }
        production::rally::follow(&mut self.objects, &self.vision);
    }
}

/// `o` drops every reference to the objects `gone` (sorted by id), but for its orders that outlive their targets
/// (`orders::Order::outlives_target`; `remembered` says whether its owner remembers an object out of its sight) and
/// its rally point on a remembered object (design change 0139).
fn forget(o: &mut Object, gone: &[ObjectId], remembered: impl Fn(ObjectId) -> bool) {
    let is_gone = |id: ObjectId| gone.binary_search(&id).is_ok();
    let ends = |order: &Order, state: Option<&Behaviour>| {
        (order.target()).is_some_and(|id| is_gone(id) && !order.outlives_target(state, remembered(id)))
    };
    let mut ended = false;
    if let Some(orders) = &mut o.orders {
        if (orders.current.as_ref()).is_some_and(|active| ends(&active.order, Some(&active.state))) {
            orders.current = None;
            ended = true;
        }
        orders.queue.retain(|order| !ends(order, None));
        // The enemies its behaviours engaged by themselves (`orders::state`).
        orders.forget(&is_gone);
    }
    if let Some(mover) = &mut o.mover {
        match mover.goal {
            MoveGoal::Object { id, .. } if is_gone(id) => mover.finish(MoveStatus::TargetLost),
            // (M3-G2) The order's walk to where its target was last seen, a move to a point, ends with the order
            // (`systems/controls/behaviors.md > MovingToObject > Completion`: "the unit stops where it is"), as in
            // `forget_seen_gone`. It went on to that point without an order (`behaviours_soak`'s seed 17).
            MoveGoal::Point(_) if ended => mover.stop(),
            _ => {}
        }
    }
    if let Some(attack) = &mut o.attack {
        attack.forget(gone);
    }
    if o.extractor.is_some_and(|extractor| is_gone(extractor.patch)) {
        o.extractor = None;
    }
    // A rally point on an object that is gone resets to None (`factions/gdo/structures.md > Barracks > RallyPoint
    // behavior`; for an enemy unit only while its owner saw it, `production::rally::follow`), but for one on a
    // remembered object (design change 0139: "A rally point on an enemy structure stays on it after it leaves sight").
    if let Some(production) = &mut o.production {
        production.forget(|id| is_gone(id) && !remembered(id));
    }
    if let Some(worker) = &mut o.worker {
        worker.forget(&is_gone);
    }
    // The building whose doorway a unit stood in: nothing is left to overlap, nor to leave along (design change
    // 0145).
    if let Some(mover) = &mut o.mover
        && mover.exit.is_some_and(is_gone)
    {
        mover.exit = None;
        mover.exit_heading = None;
    }
}

/// [`Sim::follow_targets`]: `o`'s owner has seen the objects `gone` (sorted by id) gone, which it remembered until
/// now: its orders that name one end, the current one where the unit is, but for a Gather on its way to its spot
/// (`orders::Order::outlives_target`); and its rally point on one resets.
fn forget_seen_gone(o: &mut Object, gone: &[ObjectId]) {
    let is_gone = |id: ObjectId| gone.binary_search(&id).is_ok();
    let ends = |order: &Order, state: Option<&Behaviour>| {
        (order.target()).is_some_and(|id| is_gone(id) && !order.outlives_target(state, false))
    };
    if let Some(orders) = &mut o.orders {
        if (orders.current.as_ref()).is_some_and(|active| ends(&active.order, Some(&active.state))) {
            orders.current = None;
            // The walk to the last-known position, a move to a point, stops there.
            if let Some(mover) = &mut o.mover
                && mover.goal != MoveGoal::None
            {
                mover.stop();
            }
        }
        orders.queue.retain(|order| !ends(order, None));
    }
    if let Some(production) = &mut o.production {
        production.forget(is_gone);
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
        // (No player has a record of a unit, so nothing here outlives it.)
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
