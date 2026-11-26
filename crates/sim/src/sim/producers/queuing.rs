//! The gate's rules for `Command::Production` (slice M2-D2; `Sim::check_production`): which producers queue an entry,
//! in design change 0083's order, what X takes back, and where a rally point goes. A pool producer's entries reserve
//! their units here (`producers/pool.rs`, slice M5-F2).

use super::{ProductionPlan, producer};
use crate::augmentation::RecruitKind;
use crate::defs::ObjectKind;
use crate::holding;
use crate::ids::{ObjectId, PlayerId};
use crate::orders::Target;
use crate::production::{ProductionCommand, QueueEntry, Refusal};
use crate::sim::gate::Checked;
use crate::sim::{Reject, Sim};

/// A producer that may take a `Train` or a `TrainFrom` ([`Sim::plan_training`]): the work it has queued, its id, the
/// unit's cost, why it can't take the unit whatever the bank, and the units the entry would reserve.
type Candidate = (u64, ObjectId, u32, Option<Refusal>, Vec<ObjectId>);

impl Sim {
    /// The gate's rules for `Command::Production` (`factions/gdo/structures/barracks.md >
    /// ObjectInterfaceState[Barracks]`). The producers are the player's objects among `objects` that produce
    /// units, as an order's units are found ([`Sim::own_units`]: stale and repeated ids dropped, `NotOwned` for
    /// someone else's); none is [`Reject::NoProducers`]. Then:
    /// - **Train**: at least one producer queues the unit, else why not: [`Reject::CannotProduce`] (no producer
    ///   has the recipe, or only a Recruitment Hall, which trains nothing on command: `sim/recruitment.rs`),
    ///   [`Reject::Requires`] (the player owns none of a type its entry requires, slice M4-K:
    ///   `core/tech_and_research.md > Tech Prerequisites > Unit Prerequisites`, `crate::tech`; a unit already queued
    ///   goes on whatever is lost later), [`Reject::InsufficientCrystals`] or [`Reject::QueueFull`]. Queuing "needs
    ///   only its Space Crystals (paid when queued) and a free queue slot; it does **not** check or reserve Unit
    ///   Control" (design change 0143, `factions/gdo/overview.md > GDOResources > Unit Control > Reserved when
    ///   production starts`; the Colonists' Beacon Capacity alike): the army cap is checked when production starts
    ///   (`producers/step.rs`).
    ///   Design change 0083: the producers are served by the work they have queued, least first, ties by id, each
    ///   check counting what the ones before it in this command took, **until the crystal check fails** (0143: 0083
    ///   no longer considers the army cap); a full queue is passed over. A pool producer (`producers/pool.rs`, slice
    ///   M5-F2) takes no `Train`: its units are of a kind.
    /// - **TrainFrom** (slice M5-F2): the same, at pool producers alone, each needing the free units of the kind that
    ///   the unit is made of ([`Reject::NoRecruit`]; passed over like a full queue) but no free queue slot: its
    ///   capacity bounds its queue (design change 0347 point 5).
    /// - **Release** (slice M5-F2): a pool producer with a free unit of the kind queues it ([`Sim::plan_release`]).
    /// - **CancelProduction**: a producer has something queued, else [`Reject::NothingQueued`]; never a Recruitment
    ///   Hall, whose Recruit can't be cancelled (design change 0342).
    /// - **SetRally**: a point on the map ([`Reject::OffMap`]), or an object that exists and that the player sees,
    ///   as for an order's target ([`Sim::check_target_seen`]). The producer itself is always a target: it clears
    ///   the rally (design change 0086).
    ///
    ///   **Placeholder** (design change 0088: "Rallying on a remembered (fogged) enemy structure: TBD with
    ///   Director question Q0017 point 2"): an object out of sight that the player remembers becomes a ground
    ///   rally at its last-known position, the centre of its record's footprint.
    pub(in crate::sim) fn check_production(
        &self,
        player: PlayerId,
        objects: &[ObjectId],
        command: ProductionCommand,
    ) -> Result<Checked, Reject> {
        let producers = self.own_units(player, objects, |o| producer(o).is_some())?;
        if producers.is_empty() {
            return Err(Reject::NoProducers);
        }
        let plan = match command {
            ProductionCommand::Train(kind) => self.plan_training(player, &producers, kind, None)?,
            ProductionCommand::TrainFrom { kind, from } => self.plan_training(player, &producers, kind, Some(from))?,
            ProductionCommand::Release(from) => self.plan_release(&producers, from)?,
            ProductionCommand::CancelProduction => {
                // A Recruitment Hall's Recruit can't be cancelled (design change 0342: "Recruits can't be cancelled").
                let queued: Vec<ObjectId> = producers
                    .into_iter()
                    .filter(|id| {
                        producer(&self.objects[id])
                            .is_some_and(|(p, def)| def.recruitment.is_none() && !p.queue.is_empty())
                    })
                    .collect();
                if queued.is_empty() {
                    return Err(Reject::NothingQueued);
                }
                ProductionPlan::Cancel(queued)
            }
            ProductionCommand::SetRally(Target::Point(p)) => {
                // `contains_point` also rejects NaN and infinite positions.
                if !self.map.contains_point(p) {
                    return Err(Reject::OffMap);
                }
                ProductionPlan::SetRally(producers, Target::Point(p))
            }
            // Design change 0139: on an object the player sees or remembers (`Sim::check_target_known`).
            ProductionCommand::SetRally(Target::Object(id)) => {
                if producers != [id] {
                    self.check_target_known(player, id)?;
                }
                ProductionPlan::SetRally(producers, Target::Object(id))
            }
        };
        Ok(Checked::Production { player, plan })
    }

    /// Design change 0083 (`systems/controls/selection.md > SelectionGroup > No smart-casting > Group
    /// production with limited resources`): which of `producers` queue a unit of `kind` for `player`: made of their
    /// free units of kind `from` at pool producers (`ProductionCommand::TrainFrom`, slice M5-F2, `producers/pool.rs`),
    /// or at the others with no `from` (`ProductionCommand::Train`), each taking only its own.
    fn plan_training(
        &self,
        player: PlayerId,
        producers: &[ObjectId],
        kind: ObjectKind,
        from: Option<RecruitKind>,
    ) -> Result<ProductionPlan, Reject> {
        // Least work first, ties by id. A producer whose entry requires a type the player doesn't own queues nothing
        // (`core/tech_and_research.md > Tech Prerequisites > Unit Prerequisites`: "checked only when the unit is
        // queued"; slice M4-K, `crate::tech`).
        let entry = QueueEntry::Unit(kind);
        let mut unmet = None;
        let mut able: Vec<Candidate> = producers
            .iter()
            .filter_map(|&id| {
                let o = &self.objects[&id];
                let (production, def) = producer(o)?;
                // A Recruitment Hall recruits by itself (`ProductionDef::recruitment`, `sim/recruitment.rs`); a pool
                // producer trains of its units, of a kind.
                let recipe = def.recipe(kind).filter(|_| def.recruitment.is_none())?;
                if holding::trains(o.kind) != from.is_some() {
                    return None;
                }
                if let Some(missing) = self.unmet_requirement(player, recipe.requires.iter().copied()) {
                    unmet.get_or_insert(missing);
                    return None;
                }
                let (passed_over, units) = match from {
                    // Its capacity bounds its queue, not `QUEUE_LIMIT` (design change 0347 point 5).
                    Some(from) => {
                        let mut units = self.free_units(id, from);
                        let needed = entry.units_reserved();
                        let short = units.len() < needed;
                        units.truncate(needed);
                        (short.then_some(Refusal::NoRecruit), units)
                    }
                    None => (production.is_full().then_some(Refusal::QueueFull), Vec::new()),
                };
                Some((production.remaining_work(def), id, recipe.cost, passed_over, units))
            })
            .collect();
        if able.is_empty() {
            return Err(unmet.map_or(Reject::CannotProduce(kind), Reject::Requires));
        }
        able.sort_unstable_by_key(|&(work, id, ..)| (work, id));
        let mut crystals = self.player(player).map_or(0, |p| p.crystals);
        let (mut accepted, mut refused) = (Vec::new(), Vec::new());
        let (mut stopped, mut passed): (Option<Refusal>, Option<Refusal>) = (None, None);
        for (_, id, cost, passed_over, units) in able {
            if let Some(reason) = passed_over {
                passed.get_or_insert(reason);
                refused.push((id, reason));
                continue;
            }
            if stopped.is_none() && crystals < cost {
                stopped = Some(Refusal::Crystals);
            }
            match stopped {
                Some(reason) => refused.push((id, reason)),
                None => {
                    crystals -= cost;
                    accepted.push((id, units));
                }
            }
        }
        if accepted.is_empty() {
            return Err(match stopped.or(passed) {
                Some(Refusal::Crystals) => Reject::InsufficientCrystals,
                Some(Refusal::NoRecruit) => from.map_or(Reject::QueueFull, Reject::NoRecruit),
                Some(Refusal::QueueFull) | None => Reject::QueueFull,
            });
        }
        Ok(ProductionPlan::Train {
            entry,
            accepted,
            refused,
        })
    }

    /// (M5-F2) Which of `producers` queue a Release of a unit of kind `from` (design change 0337,
    /// `factions/cults/structures/README.md > Choosing Recruits, Release and Eject All`): every pool producer with a
    /// free unit of that kind, in id order, each reserving its one that entered first. A Release costs nothing, so
    /// those without such a unit pass silently: no "insufficient resources" to tell (design change 0083's feedback),
    /// and the panel sends it to one building, the Ungroupable one Tabbed to (design change 0348 point 5).
    /// [`Reject::NoHolders`] if none of them holds units, [`Reject::NoRecruit`] if none has such a unit free.
    fn plan_release(&self, producers: &[ObjectId], from: RecruitKind) -> Result<ProductionPlan, Reject> {
        let pools: Vec<ObjectId> = (producers.iter().copied())
            .filter(|id| holding::trains(self.objects[id].kind))
            .collect();
        if pools.is_empty() {
            return Err(Reject::NoHolders);
        }
        let accepted: Vec<(ObjectId, Vec<ObjectId>)> = (pools.into_iter())
            .filter_map(|id| Some((id, vec![*self.free_units(id, from).first()?])))
            .collect();
        if accepted.is_empty() {
            return Err(Reject::NoRecruit(from));
        }
        Ok(ProductionPlan::Train {
            entry: QueueEntry::Release,
            accepted,
            refused: Vec::new(),
        })
    }
}
