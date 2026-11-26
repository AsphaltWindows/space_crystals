//! Production in the sim (slice M2-D2; `crate::production` has the state and the rules it runs): the application of
//! `Command::Production`, whose gate is `producers/queuing.rs`, the producers' step of the economy phase
//! (`producers/step.rs`, with the start check of design change 0143), the release of a finished unit
//! (`producers/release.rs`), and the refund when a producer leaves the game. A producer that holds units makes them
//! of its units inside (`producers/pool.rs`, slice M5-F2).

pub(super) mod pool;
mod queuing;
mod release;
mod step;

use super::Sim;
use crate::defs::ProductionDef;
use crate::economy::{ArmyResource, Refund};
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::object::Object;
use crate::orders::Target;
use crate::production::{Production, ProductionStatus, QueueEntry, Rally, Refusal};

/// A `Command::Production` that passed the gate, with what the checks resolved.
pub(super) enum ProductionPlan {
    /// `accepted` queue `entry` and pay for it, in this order (design change 0083), each reserving the units listed with
    /// it (a pool producer's, `producers/pool.rs`; none elsewhere); `refused` don't.
    Train {
        entry: QueueEntry,
        accepted: Vec<(ObjectId, Vec<ObjectId>)>,
        refused: Vec<(ObjectId, Refusal)>,
    },
    /// The producers with something queued, in id order.
    Cancel(Vec<ObjectId>),
    /// The producers, in id order, and the target as the gate resolved it.
    SetRally(Vec<ObjectId>, Target),
}

/// A producer and what its type produces.
fn producer(o: &Object) -> Option<(&Production, &'static ProductionDef)> {
    let def = o.def().structure.as_ref()?.production.as_ref()?;
    Some((o.production.as_ref()?, def))
}

/// The event of `producer`'s entry `entry` leaving its queue unmade, with `refund` Space Crystals back (X, or the
/// producer leaving the game): a unit's `SimEvent::ProductionCancelled`, a Release's `SimEvent::ReleaseCancelled`
/// (slice M5-F2), which names the unit it had reserved, `reserved`'s first.
pub(super) fn cancelled(producer: ObjectId, entry: QueueEntry, reserved: &[ObjectId], refund: u32) -> Option<SimEvent> {
    match entry {
        QueueEntry::Unit(kind) => Some(SimEvent::ProductionCancelled { producer, kind, refund }),
        QueueEntry::Release => Some(SimEvent::ReleaseCancelled {
            producer,
            unit: *reserved.first()?,
        }),
    }
}

impl Sim {
    /// What producer `id` is working on (`factions/gdo/structures/barracks.md > BarracksInstanceState`:
    /// CurrentBuild and CurrentBuildProgress), for the interface: `None` for an object that produces nothing or has
    /// an empty queue. `waiting` says the unit is finished and held inside because the exit doorway is occupied
    /// (design change 0085: shown at 100%); `blocked` names the army cap the unit waits for at 0% (design change
    /// 0143: the owner's Unit Control or Beacon Capacity).
    pub fn production_status(&self, id: ObjectId) -> Option<ProductionStatus> {
        let o = self.objects.get(&id)?;
        let (production, def) = producer(o)?;
        let faction = o.owner.and_then(|owner| self.player(owner)).map(|p| p.faction);
        production.status(def, faction.map(ArmyResource::of))
    }

    /// Applies a checked `Command::Production` of `player`.
    pub(super) fn apply_production(&mut self, player: PlayerId, plan: ProductionPlan) {
        match plan {
            ProductionPlan::Train {
                entry,
                accepted,
                refused,
            } => {
                for (id, units) in accepted {
                    let Some(cost) = self.objects.get(&id).and_then(producer).map(|(_, def)| entry.cost(def)) else {
                        continue;
                    };
                    let event = match (entry, units.first()) {
                        (QueueEntry::Unit(kind), _) => SimEvent::ProductionQueued { producer: id, kind },
                        (QueueEntry::Release, Some(&unit)) => SimEvent::ReleaseQueued { producer: id, unit },
                        (QueueEntry::Release, None) => continue,
                    };
                    // "deducts 50 Space Crystals from player, adds Peacekeeper to BuildQueue" (the gate checked
                    // the bank, each producer after what the ones before it took); a Release is free.
                    let paid = self
                        .players
                        .iter_mut()
                        .find(|p| p.id == player)
                        .is_some_and(|p| p.spend_crystals(cost));
                    if !paid {
                        continue;
                    }
                    if let Some(production) = self.objects.get_mut(&id).and_then(|o| o.production.as_mut()) {
                        production.push(entry, units);
                        self.events.push(event);
                    }
                }
                if let QueueEntry::Unit(kind) = entry {
                    self.events
                        .extend(refused.into_iter().map(|(id, reason)| SimEvent::ProductionRefused {
                            producer: id,
                            kind,
                            reason,
                        }));
                }
            }
            ProductionPlan::Cancel(producers) => {
                for id in producers {
                    let Some(o) = self.objects.get_mut(&id) else {
                        continue;
                    };
                    let def = o.def().structure.as_ref().and_then(|s| s.production.as_ref());
                    let Some((entry, reserved)) = o.production.as_mut().and_then(Production::cancel_last) else {
                        continue;
                    };
                    // "removes last entry from BuildQueue, refunds full cost to player". The army cost goes with the
                    // unit in production (design change 0143), and a waiting one never took any: "the player can
                    // still cancel it for a full refund". A pool producer's entry "returns its reserved Recruit to
                    // StoredRecruits" (`factions/cults/structures/armory.md > ObjectInterfaceState[Armory]`).
                    let refund = Refund::Full.of(def.map_or(0, |def| entry.cost(def)));
                    if let Some(p) = self.players.iter_mut().find(|p| p.id == player) {
                        p.credit_crystals(refund);
                    }
                    self.unreserve(&reserved);
                    self.events.extend(cancelled(id, entry, &reserved, refund));
                }
            }
            ProductionPlan::SetRally(producers, target) => {
                for id in producers {
                    let rally = match target {
                        Target::Point(p) => Some(Rally::Point(p)),
                        // Design change 0086: "Setting a production building's rally point on the building itself
                        // clears it (RallyPoint = None)."
                        Target::Object(target) if target == id => None,
                        Target::Object(target) => self.rally_on(player, target),
                    };
                    if let Some(production) = self.objects.get_mut(&id).and_then(|o| o.production.as_mut()) {
                        production.rally = rally;
                    }
                }
            }
        }
    }

    /// `factions/gdo/structures/barracks.md > Destroyed while producing` (design changes 0084 and 0085): object
    /// `id` is about to leave the game. If it produces units, its whole BuildQueue is refunded to its owner "as if
    /// cancelled", the unit in production and a finished one still waiting inside included: the full Space
    /// Crystals cost of each. Of their army cost (design change 0143) only the unit in production's goes, being
    /// the only one that took any ("Unit Control released for the unit in production"); a producer that waited for
    /// room frees nothing. A pool producer's entries (slice M5-F2) give their units back to the pool
    /// (`Sim::unreserve`), which come out as they are, Recruits, when the building leaves the game
    /// (`factions/cults/economy.md > Cults Building Mechanics > Evacuation on Destruction`: "Orders are cancelled with
    /// a full refund"; "a Recruit mid-training comes out as a Recruit"; `sim/holding/evacuation.rs`): all but a
    /// finished one, which the evacuation took off the queue first, its unit coming out trained (design change 0349
    /// point 3).
    pub(super) fn refund_production(&mut self, id: ObjectId) {
        let Some(o) = self.objects.get_mut(&id) else {
            return;
        };
        let def = o.def().structure.as_ref().and_then(|s| s.production.as_ref());
        let (Some(production), Some(def)) = (&mut o.production, def) else {
            return;
        };
        let owner = o.owner;
        for (entry, reserved) in production.take_queue() {
            let refund = Refund::Full.of(entry.cost(def));
            if let Some(p) = self.players.iter_mut().find(|p| Some(p.id) == owner) {
                p.credit_crystals(refund);
            }
            self.unreserve(&reserved);
            self.events.extend(cancelled(id, entry, &reserved, refund));
        }
    }
}
