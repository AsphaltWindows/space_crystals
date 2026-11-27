//! The gate's rules for `Command::Production` (slice M2-D2; `Sim::check_production`): which producers queue an entry,
//! in design change 0083's order, what X takes back, and where a rally point goes. A pool producer's entries reserve
//! their units here (`producers/pool.rs`, slice M5-F2), an Altar's augmentation orders among them (slice M5-N1b).

use super::{ProductionPlan, producer};
use crate::aircraft;
use crate::augmentation::{AugmentOrder, RecruitKind};
use crate::defs::ObjectKind;
use crate::holding;
use crate::ids::{ObjectId, PlayerId};
use crate::orders::Target;
use crate::production::{ProductionCommand, QueueEntry, Refusal, takes_rally};
use crate::sim::gate::Checked;
use crate::sim::{Reject, Sim};

/// A producer that may take a `Train`, a `TrainFrom` ([`Sim::plan_training`]) or an `Augment` ([`Sim::plan_augment`]).
struct Candidate {
    /// The work it has queued (`Production::remaining_work`), by which it is served.
    work: u64,
    id: ObjectId,
    /// The entry's price there: its Space Crystals and (M6-GD) its Supplies (`QueueEntry::cost`,
    /// `QueueEntry::supplies`).
    crystals: u32,
    supplies: u32,
    /// Why it can't take the entry whatever the bank.
    passed_over: Option<Refusal>,
    /// The units the entry would reserve.
    units: Vec<ObjectId>,
}

impl Sim {
    /// The gate's rules for `Command::Production` (`factions/gdo/structures/barracks.md >
    /// ObjectInterfaceState[Barracks]`). The producers are the player's objects among `objects` that produce
    /// units, as an order's units are found ([`Sim::own_units`]: stale and repeated ids dropped, `NotOwned` for
    /// someone else's), at work: not a site nor a building that transforms (`Object::is_operational`), whose
    /// production waits for its completion (`super::super::transform::hold_back`); none is [`Reject::NoProducers`].
    /// Then:
    /// - **Train**: at least one producer queues the unit, else why not: [`Reject::CannotProduce`] (no producer
    ///   has the recipe, or only a Recruitment Hall, which trains nothing on command: `sim/recruitment.rs`),
    ///   [`Reject::Requires`] (the player owns none of a type its entry requires, slice M4-K:
    ///   `core/tech_and_research.md > Tech Prerequisites > Unit Prerequisites`, `crate::tech`; a unit already queued
    ///   goes on whatever is lost later), [`Reject::InsufficientCrystals`], (M6-GD) [`Reject::InsufficientSupplies`],
    ///   [`Reject::QueueFull`] or (M6-GH1) [`Reject::AirfieldFull`], an aircraft at an Airfield without room for it
    ///   (`crate::aircraft`), passed over as a full queue. Queuing "needs only its Space Crystals (paid when queued) and a free queue slot; it
    ///   does **not** check or reserve Unit Control" (design change 0143, `factions/gdo/overview.md > GDOResources >
    ///   Unit Control > Reserved when production starts`; the Colonists' Beacon Capacity alike): the army cap is
    ///   checked when production starts (`producers/step.rs`). (M6-GD) A recipe with Supplies (`Recipe::supplies`)
    ///   needs them too, paid with its crystals (design changes 0435 and 0441: "deducts 100 Space Crystals and 1
    ///   Supply", "the player has sufficient Space Crystals and Supplies").
    ///   Design change 0083: the producers are served by the work they have queued, least first, ties by id, each
    ///   check counting what the ones before it in this command took, **until the crystal check fails** (0143: 0083
    ///   no longer considers the army cap); a full queue is passed over. A pool producer (`producers/pool.rs`, slice
    ///   M5-F2) takes no `Train`: its units are of a kind.
    /// - **TrainFrom** (slice M5-F2): the same, at pool producers alone, each needing the free units of the kind that
    ///   the unit is made of ([`Reject::NoRecruit`]; passed over like a full queue) but no free queue slot: its
    ///   capacity bounds its queue (design change 0347 point 5).
    /// - **Release** (slice M5-F2): a pool producer with a free unit of the kind queues it ([`Sim::plan_release`]); a
    ///   **ReleaseOf** (slice M5-N1b), one with a free unit of the type and the kind.
    /// - **Augment** (slice M5-N1b): an Altar with a free unit of the type and the kind that the order fits, and room
    ///   in its queue, served as a training is ([`Sim::plan_augment`]).
    /// - **Research** (slice M5-N3): one structure that offers the technology queues it, if T74's checks pass
    ///   (`producers/research.rs`, `Sim::plan_research`).
    /// - **CancelProduction**: a producer has something queued, else [`Reject::NothingQueued`]; never a Recruitment
    ///   Hall, whose Recruit can't be cancelled (design change 0342). A Temple's takes back its last research
    ///   (`factions/cults/structures/temple.md > Placeholder stats`: "**X: Cancel** (last research)").
    /// - **SetRally**: a point on the map ([`Reject::OffMap`]), or an object that exists and that the player sees,
    ///   as for an order's target ([`Sim::check_target_seen`]). The producer itself is always a target: it clears
    ///   the rally (design change 0086). (M5-N3) Never a research structure, which has no RallyPoint
    ///   (`factions/cults/structures/temple.md > Placeholder stats`: its panel has no Set Rally Point), its staff
    ///   walking straight out of its exit side and stopping there (`core/entities.md > Structure Type > Exit Doorway`
    ///   2): **Placeholder (not in the design: where a Temple's staff go)**. (M5-S6) Its producers are the player's
    ///   buildings of a type that has a RallyPoint ([`takes_rally`]), finished or not: an unfinished one takes it too
    ///   (design change 0396: "A building that will produce or release units can have its RallyPoint set while it is
    ///   still under construction or transformation, for every faction"); [`Reject::NoProducers`] if there is none.
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
        let producers = match command {
            ProductionCommand::SetRally(_) => self.own_units(player, objects, |o| takes_rally(o.kind))?,
            _ => self.own_units(player, objects, |o| o.is_operational() && producer(o).is_some())?,
        };
        if producers.is_empty() {
            return Err(Reject::NoProducers);
        }
        let plan = match command {
            ProductionCommand::Train(kind) => self.plan_training(player, &producers, kind, None)?,
            ProductionCommand::TrainFrom { kind, from } => self.plan_training(player, &producers, kind, Some(from))?,
            ProductionCommand::Release(from) => self.plan_release(&producers, None, from)?,
            ProductionCommand::ReleaseOf { kind, from } => self.plan_release(&producers, Some(kind), from)?,
            ProductionCommand::Augment { kind, from, order } => {
                self.plan_augment(player, &producers, kind, from, order)?
            }
            ProductionCommand::Research(tech) => self.plan_research(player, &producers, tech)?,
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
            ProductionCommand::SetRally(target) => {
                match target {
                    // `contains_point` also rejects NaN and infinite positions.
                    Target::Point(p) if !self.map.contains_point(p) => return Err(Reject::OffMap),
                    // Design change 0139: on an object the player sees or remembers (`Sim::check_target_known`).
                    Target::Object(id) if producers != [id] => {
                        self.check_target_known(player, id)?;
                    }
                    Target::Point(_) | Target::Object(_) => {}
                }
                ProductionPlan::SetRally(producers, target)
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
        let able: Vec<Candidate> = producers
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
                    // (M6-GH1) An aircraft at an Airfield without room for it (`crate::aircraft::room`; `factions/gdo/
                    // structures/airfield.md > Production limit`: "E.g. 3 assigned + 1 queued = full"), passed over as
                    // a full queue.
                    None => {
                        let refusal = (production.is_full().then_some(Refusal::QueueFull))
                            .or_else(|| self.airfield_full(o, kind).then_some(Refusal::AirfieldFull));
                        (refusal, Vec::new())
                    }
                };
                Some(Candidate {
                    work: production.remaining_work(def),
                    id,
                    crystals: recipe.cost,
                    supplies: recipe.supplies,
                    passed_over,
                    units,
                })
            })
            .collect();
        if able.is_empty() {
            return Err(unmet.map_or(Reject::CannotProduce(kind), Reject::Requires));
        }
        self.serve(player, entry, able, from)
    }

    /// (M6-GH1) Whether a unit of `kind` is an aircraft that the Airfield `o` has no room for (`crate::aircraft::room`:
    /// its assignments and the aircraft in its queue at its capacity).
    fn airfield_full(&self, o: &crate::object::Object, kind: ObjectKind) -> bool {
        aircraft::def_of(kind).is_some()
            && aircraft::airfield_def(o).is_some()
            && aircraft::room(&self.objects, o.id) == 0
    }

    /// (M5-N1b) Which of `producers` queue `order` for `player`, each for its free unit of type `kind` and kind `from`
    /// that entered first (`factions/cults/augmentation.md > Altar > ObjectInterfaceState[Altar] > AugmentMenu[UnitType,
    /// State]`: "Available only if the group still has an unreserved unit, the AugmentationQueue is not full, and the
    /// player has sufficient Space Crystals"). [`Reject::AlreadyAugmented`] for units the order doesn't fit ("An order
    /// can only target a unit that is missing every Augmentation the order applies"), [`Reject::CannotAugment`] if none
    /// of the producers takes the order; then they are served as for a training ([`Sim::serve`], design change 0083),
    /// one with a full queue ([`Reject::QueueFull`]) or without such a unit ([`Reject::NoRecruit`]) passed over. The
    /// queue is full with as many entries as the building holds units (`HoldsDef::capacity`): "bounded by
    /// InternalUnitCapacity, reserved units included, Q0104". Each entry reserving a unit inside, a full queue has no
    /// free unit left either, and it is the refusal given. **Placeholder (not in the design: which of the two
    /// refusals)**; the "nominal max 20" is above the capacity, so never reached.
    fn plan_augment(
        &self,
        player: PlayerId,
        producers: &[ObjectId],
        kind: ObjectKind,
        from: RecruitKind,
        order: AugmentOrder,
    ) -> Result<ProductionPlan, Reject> {
        if !order.fits(from) {
            return Err(Reject::AlreadyAugmented(from));
        }
        let able: Vec<Candidate> = producers
            .iter()
            .filter_map(|&id| {
                let o = &self.objects[&id];
                let (production, def) = producer(o)?;
                let (recipe, holds) = (def.augment_recipe(order)?, holding::holds(o)?);
                let mut units = self.free_units_of(id, kind, from);
                units.truncate(1);
                let passed_over = if production.queue.len() >= holds.capacity as usize {
                    Some(Refusal::QueueFull)
                } else {
                    units.is_empty().then_some(Refusal::NoRecruit)
                };
                Some(Candidate {
                    work: production.remaining_work(def),
                    id,
                    crystals: recipe.cost,
                    supplies: 0,
                    passed_over,
                    units,
                })
            })
            .collect();
        if able.is_empty() {
            return Err(Reject::CannotAugment(order));
        }
        self.serve(player, QueueEntry::Augment { order }, able, Some(from))
    }

    /// Design change 0083's serving of `able`, the producers that may queue `entry` for `player`, which may take units of
    /// kind `from` ([`Sim::plan_training`], [`Sim::plan_augment`]): by the work they have queued, least first, ties by id,
    /// each paying out of what the ones before it in this command left, **until the check of the bank fails**: its Space
    /// Crystals, then (M6-GD) its Supplies, both paid when the entry is queued ("crystals or any other resource paid at
    /// queue time", `systems/controls/selection.md > Group production with limited resources`; "both are required"). One
    /// passed over refuses it whatever the bank. If none takes it, the first reason: the bank's before a passed over
    /// one's, the crystals' before the Supplies' when both are short.
    fn serve(
        &self,
        player: PlayerId,
        entry: QueueEntry,
        mut able: Vec<Candidate>,
        from: Option<RecruitKind>,
    ) -> Result<ProductionPlan, Reject> {
        able.sort_unstable_by_key(|candidate| (candidate.work, candidate.id));
        let bank = self.player(player);
        let mut crystals = bank.map_or(0, |p| p.crystals);
        let mut supplies = bank.map_or(0, |p| p.supplies);
        let (mut accepted, mut refused) = (Vec::new(), Vec::new());
        let (mut stopped, mut passed): (Option<Refusal>, Option<Refusal>) = (None, None);
        for candidate in able {
            let id = candidate.id;
            if let Some(reason) = candidate.passed_over {
                passed.get_or_insert(reason);
                refused.push((id, reason));
                continue;
            }
            if stopped.is_none() && crystals < candidate.crystals {
                stopped = Some(Refusal::Crystals);
            }
            if stopped.is_none() && supplies < candidate.supplies {
                stopped = Some(Refusal::Supplies);
            }
            match stopped {
                Some(reason) => refused.push((id, reason)),
                None => {
                    crystals -= candidate.crystals;
                    supplies -= candidate.supplies;
                    accepted.push((id, candidate.units));
                }
            }
        }
        if accepted.is_empty() {
            return Err(match stopped.or(passed) {
                Some(Refusal::Crystals) => Reject::InsufficientCrystals,
                Some(Refusal::Supplies) => Reject::InsufficientSupplies,
                Some(Refusal::NoRecruit) => from.map_or(Reject::QueueFull, Reject::NoRecruit),
                Some(Refusal::AirfieldFull) => Reject::AirfieldFull,
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
    /// [`Reject::NoHolders`] if none of them holds units, [`Reject::NoRecruit`] if none has such a unit free. (M5-N1b)
    /// With a `unit` type, an Altar's (`ProductionCommand::ReleaseOf`), of that type too ([`Sim::free_units_of`]).
    fn plan_release(
        &self,
        producers: &[ObjectId],
        unit: Option<ObjectKind>,
        from: RecruitKind,
    ) -> Result<ProductionPlan, Reject> {
        let pools: Vec<ObjectId> = (producers.iter().copied())
            .filter(|id| holding::trains(self.objects[id].kind))
            .collect();
        if pools.is_empty() {
            return Err(Reject::NoHolders);
        }
        let free = |id| match unit {
            Some(unit) => self.free_units_of(id, unit, from),
            None => self.free_units(id, from),
        };
        let accepted: Vec<(ObjectId, Vec<ObjectId>)> = (pools.into_iter())
            .filter_map(|id| Some((id, vec![*free(id).first()?])))
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
