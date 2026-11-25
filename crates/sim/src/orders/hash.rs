//! The state hash of the orders (`crate::hash`): each type's fields in a fixed encoding, destructured without `..`.

use super::{ActiveOrder, Order, OrderGroup, OrderKind, Orders, Target};
use crate::hash::{StateHash, StateHasher};

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
            OrderKind::Gather(id) => {
                h.tag(6);
                id.hash(h);
            }
            OrderKind::DropOff(id) => {
                h.tag(7);
                id.hash(h);
            }
            OrderKind::Deploy { origin } => {
                h.tag(8);
                origin.hash(h);
            }
            OrderKind::Enter(id) => {
                h.tag(9);
                id.hash(h);
            }
            OrderKind::AttackGround(at) => {
                h.tag(10);
                at.hash(h);
            }
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
        let ActiveOrder { order, since, state } = self;
        order.hash(h);
        since.hash(h);
        state.hash(h);
    }
}

impl StateHash for Orders {
    fn hash(&self, h: &mut StateHasher) {
        let Orders { queue, current, auto } = self;
        queue.hash(h);
        current.hash(h);
        auto.hash(h);
    }
}
