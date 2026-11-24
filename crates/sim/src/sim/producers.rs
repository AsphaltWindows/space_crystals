//! Production in the sim (slice M2-D2; `crate::production` has the state and the rules it runs): the gate and the
//! application of `Command::Production`, the producers' step of the economy phase, and the refund when a producer
//! leaves the game.

use glam::Vec2;

use super::gate::Checked;
use super::{Reject, Sim};
use crate::defs::{ObjectKind, ProductionDef, UnitDef};
use crate::economy::{self, Refund};
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::math;
use crate::nav;
use crate::object::Object;
use crate::orders::Target;
use crate::paths::boxes_overlap;
use crate::production::{Doorway, Production, ProductionCommand, ProductionStatus, Rally, Refusal, rally};

/// When something stands right in front of the doorway, the released unit stands this far from it, space units,
/// so that no rounding makes the two boxes overlap (a technical margin: a sixty-fourth of a space unit).
const SPOT_CLEARANCE: f32 = 1.0 / 64.0;

/// A `Command::Production` that passed the gate, with what the checks resolved.
pub(super) enum ProductionPlan {
    /// `accepted` queue the unit and pay, in this order (design change 0083); `refused` don't.
    Train {
        kind: ObjectKind,
        accepted: Vec<ObjectId>,
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

impl Sim {
    /// What producer `id` is working on (`factions/gdo/structures.md > Barracks > BarracksInstanceState`:
    /// CurrentBuild and CurrentBuildProgress), for the interface: `None` for an object that produces nothing or has
    /// an empty queue. `waiting` says the unit is finished and held inside because the exit doorway is occupied
    /// (design change 0085: shown at 100%).
    pub fn production_status(&self, id: ObjectId) -> Option<ProductionStatus> {
        let (production, def) = producer(self.objects.get(&id)?)?;
        production.status(def)
    }

    /// The gate's rules for `Command::Production` (`factions/gdo/structures.md > Barracks >
    /// ObjectInterfaceState[Barracks]`). The producers are the player's objects among `objects` that produce
    /// units, as an order's units are found ([`Sim::own_units`]: stale and repeated ids dropped, `NotOwned` for
    /// someone else's); none is [`Reject::NoProducers`]. Then:
    /// - **Train**: at least one producer queues the unit, else why not: [`Reject::CannotProduce`] (no producer
    ///   has the recipe), [`Reject::InsufficientCrystals`], [`Reject::UnitControlCap`] (the army cap of the player's
    ///   faction, plan decision T20: [`Sim::army_cap`]) or [`Reject::QueueFull`].
    ///   Design change 0083: the producers are served by the work they have queued, least first, ties by id, each
    ///   check counting what the ones before it in this command took, **until a cost check fails**; a full queue
    ///   is passed over.
    /// - **CancelProduction**: a producer has something queued, else [`Reject::NothingQueued`].
    /// - **SetRally**: a point on the map ([`Reject::OffMap`]), or an object that exists and that the player sees,
    ///   as for an order's target ([`Sim::check_target_seen`]). The producer itself is always a target: it clears
    ///   the rally (design change 0086).
    ///
    ///   **Placeholder** (design change 0088: "Rallying on a remembered (fogged) enemy structure: TBD with
    ///   Director question Q0017 point 2"): an object out of sight that the player remembers becomes a ground
    ///   rally at its last-known position, the centre of its record's footprint.
    pub(super) fn check_production(
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
            ProductionCommand::Train(kind) => self.plan_training(player, &producers, kind)?,
            ProductionCommand::CancelProduction => {
                let queued: Vec<ObjectId> = producers
                    .into_iter()
                    .filter(|id| {
                        self.objects[id]
                            .production
                            .as_ref()
                            .is_some_and(|p| !p.queue.is_empty())
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
            ProductionCommand::SetRally(Target::Object(id)) => {
                let target = self.referenced(id, None)?;
                let itself = producers == [id];
                let target = match self.check_target_seen(player, target) {
                    _ if itself => Target::Object(id),
                    Ok(()) => Target::Object(id),
                    Err(unseen) => {
                        let record = self.vision.player(player).and_then(|v| v.remembered_record(id));
                        Target::Point(record.ok_or(unseen)?.placement.center())
                    }
                };
                ProductionPlan::SetRally(producers, target)
            }
        };
        Ok(Checked::Production { player, plan })
    }

    /// Design change 0083 (`systems/controls/selection_and_hud.md > SelectionGroup > No smart-casting > Group
    /// production with limited resources`): which of `producers` queue a unit of `kind` for `player`.
    fn plan_training(
        &self,
        player: PlayerId,
        producers: &[ObjectId],
        kind: ObjectKind,
    ) -> Result<ProductionPlan, Reject> {
        // (work queued, id, the unit's cost, whether the queue is full), least work first, ties by id.
        let mut able: Vec<(u64, ObjectId, u32, bool)> = producers
            .iter()
            .filter_map(|&id| {
                let (production, def) = producer(&self.objects[&id])?;
                let recipe = def.recipe(kind)?;
                Some((production.remaining_work(def), id, recipe.cost, production.is_full()))
            })
            .collect();
        if able.is_empty() {
            return Err(Reject::CannotProduce(kind));
        }
        able.sort_unstable_by_key(|&(work, id, ..)| (work, id));
        let mut crystals = self.player(player).map_or(0, |p| p.crystals);
        // Plan decision T20: the army cap of the player's faction, if it has one. Design change 0082: the units
        // queued count already, these included as they are queued.
        let mut army = self.army_cap(player);
        let army_cost = army.map_or(0, |cap| cap.resource.cost_of(kind));
        let (mut accepted, mut refused) = (Vec::new(), Vec::new());
        let mut stopped: Option<Refusal> = None;
        for (_, id, cost, full) in able {
            if full {
                refused.push((id, Refusal::QueueFull));
                continue;
            }
            if stopped.is_none() {
                if crystals < cost {
                    stopped = Some(Refusal::Crystals);
                } else if army.is_some_and(|cap| !cap.fits(army_cost)) {
                    stopped = Some(Refusal::UnitControl);
                }
            }
            match stopped {
                Some(reason) => refused.push((id, reason)),
                None => {
                    crystals -= cost;
                    if let Some(cap) = &mut army {
                        cap.used = cap.used.saturating_add(army_cost);
                    }
                    accepted.push(id);
                }
            }
        }
        if accepted.is_empty() {
            return Err(match stopped {
                Some(Refusal::Crystals) => Reject::InsufficientCrystals,
                Some(Refusal::UnitControl) => Reject::UnitControlCap,
                Some(Refusal::QueueFull) | None => Reject::QueueFull,
            });
        }
        Ok(ProductionPlan::Train {
            kind,
            accepted,
            refused,
        })
    }

    /// Applies a checked `Command::Production` of `player`.
    pub(super) fn apply_production(&mut self, player: PlayerId, plan: ProductionPlan) {
        match plan {
            ProductionPlan::Train {
                kind,
                accepted,
                refused,
            } => {
                for id in accepted {
                    let Some(cost) = self
                        .objects
                        .get(&id)
                        .and_then(producer)
                        .and_then(|(_, def)| def.recipe(kind))
                        .map(|recipe| recipe.cost)
                    else {
                        continue;
                    };
                    // "deducts 50 Space Crystals from player, adds Peacekeeper to BuildQueue" (the gate checked
                    // the bank, each producer after what the ones before it took).
                    let paid = self
                        .players
                        .iter_mut()
                        .find(|p| p.id == player)
                        .is_some_and(|p| p.spend_crystals(cost));
                    if !paid {
                        continue;
                    }
                    if let Some(production) = self.objects.get_mut(&id).and_then(|o| o.production.as_mut()) {
                        production.queue.push(kind);
                        self.events.push(SimEvent::ProductionQueued { producer: id, kind });
                    }
                }
                self.events
                    .extend(refused.into_iter().map(|(id, reason)| SimEvent::ProductionRefused {
                        producer: id,
                        kind,
                        reason,
                    }));
            }
            ProductionPlan::Cancel(producers) => {
                for id in producers {
                    let Some(o) = self.objects.get_mut(&id) else {
                        continue;
                    };
                    let def = o.def().structure.as_ref().and_then(|s| s.production.as_ref());
                    let Some(kind) = o.production.as_mut().and_then(Production::cancel_last) else {
                        continue;
                    };
                    // "removes last entry from BuildQueue, refunds full cost to player"; its Unit Control goes
                    // with the entry (design change 0082).
                    let cost = def.and_then(|def| def.recipe(kind)).map_or(0, |recipe| recipe.cost);
                    let refund = Refund::Full.of(cost);
                    if let Some(p) = self.players.iter_mut().find(|p| p.id == player) {
                        p.credit_crystals(refund);
                    }
                    self.events.push(SimEvent::ProductionCancelled {
                        producer: id,
                        kind,
                        refund,
                    });
                }
            }
            ProductionPlan::SetRally(producers, target) => {
                for id in producers {
                    let rally = match target {
                        Target::Point(p) => Some(Rally::Point(p)),
                        // Design change 0086: "Setting a production building's rally point on the building itself
                        // clears it (RallyPoint = None)."
                        Target::Object(target) if target == id => None,
                        Target::Object(target) => self
                            .objects
                            .get(&target)
                            .map(|o| Rally::Object { id: target, at: o.pos }),
                    };
                    if let Some(production) = self.objects.get_mut(&id).and_then(|o| o.production.as_mut()) {
                        production.rally = rally;
                    }
                }
            }
        }
    }

    /// `factions/gdo/structures.md > Barracks > Destroyed while producing` (design changes 0084 and 0085): object
    /// `id` is about to leave the game. If it produces units, its whole BuildQueue is refunded to its owner "as if
    /// cancelled", the unit in production and a finished one still waiting inside included: the full Space
    /// Crystals cost of each. Their Unit Control is released with the queue.
    pub(super) fn refund_production(&mut self, id: ObjectId) {
        let Some(o) = self.objects.get_mut(&id) else {
            return;
        };
        let def = o.def().structure.as_ref().and_then(|s| s.production.as_ref());
        let (Some(production), Some(def)) = (&mut o.production, def) else {
            return;
        };
        let owner = o.owner;
        for kind in std::mem::take(&mut production.queue) {
            let refund = Refund::Full.of(def.recipe(kind).map_or(0, |recipe| recipe.cost));
            if let Some(p) = self.players.iter_mut().find(|p| Some(p.id) == owner) {
                p.credit_crystals(refund);
            }
            self.events.push(SimEvent::ProductionCancelled {
                producer: id,
                kind,
                refund,
            });
        }
        production.progress = 0;
    }

    /// The producers' step of the economy phase (plan decision T5; `ARCHITECTURE.md > Tick pipeline`), in id order:
    /// the unit in production advances by its owner's power ratio (design change 0089: "unit production speed" is
    /// slowed at negative power), and a finished one is released through the exit doorway, or held while that is
    /// occupied ([`Sim::release_unit`]). Producers from `first_new` on were created this tick and wait for the
    /// next. The released units are in the game from this tick, and act from the next (the spawn-tick rule).
    pub(super) fn step_production(&mut self, first_new: ObjectId) {
        let busy: Vec<ObjectId> = self
            .objects
            .range(..first_new)
            .filter(|(_, o)| o.production.as_ref().is_some_and(|p| !p.queue.is_empty()))
            .map(|(&id, _)| id)
            .collect();
        if busy.is_empty() {
            return;
        }
        let power = economy::power::summaries(self.objects.values());
        for id in busy {
            let Some(o) = self.objects.get_mut(&id) else {
                continue;
            };
            let def = o.def().structure.as_ref().and_then(|s| s.production.as_ref());
            let (Some(owner), Some(def), Some(production)) = (o.owner, def, &mut o.production) else {
                continue;
            };
            let speed = power.get(&owner).copied().unwrap_or_default().speed(o.kind);
            if production.advance(def, speed) {
                self.release_unit(id);
            }
        }
    }

    /// `core/entities.md > Structure Type > Exit Doorway` (design change 0080): `producer`'s finished unit comes
    /// into the game in its exit doorway, facing out, with the order its rally point gives
    /// (`> Barracks > RallyPoint behavior`); or it stays inside, finished, while the doorway is occupied: "the
    /// next unit can only appear once the doorway is free. While a unit still stands in it, the building pauses".
    /// The doorway is occupied while a unit this building released still overlaps its footprint (`Mover::exit`).
    fn release_unit(&mut self, producer: ObjectId) {
        let Some(building) = self.objects.get(&producer) else {
            return;
        };
        let (Some(door), Some(owner)) = (Doorway::of(building), building.owner) else {
            return;
        };
        let Some(production) = &building.production else {
            return;
        };
        let (Some(kind), rally) = (production.current(), production.rally) else {
            return;
        };
        let def = kind.def();
        let (Some(unit), Some(half)) = (&def.unit, def.half_extents()) else {
            return;
        };
        let occupied = self
            .objects
            .values()
            .any(|o| o.mover.as_ref().is_some_and(|mover| mover.exit == Some(producer)));
        if occupied || self.next_id == u32::MAX {
            return;
        }
        let Some(pos) = self.doorway_spot(&door, unit, half) else {
            return;
        };
        if let Some(production) = self.objects.get_mut(&producer).and_then(|o| o.production.as_mut()) {
            production.release();
        }
        let id = self.insert_object(kind, Some(owner), pos, None, None);
        if let Some(o) = self.objects.get_mut(&id) {
            o.facing = math::vector_angle(door.outward());
            o.prev_facing = o.facing;
            if let Some(mover) = &mut o.mover {
                mover.exit = door.holds(pos, half).then_some(producer);
            }
        }
        self.events.push(SimEvent::UnitProduced { producer, unit: id });
        let (armed, worker) = (unit.attack.is_some(), unit.worker.is_some());
        let order = rally.and_then(|rally| rally::order_for(&rally, Some(owner), armed, worker, &self.objects));
        if let Some(kind) = order {
            self.give_order(&[id], kind, false);
        }
    }

    /// Where a unit with half extents `half` appears in `door`: "half inside the building, at the middle of the
    /// exit side", or as far out as there is room when another unit or a blocked tile is right in front of the
    /// doorway (fully inside, against the exit side, at most: see `production::exit` for this reading of the
    /// design). Its box overlaps no unit of its layer and no blocked tile but the building's own. `None` if there
    /// is no such spot.
    fn doorway_spot(&self, door: &Doorway, unit: &UnitDef, half: Vec2) -> Option<Vec2> {
        let base = unit.base_def();
        let grid = base
            .nav_class()
            .and_then(|class| self.nav.nav().grid(class, nav::size_class(half * 2.0)));
        let clear = |p: Vec2| grid.is_none_or(|grid| door.box_clear(grid, half, p));
        let depth = door.outward().abs().dot(half);
        // The other units of its layer, with their half extents.
        let others = || {
            self.objects.values().filter_map(|o| {
                let other = o.def();
                let same_layer = base.domain.hard_collision()
                    && other.unit.as_ref().is_some_and(|u| u.base_def().domain == base.domain);
                Some((o.pos, other.half_extents().filter(|_| same_layer)?))
            })
        };
        let mut out: f32 = if clear(door.spot(0.0)) { 0.0 } else { -depth };
        for (pos, other_half) in others() {
            if let Some(room) = door.room_before(half, pos, other_half) {
                out = out.min(room - SPOT_CLEARANCE);
            }
        }
        // Not deeper than fully inside, the clearance apart.
        if out < -depth - 2.0 * SPOT_CLEARANCE {
            return None;
        }
        let spot = door.spot(out);
        let free = clear(spot) && others().all(|(pos, other_half)| !boxes_overlap(spot, half, pos, other_half));
        free.then_some(spot)
    }
}
