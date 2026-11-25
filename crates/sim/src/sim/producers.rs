//! Production in the sim (slice M2-D2; `crate::production` has the state and the rules it runs): the gate and the
//! application of `Command::Production`, the producers' step of the economy phase (`producers/step.rs`, with the
//! start check of design change 0143), the release of a finished unit, and the refund when a producer leaves the
//! game.

mod step;

use glam::Vec2;

use super::gate::Checked;
use super::{Reject, Sim};
use crate::defs::{ExitRule, ObjectKind, ProductionDef, UnitDef};
use crate::economy::{ArmyResource, Refund};
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::math;
use crate::nav;
use crate::object::Object;
use crate::orders::Target;
use crate::paths::boxes_overlap;
use crate::production::{Door, ExitDoor, Production, ProductionCommand, ProductionStatus, Rally, Refusal};

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

/// What `o`'s type produces, if anything.
fn producer_def(o: &Object) -> Option<&'static ProductionDef> {
    o.def().structure.as_ref()?.production.as_ref()
}

/// A producer and what its type produces.
fn producer(o: &Object) -> Option<(&Production, &'static ProductionDef)> {
    let def = o.def().structure.as_ref()?.production.as_ref()?;
    Some((o.production.as_ref()?, def))
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
        production.status(def, faction.and_then(ArmyResource::of))
    }

    /// The gate's rules for `Command::Production` (`factions/gdo/structures/barracks.md >
    /// ObjectInterfaceState[Barracks]`). The producers are the player's objects among `objects` that produce
    /// units, as an order's units are found ([`Sim::own_units`]: stale and repeated ids dropped, `NotOwned` for
    /// someone else's); none is [`Reject::NoProducers`]. Then:
    /// - **Train**: at least one producer queues the unit, else why not: [`Reject::CannotProduce`] (no producer
    ///   has the recipe), [`Reject::Requires`] (the player owns none of a type its entry requires, slice M4-K:
    ///   `core/tech_and_research.md > Tech Prerequisites > Unit Prerequisites`, `crate::tech`; a unit already queued
    ///   goes on whatever is lost later), [`Reject::InsufficientCrystals`] or [`Reject::QueueFull`]. Queuing "needs
    ///   only its Space Crystals (paid when queued) and a free queue slot; it does **not** check or reserve Unit
    ///   Control" (design change 0143, `factions/gdo/overview.md > GDOResources > Unit Control > Reserved when
    ///   production starts`; the Colonists' Beacon Capacity alike): the army cap is checked when production starts
    ///   (`producers/step.rs`).
    ///   Design change 0083: the producers are served by the work they have queued, least first, ties by id, each
    ///   check counting what the ones before it in this command took, **until the crystal check fails** (0143: 0083
    ///   no longer considers the army cap); a full queue is passed over.
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

    /// Design change 0083 (`systems/controls/selection_and_hud.md > SelectionGroup > No smart-casting > Group
    /// production with limited resources`): which of `producers` queue a unit of `kind` for `player`.
    fn plan_training(
        &self,
        player: PlayerId,
        producers: &[ObjectId],
        kind: ObjectKind,
    ) -> Result<ProductionPlan, Reject> {
        // (work queued, id, the unit's cost, whether the queue is full), least work first, ties by id. A producer whose
        // entry requires a type the player doesn't own queues nothing (`core/tech_and_research.md > Tech Prerequisites
        // > Unit Prerequisites`: "checked only when the unit is queued"; slice M4-K, `crate::tech`).
        let mut unmet = None;
        let mut able: Vec<(u64, ObjectId, u32, bool)> = producers
            .iter()
            .filter_map(|&id| {
                let (production, def) = producer(&self.objects[&id])?;
                let recipe = def.recipe(kind)?;
                if let Some(missing) = self.unmet_requirement(player, recipe.requires.iter().copied()) {
                    unmet.get_or_insert(missing);
                    return None;
                }
                Some((production.remaining_work(def), id, recipe.cost, production.is_full()))
            })
            .collect();
        if able.is_empty() {
            return Err(unmet.map_or(Reject::CannotProduce(kind), Reject::Requires));
        }
        able.sort_unstable_by_key(|&(work, id, ..)| (work, id));
        let mut crystals = self.player(player).map_or(0, |p| p.crystals);
        let (mut accepted, mut refused) = (Vec::new(), Vec::new());
        let mut stopped: Option<Refusal> = None;
        for (_, id, cost, full) in able {
            if full {
                refused.push((id, Refusal::QueueFull));
                continue;
            }
            if stopped.is_none() && crystals < cost {
                stopped = Some(Refusal::Crystals);
            }
            match stopped {
                Some(reason) => refused.push((id, reason)),
                None => {
                    crystals -= cost;
                    accepted.push(id);
                }
            }
        }
        if accepted.is_empty() {
            return Err(match stopped {
                Some(Refusal::Crystals) => Reject::InsufficientCrystals,
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
                    // "removes last entry from BuildQueue, refunds full cost to player". The army cost goes with the
                    // unit in production (design change 0143), and a waiting one never took any: "the player can
                    // still cancel it for a full refund".
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
    /// room frees nothing.
    pub(super) fn refund_production(&mut self, id: ObjectId) {
        let Some(o) = self.objects.get_mut(&id) else {
            return;
        };
        let def = o.def().structure.as_ref().and_then(|s| s.production.as_ref());
        let (Some(production), Some(def)) = (&mut o.production, def) else {
            return;
        };
        let owner = o.owner;
        for kind in production.take_queue() {
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
    }

    /// `core/entities.md > Structure Type > Exit Doorway` (design change 0080): `producer`'s finished unit comes into
    /// the game in its exit doorway with the order its rally point gives (`factions/gdo/structures/barracks.md >
    /// RallyPoint behavior`); or it stays inside, finished, while the doorway is occupied: "the next unit can only
    /// appear once the doorway is free. While a unit still stands in it, the building pauses". The doorway is occupied
    /// while a unit this building released (`Mover::exit`) is still in it. Out of a side doorway the unit appears half
    /// inside the middle of the exit side, facing out ([`Sim::doorway_spot`]); out of a portal's energy doorway, at the
    /// centre, facing the heading it leaves along (design change 0145: `production::EnergyDoorway::release`, which
    /// draws from the sim's Rng only with no rally point and two or more ways out to choose from).
    fn release_unit(&mut self, producer: ObjectId) {
        let Some(building) = self.objects.get(&producer) else {
            return;
        };
        // A producer underground releases into its owner's Tunnel Network (`sim/network.rs`).
        if producer_def(building).is_some_and(|def| def.exits == ExitRule::Network) {
            return self.release_into_network(producer);
        }
        let (Some(doorway), Some(owner)) = (ExitDoor::of(building), building.owner) else {
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
        if self.next_id == u32::MAX {
            return;
        }
        let occupied = self
            .objects
            .values()
            .any(|o| o.mover.as_ref().is_some_and(|mover| mover.exit == Some(producer)));
        if occupied {
            return;
        }
        // Where it appears, which way it faces, and the heading of a unit leaving an energy doorway.
        let (pos, facing, heading) = match doorway {
            ExitDoor::Side(doorway) => {
                let door = doorway.door();
                let Some(pos) = self.doorway_spot(&door, unit, half) else {
                    return;
                };
                (pos, door.outward(), None)
            }
            ExitDoor::Centre(portal) => {
                let grid = unit
                    .base_def()
                    .nav_class()
                    .and_then(|class| self.nav.nav().grid(class, nav::size_class(half * 2.0)));
                let rng = &mut self.rng;
                let rally = rally.map(|rally| rally.point());
                let Some(release) = portal.release(unit, half, grid, &self.objects, rally, |n| rng.below(n)) else {
                    return;
                };
                (release.pos, release.heading, Some(release.heading))
            }
        };
        if let Some(production) = self.objects.get_mut(&producer).and_then(|o| o.production.as_mut()) {
            production.release();
        }
        let id = self.insert_object(kind, Some(owner), pos, None, None);
        if let Some(o) = self.objects.get_mut(&id) {
            o.facing = math::vector_angle(facing);
            o.prev_facing = o.facing;
            if let Some(mover) = &mut o.mover {
                mover.exit = doorway.holds(pos, half).then_some(producer);
                mover.exit_heading = heading.filter(|_| mover.exit.is_some());
            }
        }
        self.events.push(SimEvent::UnitProduced { producer, unit: id });
        if let Some(kind) = rally.and_then(|rally| self.rally_order(id, rally)) {
            self.give_order(&[id], kind, false);
        }
    }

    /// Where a unit with half extents `half` appears in `door`: "half inside the building, at the middle of the
    /// exit side", or as far out as there is room when another unit or a blocked tile is right in front of the
    /// doorway (fully inside, against the exit side, at most: see `production::exit` for this reading of the
    /// design). Its box overlaps no unit of its layer and no blocked tile but the building's own. `None` if there
    /// is no such spot.
    fn doorway_spot(&self, door: &Door, unit: &UnitDef, half: Vec2) -> Option<Vec2> {
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
