//! The `Orders` capability: a unit's command queue and the behaviour of its current order —
//! `systems/controls/commands.md > Unit Command`, `systems/controls/behaviors.md`, `docs/movement.md > 3`, `> 4`.
//!
//! A player's `Command::Order` (phase 4) only writes queues ([`Orders::give`]) and computes group slots
//! ([`slots`]); it never moves anything. Phase 5a ([`update`]) completes orders, starts the next one and runs the
//! behaviours, which write `Mover::goal` and read `Mover::status`.
//!
//! Behaviours in this slice: Move to a point, Stop and Hold. Move to an object, Attack, AttackMove and Patrol are
//! accepted into the queue but complete at once (slice E gives them their behaviour).

pub(crate) mod slots;

use std::collections::{BTreeMap, VecDeque};

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::hash::{StateHash, StateHasher};
use crate::ids::ObjectId;
use crate::mover::{MoveGoal, MoveStatus, Mover};
use crate::nav::PathGroup;
use crate::object::Object;
use crate::units::Tick;

/// `docs/movement.md > 2`, G13: orders waiting in one unit's queue, besides the current one. A technical cap: an
/// order queued beyond it is dropped for that unit.
pub const MAX_QUEUED_ORDERS: usize = 32;

/// Units one order command may name, duplicates included. Technical: selections have no size limit
/// (`systems/controls/selection_and_hud.md`), so this only bounds the work of one command.
pub const MAX_ORDER_UNITS: usize = 1024;

/// The units of one order command share it (`docs/movement.md > 3`). Allocated from `Sim::next_group`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct OrderGroup(pub u32);

/// What a move order goes to (`systems/controls/commands.md > Move`: a location or an object).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Target {
    /// A point, space units: where the player clicked.
    Point(Vec2),
    Object(ObjectId),
}

/// The orders of `systems/controls/commands.md` that units have so far (`docs/movement.md > 3`).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum OrderKind {
    /// `> Move`.
    Move(Target),
    /// `> Attack`: a destructible object.
    Attack(ObjectId),
    /// `> AttackMove`: a point.
    AttackMove(Vec2),
    /// `> Patrol`: the far end of the patrol, a point.
    Patrol(Vec2),
    /// `> HoldPosition`.
    Hold,
    /// `> Stop`.
    Stop,
}

/// One unit's order.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Order {
    pub kind: OrderKind,
    pub group: OrderGroup,
    /// For a group's move to a point: where this unit goes, its slot around the clicked point
    /// (`docs/movement.md > 6.4`); the point itself if `None`.
    pub slot: Option<Vec2>,
}

impl Order {
    /// The point a move to a point goes to: the unit's slot, else the clicked point.
    pub fn destination(&self) -> Option<Vec2> {
        match self.kind {
            OrderKind::Move(Target::Point(click)) => Some(self.slot.unwrap_or(click)),
            OrderKind::Move(Target::Object(_))
            | OrderKind::Attack(_)
            | OrderKind::AttackMove(_)
            | OrderKind::Patrol(_)
            | OrderKind::Hold
            | OrderKind::Stop => None,
        }
    }
}

/// The order a unit is carrying out.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActiveOrder {
    pub order: Order,
    /// The tick it started.
    pub since: Tick,
}

/// A unit's orders (`systems/controls/commands.md > Unit Command`): the current one, and the queue of those
/// that start, in order, when it completes.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Orders {
    /// At most [`MAX_QUEUED_ORDERS`].
    pub queue: VecDeque<Order>,
    pub current: Option<ActiveOrder>,
}

impl Orders {
    /// `systems/controls/commands.md > Unit Command`: with Shift (`queue`) the order is appended to the queue;
    /// without, it replaces the whole queue and the current order. Returns `false` if the queue was full and the
    /// order was dropped. The new order starts in phase 5a.
    pub fn give(&mut self, order: Order, queue: bool) -> bool {
        if !queue {
            self.queue.clear();
            self.current = None;
        }
        if self.queue.len() >= MAX_QUEUED_ORDERS {
            return false;
        }
        self.queue.push_back(order);
        true
    }

    /// Whether the unit has no order at all.
    pub fn is_idle(&self) -> bool {
        self.current.is_none() && self.queue.is_empty()
    }
}

/// Phase 5a (`docs/movement.md > 4`): for every unit before `first_new`, in id order, complete the current order
/// if its behaviour is done, start queued orders until one is still running, and let that behaviour write the
/// unit's goal. It reads `Mover::status`, which phase 5d set last tick.
pub fn update(objects: &mut BTreeMap<ObjectId, Object>, first_new: ObjectId, tick: Tick) {
    for o in objects.range_mut(..first_new).map(|(_, o)| o) {
        let (Some(orders), Some(mover)) = (&mut o.orders, &mut o.mover) else {
            continue;
        };
        update_unit(orders, mover, tick);
    }
}

fn update_unit(orders: &mut Orders, mover: &mut Mover, tick: Tick) {
    // Each pass completes an order or starts one from the queue, which is bounded, so this ends.
    loop {
        if let Some(active) = &orders.current {
            if !is_complete(&active.order, mover) {
                return;
            }
            orders.current = None;
        }
        let Some(order) = orders.queue.pop_front() else {
            // No order left: the unit has no behaviour and keeps its last move's outcome.
            return;
        };
        start(&order, mover, tick);
        orders.current = Some(ActiveOrder { order, since: tick });
    }
}

/// A behaviour begins (`systems/controls/behaviors.md`).
fn start(order: &Order, mover: &mut Mover, tick: Tick) {
    match order.kind {
        // `> MovingToLocation`: the unit goes to its slot; the group plans from the clicked point.
        OrderKind::Move(Target::Point(click)) => {
            let goal = MoveGoal::Point(order.slot.unwrap_or(click));
            let group = PathGroup {
                id: order.group.0,
                anchor: click,
            };
            mover.go_to(goal, Some(group), tick);
        }
        // `> StoppingBehavior`, `> HoldingPosition`: no goal. Infinite deceleration stops the unit at once.
        OrderKind::Stop | OrderKind::Hold => mover.stop(),
        // TODO(slice E): `> MovingToObject`, `> AttackingObject`, `> AttackMovingToLocation`, `> Patrolling`.
        // Until then they stop the unit and complete at once.
        OrderKind::Move(Target::Object(_)) | OrderKind::Attack(_) | OrderKind::AttackMove(_) | OrderKind::Patrol(_) => {
            mover.stop()
        }
    }
}

/// Whether the order's behaviour is complete, so the next order may start.
fn is_complete(order: &Order, mover: &Mover) -> bool {
    match order.kind {
        // `> MovingToLocation`: complete when stopped at the target, or after failing to make progress
        // (`> Failure`).
        OrderKind::Move(Target::Point(_)) => matches!(
            mover.status,
            MoveStatus::Arrived | MoveStatus::Failed | MoveStatus::TargetLost | MoveStatus::Idle
        ),
        // `> StoppingBehavior`: complete when stopped, at once with infinite deceleration.
        OrderKind::Stop => true,
        // `> HoldingPosition`: never completes; only a new command ends it.
        OrderKind::Hold => false,
        // TODO(slice E): their behaviours; no-ops until then.
        OrderKind::Move(Target::Object(_)) | OrderKind::Attack(_) | OrderKind::AttackMove(_) | OrderKind::Patrol(_) => {
            true
        }
    }
}

impl StateHash for OrderGroup {
    fn hash(&self, h: &mut StateHasher) {
        self.0.hash(h);
    }
}

impl StateHash for Target {
    fn hash(&self, h: &mut StateHasher) {
        match self {
            Target::Point(p) => {
                h.tag(0);
                p.hash(h);
            }
            Target::Object(id) => {
                h.tag(1);
                id.hash(h);
            }
        }
    }
}

impl StateHash for OrderKind {
    fn hash(&self, h: &mut StateHasher) {
        match self {
            OrderKind::Move(target) => {
                h.tag(0);
                target.hash(h);
            }
            OrderKind::Attack(id) => {
                h.tag(1);
                id.hash(h);
            }
            OrderKind::AttackMove(p) => {
                h.tag(2);
                p.hash(h);
            }
            OrderKind::Patrol(p) => {
                h.tag(3);
                p.hash(h);
            }
            OrderKind::Hold => h.tag(4),
            OrderKind::Stop => h.tag(5),
        }
    }
}

impl StateHash for Order {
    fn hash(&self, h: &mut StateHasher) {
        let Order { kind, group, slot } = self;
        kind.hash(h);
        group.hash(h);
        slot.hash(h);
    }
}

impl StateHash for ActiveOrder {
    fn hash(&self, h: &mut StateHasher) {
        let ActiveOrder { order, since } = self;
        order.hash(h);
        since.hash(h);
    }
}

impl StateHash for Orders {
    fn hash(&self, h: &mut StateHasher) {
        let Orders { queue, current } = self;
        queue.hash(h);
        current.hash(h);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn order(kind: OrderKind) -> Order {
        Order {
            kind,
            group: OrderGroup(0),
            slot: None,
        }
    }

    fn move_to(x: f32, y: f32) -> Order {
        order(OrderKind::Move(Target::Point(Vec2::new(x, y))))
    }

    /// `systems/controls/commands.md > Unit Command`.
    #[test]
    fn shift_appends_and_otherwise_the_order_replaces_the_queue() {
        let mut orders = Orders::default();
        assert!(orders.is_idle());
        assert!(orders.give(move_to(1.0, 1.0), true));
        assert!(orders.give(move_to(2.0, 2.0), true));
        orders.current = Some(ActiveOrder {
            order: move_to(0.0, 0.0),
            since: 1,
        });
        assert_eq!(orders.queue.len(), 2);
        assert!(orders.give(order(OrderKind::Stop), false));
        assert_eq!(orders.current, None);
        assert_eq!(orders.queue, [order(OrderKind::Stop)]);
    }

    #[test]
    fn the_queue_is_capped() {
        let mut orders = Orders::default();
        for i in 0..MAX_QUEUED_ORDERS {
            assert!(orders.give(move_to(i as f32, 0.0), true));
        }
        assert!(!orders.give(move_to(99.0, 0.0), true));
        assert_eq!(orders.queue.len(), MAX_QUEUED_ORDERS);
        // Replacing always fits.
        assert!(orders.give(move_to(99.0, 0.0), false));
        assert_eq!(orders.queue.len(), 1);
    }

    #[test]
    fn moves_complete_on_arrival_and_the_next_order_starts_at_once() {
        let mut orders = Orders::default();
        let mut mover = Mover::default();
        orders.give(move_to(10.0, 0.0), true);
        orders.give(move_to(20.0, 0.0), true);
        update_unit(&mut orders, &mut mover, 1);
        assert_eq!(mover.goal, MoveGoal::Point(Vec2::new(10.0, 0.0)));
        assert_eq!(orders.queue.len(), 1);
        // Still on the way: nothing changes.
        update_unit(&mut orders, &mut mover, 2);
        assert_eq!(orders.current.as_ref().map(|a| a.since), Some(1));
        mover.finish(MoveStatus::Arrived);
        update_unit(&mut orders, &mut mover, 3);
        assert_eq!(mover.goal, MoveGoal::Point(Vec2::new(20.0, 0.0)));
        assert_eq!(orders.current.as_ref().map(|a| a.since), Some(3));
        mover.finish(MoveStatus::Failed);
        update_unit(&mut orders, &mut mover, 4);
        assert!(orders.is_idle());
        assert_eq!(mover.status, MoveStatus::Failed, "the last outcome stays readable");
    }

    #[test]
    fn stop_completes_at_once_and_hold_never_does() {
        let mut orders = Orders::default();
        let mut mover = Mover::default();
        orders.give(move_to(10.0, 0.0), true);
        update_unit(&mut orders, &mut mover, 1);
        orders.give(order(OrderKind::Stop), true);
        orders.give(order(OrderKind::Hold), false);
        orders.give(move_to(30.0, 0.0), true);
        update_unit(&mut orders, &mut mover, 2);
        assert_eq!(mover.goal, MoveGoal::None);
        assert_eq!(orders.current.as_ref().map(|a| a.order.kind), Some(OrderKind::Hold));
        for tick in 3..50 {
            update_unit(&mut orders, &mut mover, tick);
        }
        assert_eq!(orders.queue.len(), 1, "orders queued behind Hold wait");
        assert_eq!(mover.goal, MoveGoal::None);

        let mut stopping = Orders::default();
        stopping.give(order(OrderKind::Stop), true);
        stopping.give(move_to(5.0, 5.0), true);
        update_unit(&mut stopping, &mut mover, 60);
        assert_eq!(
            mover.goal,
            MoveGoal::Point(Vec2::new(5.0, 5.0)),
            "Stop completed in its tick"
        );
    }

    #[test]
    fn later_orders_are_no_ops_for_now() {
        let mut orders = Orders::default();
        let mut mover = Mover::default();
        mover.go_to(MoveGoal::Point(Vec2::ONE), None, 1);
        for kind in [
            OrderKind::Move(Target::Object(ObjectId(3))),
            OrderKind::Attack(ObjectId(3)),
            OrderKind::AttackMove(Vec2::ONE),
            OrderKind::Patrol(Vec2::ONE),
        ] {
            orders.give(order(kind), true);
        }
        update_unit(&mut orders, &mut mover, 2);
        assert!(orders.is_idle());
        assert_eq!(mover.goal, MoveGoal::None, "they stop the unit");
    }

    #[test]
    fn a_slot_overrides_the_clicked_point() {
        let mut o = move_to(1.0, 2.0);
        assert_eq!(o.destination(), Some(Vec2::new(1.0, 2.0)));
        o.slot = Some(Vec2::new(3.0, 4.0));
        assert_eq!(o.destination(), Some(Vec2::new(3.0, 4.0)));
        assert_eq!(order(OrderKind::Hold).destination(), None);
    }
}
