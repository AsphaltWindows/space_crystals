//! The producers' step of the economy phase (plan decision T5; `ARCHITECTURE.md > Tick pipeline`), with the start
//! check of design change 0143 (`factions/gdo/overview.md > GDOResources > Unit Control`, "Reserved when production
//! starts"; `factions/colonists/overview.md > Beacon Capacity > Reserved when production starts`): "When the unit
//! reaches the front of the queue and production is about to start, it needs free [capacity] (available minus used)
//! at least equal to its [cost] [...]. If there isn't enough, the building **waits** with the unit at the front (0%
//! progress) and starts automatically once there is room".
//!
//! - **Which units**: a unit at 0% at the front of its queue, whether it has just reached the front or waits there.
//!   A unit in production (`Production::in_production`) is never checked again: it "always finishes" (Q0041 point
//!   2), however far the cap falls below what is used (design change 0112, `> Over capacity`).
//! - **The figures** are those of [`crate::Sim::army_cap`]: `available − used ≥ cost` ([`ArmyCap::fits`]), with
//!   `used` the player's units and their producers' units in production. They are worked out once a step for each
//!   player who has a unit to start, and the units that start are added as they do ([`Room`]), so that one room is
//!   given once.
//! - **Who is served first** when several producers wait for the same room: id order, the order of the step. The
//!   one other order in the design, design change 0083's "least work queued first", is read as the rule of a group
//!   `Train` at queue time (`Sim::plan_training`), which 0143 leaves to the crystals: nothing in 0143 orders the
//!   starts, so the most literal reading keeps the step's (reported to the Director, slice M3-P).
//! - **The notification** ("A blocked queue raises a notification (not yet designed)"): placeholder, a
//!   `SimEvent::ProductionWaiting` in the tick a wait begins, once until the unit starts or leaves the queue, and
//!   `Production::blocked` meanwhile, which the interface shows.
//! - **At a power ratio of 0** (a GDO consumer with nothing generated, design change 0089) the check passes or fails
//!   all the same, but a unit that passes makes no progress, and so doesn't start or take anything: it is checked
//!   again in the next step, as one that waits.

use std::collections::BTreeMap;

use crate::defs::ObjectKind;
use crate::economy::{self, ArmyCap};
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::sim::Sim;

/// The army caps of the players whose producers have a unit to start in this step (`None` for a faction without one
/// yet): worked out the first time one is needed, then kept up to date with the units that start, so that the
/// producers after them in the step see them (design change 0143: "it then counts as used for its production and
/// life"). Nothing else in the producers' step changes what is used: a release turns a unit in production into a
/// unit, counted either way; the dead of the tick left before the economy phase (5e), and a Beacon that completed in
/// this tick provides already (`Sim::step_transforms` comes first).
#[derive(Default)]
struct Room(BTreeMap<PlayerId, Option<ArmyCap>>);

impl Room {
    /// A unit of `kind` of `player`'s started: it takes its cost.
    fn take(&mut self, player: PlayerId, kind: ObjectKind) {
        if let Some(Some(cap)) = self.0.get_mut(&player) {
            cap.used = cap.used.saturating_add(cap.resource.cost_of(kind));
        }
    }
}

impl Sim {
    /// The producers' step, in id order: a unit at 0% at the front of a queue starts if its owner's army cap has room
    /// for it, or the producer waits ([`Sim::may_start`]); the unit in production advances by its owner's power
    /// ratio (design change 0089: "unit production speed" is slowed at negative power), and a finished one is
    /// released through the exit doorway, or held while that is occupied (`Sim::release_unit`). Producers from
    /// `first_new` on were created this tick and wait for the next. The released units are in the game from this
    /// tick, and act from the next (the spawn-tick rule); the next unit in the queue starts in the next tick.
    pub(in crate::sim) fn step_production(&mut self, first_new: ObjectId) {
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
        let mut room = Room::default();
        for id in busy {
            if !self.may_start(id, &mut room) {
                continue;
            }
            let Some(o) = self.objects.get_mut(&id) else {
                continue;
            };
            let def = o.def().structure.as_ref().and_then(|s| s.production.as_ref());
            let (Some(owner), Some(def), Some(production)) = (o.owner, def, &mut o.production) else {
                continue;
            };
            let speed = power.get(&owner).copied().unwrap_or_default().speed(o.kind);
            let starting = production.in_production().is_none();
            let finished = production.advance(def, speed);
            if starting && let Some(kind) = production.in_production() {
                room.take(owner, kind);
            }
            if finished {
                self.release_unit(id);
            }
        }
    }

    /// Design change 0143's start check of producer `id`, before the step advances it: whether the unit at its front
    /// may get on. One in production always may; one at 0% while there is room for it under its owner's army cap,
    /// or always for a faction without a cap yet. Without room the producer waits (`Production::blocked`), and in
    /// the tick its wait begins it raises a `SimEvent::ProductionWaiting`, once (module docs).
    fn may_start(&mut self, id: ObjectId, room: &mut Room) -> bool {
        let Some(o) = self.objects.get(&id) else {
            return false;
        };
        let (Some(owner), Some(production)) = (o.owner, o.production.as_ref()) else {
            return false;
        };
        let Some(kind) = production.current() else {
            return false;
        };
        if production.in_production().is_some() {
            return true;
        }
        let cap = *room.0.entry(owner).or_insert_with(|| self.army_cap(owner));
        let Some(cap) = cap else {
            return true;
        };
        let fits = cap.fits(cap.resource.cost_of(kind));
        let Some(production) = self.objects.get_mut(&id).and_then(|o| o.production.as_mut()) else {
            return false;
        };
        let begins = !fits && !production.blocked;
        production.blocked = !fits;
        if begins {
            self.events.push(SimEvent::ProductionWaiting {
                producer: id,
                reason: cap.resource,
            });
        }
        fits
    }
}
