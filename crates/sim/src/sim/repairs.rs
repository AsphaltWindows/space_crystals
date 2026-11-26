//! The repairs in a game (slice M5-N8, plan decision T83; `crate::repair`, `orders::repair`): in the economy phase,
//! right after the sites' step (`sim/sites.rs`), each object that repairers are at work on regains HP for each of them,
//! and its owner pays for it; the Repair orders that are over end.
//!
//! - **Who is at work** (`spot_work::at_work`, as a site's builders are counted): each unit whose current order is a
//!   Repair of an object it can repair now (`repair::can_repair`), standing within reach of its edge, its walk over.
//!   A Repair of an object it can't repair any more ends: one at full HP, no longer its owner's (a Laser Turret whose
//!   Tunnel is lost is nobody's, design change 0360), or no longer complete. One whose object left the game ended in
//!   the removal path (`Sim::forget_references`).
//! - **The HP**: `n` repairers at work restore `n` × MaxHP / the repair's frames (`repair::Terms::frames`) in the tick,
//!   up to the MaxHP: "several repairers on one target repair **faster, scaling linearly** (2 repairers = twice the
//!   rate), as construction does" (`factions/cults/units/recruit.md > Repair`). Those that repair it for free first,
//!   then those that pay (**Placeholder (not in the design: free and paying repairers on one object)**, which no
//!   faction has yet: the Agent's free Tunnel is no one else's to repair).
//! - **The price** (`repair::step_of`): the HP the paying repairers restore costs the crystals of the steps of the
//!   repair's price it enters, taken from the owner's bank as it is restored ("paid gradually as HP is restored"). A
//!   bank short of them pays what it holds, the HP going no further than the top of the last step paid for
//!   (`repair::top_of_step`): the bank never goes below 0.
//! - **The end** (design change 0329, `factions/cults/units/recruit.md > Repair`: "repairers **stop once the target is
//!   at full HP**; if the player **runs out of Space Crystals** mid-repair, the order **ends** (they stop; a free
//!   Tunnel repair never runs out)"): at full HP, the current Repair of every unit on the object ends; when its
//!   owner's bank can't pay for the next step, that of every unit that pays for it does, those still on their way
//!   included (**Placeholder (not in the design: whether the repairers on their way stop too)**: the order ends for
//!   all who pay). Their queued orders go on from the next tick (`orders::update`). A Repair given while the bank is
//!   empty is taken, and ends as soon as it needs a crystal (**Placeholder (not in the design: a Repair ordered with
//!   no crystals)**: the gate doesn't judge the bank, which may fill on the way).

use std::collections::BTreeMap;

use super::Sim;
use crate::augmentation::Research;
use crate::defs::RepairPayment;
use crate::ids::ObjectId;
use crate::object::Object;
use crate::repair::{self, step_of, top_of_step};
use crate::spot_work;

/// The repairers of one object in a tick ([`Sim::step_repairs`]).
#[derive(Default)]
struct Crew {
    /// Those at work that repair it for free, and those that pay.
    free: u32,
    paid: u32,
    /// Every unit whose current order repairs it, at work or not, and how it pays.
    units: Vec<(ObjectId, RepairPayment)>,
}

/// The object `o`'s current order repairs, if it is a Repair.
fn repairing(o: &Object) -> Option<ObjectId> {
    let active = o.orders.as_ref()?.current.as_ref()?;
    spot_work::repaired(active.order.kind)
}

impl Sim {
    /// Whether unit `unit` can repair object `target` now (`repair::can_repair`): an object of its owner's, complete
    /// and below its MaxHP, of a kind its type repairs. The gate's rule for a Repair (`Sim::check_order`). `false` if
    /// either is gone.
    pub fn can_repair(&self, unit: ObjectId, target: ObjectId) -> bool {
        match (self.objects.get(&unit), self.objects.get(&target)) {
            (Some(unit), Some(target)) => repair::can_repair(unit, target, self.research()),
            _ => false,
        }
    }

    /// The repairs' step of the economy phase, right after the sites' (module docs): the objects that repairers are at
    /// work on, in id order, each regain their HP and are paid for from their owner's bank; the Repairs that are over
    /// end.
    pub(super) fn step_repairs(&mut self) {
        let mut crews: BTreeMap<ObjectId, Crew> = BTreeMap::new();
        let mut over: Vec<ObjectId> = Vec::new();
        let research = Research::new(&self.players);
        for o in self.objects.values() {
            let Some(target) = repairing(o) else {
                continue;
            };
            // Gone: the removal path ended the order.
            let Some(t) = self.objects.get(&target) else {
                continue;
            };
            let payment = repair::can_repair(o, t, research)
                .then(|| repair::payment(o.kind, t.kind))
                .flatten();
            let Some(payment) = payment else {
                over.push(o.id);
                continue;
            };
            let crew = crews.entry(target).or_default();
            crew.units.push((o.id, payment));
            if spot_work::at_work(o, &self.objects, research, spot_work::repaired).is_some() {
                match payment {
                    RepairPayment::Free => crew.free += 1,
                    RepairPayment::Paid => crew.paid += 1,
                }
            }
        }
        for (target, crew) in crews {
            if crew.free + crew.paid == 0 {
                continue;
            }
            let Some((full, broke)) = self.repair(target, crew.free, crew.paid) else {
                continue;
            };
            let ends = |payment: RepairPayment| full || (broke && payment == RepairPayment::Paid);
            over.extend(
                (crew.units.iter())
                    .filter(|(_, payment)| ends(*payment))
                    .map(|(id, _)| *id),
            );
        }
        for id in over {
            self.end_work(id);
        }
    }

    /// Object `target` regains the HP of `free` repairers at work that repair it for free and `paid` that pay, its
    /// owner paying for the latter (module docs). Returns whether it is at full HP now, and whether the bank ran short
    /// of a crystal it owed; `None` if it isn't damaged or has no terms.
    fn repair(&mut self, target: ObjectId, free: u32, paid: u32) -> Option<(bool, bool)> {
        let t = self.objects.get(&target)?;
        let research = Research::new(&self.players);
        let ((hp, max), terms, owner) = (repair::damage(t, research)?, repair::terms(t.kind)?, t.owner);
        let rate = max / terms.frames;
        let mut restored = (hp + rate * free as f32).min(max);
        let mut broke = false;
        if paid > 0
            && let Some(price) = terms.price
        {
            let wanted = (restored + rate * paid as f32).min(max);
            let from = step_of(restored, max, price);
            let owed = step_of(wanted, max, price).saturating_sub(from);
            let bank = (self.players.iter_mut()).find(|p| Some(p.id) == owner);
            let crystals = bank.as_ref().map_or(0, |bank| bank.crystals);
            let pays = owed.min(u64::from(crystals));
            restored = match pays == owed {
                true => wanted,
                false => top_of_step(from + pays, max, price, wanted).max(restored),
            };
            broke = pays < owed;
            if let Some(bank) = bank {
                // `pays` is at most the bank's crystals, a u32.
                let spent = bank.spend_crystals(pays as u32);
                debug_assert!(spent, "a repair pays what the bank holds at most");
            }
        }
        let o = self.objects.get_mut(&target)?;
        o.hp = Some(restored);
        Some((restored >= max, broke))
    }
}

#[cfg(test)]
mod tests;
