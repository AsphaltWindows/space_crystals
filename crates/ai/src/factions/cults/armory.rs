//! The Cults' Armory (`super` module docs, 1 and 3): the Recruits it takes in, and the Militants and Gunners it trains
//! (`factions/cults/structures/armory.md`).
//!
//! - **Entering** ([`Cults::armory_room`]): a Recruit without a seat enters the player's first complete Armory, by id,
//!   while it has room (`factions/cults/structures/armory.md > InternalRecruitCapacity - 10`), those inside and those
//!   on their way in counted (`factions/cults/structures/README.md > Choosing Recruits, Release and Eject All`:
//!   "Capacity counts everyone inside, reserved units included").
//! - **Training** ([`Cults::train`]): for each Plain Recruit of its pool that no order reserves yet
//!   (`factions/cults/structures/armory.md > Training`: "Queuing an order immediately deducts its Space Crystals cost
//!   from the player and reserves one Recruit from StoredRecruits"), KindMenu\[Plain\]'s Train Militant, or Train
//!   Gunner when [`gunner_next`] says so (`ProductionCommand::TrainFrom`), while the bank covers it. A unit the bank
//!   can't pay for yet holds the bank.

use sc_sim::network::Place;
use sc_sim::{Command, Object, ObjectId, ObjectKind, OrderKind, ProductionCommand, RecruitKind};

use super::{Cults, gunner_next, own};
use crate::{Turn, View};

use ObjectKind::{Armory, Gunner, Militant, Recruit};

impl Cults {
    /// The player's first complete Armory and how many more Recruits it takes (module docs).
    pub(super) fn armory_room(&self, view: &View<'_>) -> Option<(ObjectId, u32)> {
        let armory = own(view, Armory).find(|o| o.is_operational())?;
        let capacity = sc_sim::holding::holds(armory)?.capacity;
        let inside = view
            .own_held()
            .filter(|h| h.place == Place::Building(armory.id))
            .count();
        let coming = own(view, Recruit).filter(|o| enters(o, armory.id)).count();
        let taken = u32::try_from(inside + coming).unwrap_or(u32::MAX);
        Some((armory.id, capacity.saturating_sub(taken)))
    }

    /// The Armory's training (module docs).
    pub(super) fn train(&mut self, turn: &mut Turn<'_, '_>) {
        let view = turn.view();
        let Some(armory) = own(view, Armory).find(|o| o.is_operational()) else {
            return;
        };
        let Some(production) = &armory.production else {
            return;
        };
        let free = (view.own_held())
            .filter(|h| h.place == Place::Building(armory.id) && h.leaving.is_none())
            .filter(|h| !production.reserves(h.object.id) && h.object.augmentation.kind() == RecruitKind::Plain)
            .count();
        for _ in 0..free {
            let first = if gunner_next(view, self.trained) {
                Gunner
            } else {
                Militant
            };
            // A Gunner the gate refuses (its TechPrerequisite) gives way to a Militant.
            let trained = [first, Militant].into_iter().find_map(|kind| {
                let price = price(armory, kind)?;
                if !turn.can_pay(price) {
                    return Some(false);
                }
                let command = Command::Production {
                    objects: vec![armory.id],
                    command: ProductionCommand::TrainFrom {
                        kind,
                        from: RecruitKind::Plain,
                    },
                };
                turn.buy(price, command).then_some(true)
            });
            if trained != Some(true) {
                return;
            }
            self.trained += 1;
        }
    }
}

/// What `producer` charges for a `kind`, if it trains them.
fn price(producer: &Object, kind: ObjectKind) -> Option<u32> {
    let production = producer.kind.def().structure.as_ref()?.production.as_ref()?;
    production.recipe(kind).map(|r| r.cost)
}

/// Whether `o` is on its way into `building`: an Enter of it is its order, current or queued.
fn enters(o: &Object, building: ObjectId) -> bool {
    o.orders.as_ref().is_some_and(|orders| {
        let current = orders.current.iter().map(|active| active.order.kind);
        let mut kinds = current.chain(orders.queue.iter().map(|order| order.kind));
        kinds.any(|kind| matches!(kind, OrderKind::Enter(b) if b == building))
    })
}
