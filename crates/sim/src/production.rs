//! Unit production (slice M2-D2) — `factions/gdo/structures/barracks.md`, and the Colonists' portals (slice M3-D,
//! `factions/colonists/structures/robo_portal.md`, `factions/colonists/structures/bio_portal.md`): a producing
//! structure trains units from a queue, one at a time, and releases each through an exit doorway with the order its
//! rally point gives.
//!
//! - **State** ([`Production`], `Object::production`): the BuildQueue, whose head is CurrentBuild (design change
//!   0081: the limit of [`QUEUE_LIMIT`] includes it), CurrentBuildProgress in the economy's fixed point
//!   (`economy::RATIO_ONE` a frame at full power, plan decision T6) and the RallyPoint ([`Rally`], `rally.rs`).
//!   What a type produces, at what cost and through which sides, is its `ProductionDef`
//!   (`StructureDef::production`).
//! - **Commands** ([`ProductionCommand`], sent as `Command::Production`; the gate and their application are in
//!   `sim/producers.rs`): `Train`, `CancelProduction`, `SetRally`.
//! - **Costs** (plan decision T7): a unit's Space Crystals are paid when it is queued, and only they and room in the
//!   queue are checked then. Its army cost (the GDO's Unit Control, the Colonists' Beacon Capacity: `economy::army`)
//!   is taken when production starts (design change 0143, which replaces 0082's reservation at queue time, for every
//!   faction): the unit at the front starts only while its owner's army cap has room for it, and counts from then
//!   on ([`Production::in_production`]); without room the producer waits with it at 0% ([`Production::blocked`]) and
//!   starts by itself once there is. A cancelled unit is refunded in full (`economy::Refund::Full`), and so is
//!   everything a producer holds when it leaves the game (design changes 0084 and 0085, `Sim::remove_objects`).
//! - **The step** (`Sim::step_production`, in the economy phase): a unit at 0% starts if there is room for it, or
//!   waits (0143); then the unit in production advances by its owner's power ratio (design change 0089). When it is
//!   complete it is released into the building's doorway (`exit.rs`): half inside the middle of an exit side
//!   ([`Doorway`], design change 0080: the Barracks), or at the centre of a portal's energy doorway, heading out
//!   through an open side ([`EnergyDoorway`], design change 0145); or it is held at 100% while a unit the building
//!   released is still in its doorway.

pub mod exit;
pub mod rally;

use serde::{Deserialize, Serialize};

use crate::defs::{ObjectKind, ProductionDef};
use crate::economy::{ArmyResource, RATIO_ONE};
use crate::hash::{StateHash, StateHasher};
use crate::ids::ObjectId;
use crate::orders::Target;
use crate::tuning::production::QUEUE_LIMIT;

pub use exit::{Door, Doorway, EnergyDoorway, ExitDoor};
pub use rally::Rally;

/// What the player tells producing structures (`factions/gdo/structures/barracks.md >
/// ObjectInterfaceState[Barracks]`), sent as `Command::Production` to every producer of the selection
/// (`systems/controls/selection_and_hud.md > SelectionGroup > No smart-casting`).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ProductionCommand {
    /// "Q: Build Peacekeeper": pay for a unit of this kind and add it to the BuildQueue. With several producers,
    /// those with the least work queued pay first, as many as the player can afford (design change 0083).
    Train(ObjectKind),
    /// "X: Cancel Production": "removes last entry from BuildQueue, refunds full cost to player".
    CancelProduction,
    /// "C: Set Rally Point", and the right-click of a producing structure: the rally point becomes this ground
    /// point or object. The producer itself as the target clears it (design change 0086).
    SetRally(Target),
}

/// Why a producer that received a `Train` didn't queue the unit, while others of the same command did
/// (`SimEvent::ProductionRefused`; design change 0083: "the others don't queue, and the player gets
/// insufficient-resources feedback"). The army cap is no reason: queuing doesn't check it (design change 0143,
/// `systems/controls/selection_and_hud.md > Group production with limited resources`: "unit capacity is not checked
/// when queuing"); a unit without room waits at the front of its queue instead ([`Production::blocked`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Refusal {
    /// Its BuildQueue holds [`QUEUE_LIMIT`] units.
    QueueFull,
    /// The player's Space Crystals don't cover the unit, after what the producers before it took.
    Crystals,
}

/// `factions/gdo/structures/barracks.md > BarracksInstanceState`: a producing structure's state.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Production {
    /// BuildQueue, at most [`QUEUE_LIMIT`] long. Its first entry is CurrentBuild, the unit in production.
    pub queue: Vec<ObjectKind>,
    /// CurrentBuildProgress of the first entry, in [`RATIO_ONE`]s: a frame at full power adds one `RATIO_ONE`, a
    /// frame at negative power the power ratio. 0 without a queue. At the recipe's `frames × RATIO_ONE` the unit is
    /// finished; it stays there while the unit waits inside for the doorway (design change 0085: "its production
    /// shows 100%").
    pub progress: u32,
    /// RallyPoint.
    pub rally: Option<Rally>,
    /// Whether the unit at the front waits at 0% for room under its owner's army cap (design change 0143;
    /// `factions/gdo/overview.md > GDOResources > Unit Control > Reserved when production starts`,
    /// `factions/colonists/overview.md > Beacon Capacity > Reserved when production starts`: "If there isn't enough,
    /// the building **waits** with the unit at the front (0% progress) and starts automatically once there is
    /// room"). The producers' step sets it in the tick the unit's start check first fails, with a
    /// `SimEvent::ProductionWaiting`, and clears it when the unit starts or leaves the queue; the interface shows
    /// the wait by it. Optional in files and left out of them while false, so that states saved before it existed
    /// still load; hashed only while true, so that the states of games in which no producer waits hash as they did
    /// before it existed (the goldens).
    #[serde(default, skip_serializing_if = "is_false")]
    pub blocked: bool,
}

fn is_false(b: &bool) -> bool {
    !*b
}

/// What a producer is working on, for the interface ([`Production::status`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ProductionStatus {
    /// CurrentBuild.
    pub kind: ObjectKind,
    /// CurrentBuildProgress, in [`RATIO_ONE`]s.
    pub progress: u32,
    /// The progress at which the unit is finished: its recipe's frames, in [`RATIO_ONE`]s.
    pub total: u32,
    /// The unit is finished and waits inside, because the exit doorway is occupied
    /// (`core/entities.md > Structure Type > Exit Doorway`, point 3).
    pub waiting: bool,
    /// The army cap whose room the unit waits for at 0% ([`Production::blocked`], design change 0143): its owner's
    /// Unit Control or Beacon Capacity. `None` while it is in production or about to start.
    pub blocked: Option<ArmyResource>,
}

/// The progress at which a unit of `frames` frames is finished.
pub(crate) fn total_of(frames: u32) -> u32 {
    frames.saturating_mul(RATIO_ONE)
}

impl Production {
    /// CurrentBuild: the unit in production.
    pub fn current(&self) -> Option<ObjectKind> {
        self.queue.first().copied()
    }

    /// Whether the BuildQueue is full (design change 0081: 20 with the unit in production).
    pub fn is_full(&self) -> bool {
        self.queue.len() >= QUEUE_LIMIT
    }

    /// The unit in production (design change 0143): the one at the front once the building has begun progress on it,
    /// a finished one waiting inside for the doorway included. It counts against its owner's army cap
    /// (`economy::army`) from then on, through production and for as long as it lives. `None` while the queue is
    /// empty or the unit at the front is at 0%: about to start in the producers' step, waiting for room
    /// ([`Production::blocked`]), or not getting on (a consumer at a power ratio of 0 begins no progress).
    pub fn in_production(&self) -> Option<ObjectKind> {
        self.current().filter(|_| self.progress > 0)
    }

    /// What the producer, of a type with these recipes, is working on; `None` with an empty queue. `cap` is its
    /// owner's army cap, which a unit at the front that is [`Production::blocked`] waits for.
    pub fn status(&self, def: &ProductionDef, cap: Option<ArmyResource>) -> Option<ProductionStatus> {
        let kind = self.current()?;
        let total = def.recipe(kind).map_or(0, |recipe| total_of(recipe.frames));
        Some(ProductionStatus {
            kind,
            progress: self.progress.min(total),
            total,
            waiting: self.progress >= total,
            blocked: cap.filter(|_| self.blocked),
        })
    }

    /// Design change 0083, "least work queued": the production time still ahead of the producer at full power, the
    /// unit in production included, in [`RATIO_ONE`]s of a frame. **Placeholder interpretation** of the design
    /// change: remaining time, not queue length.
    pub fn remaining_work(&self, def: &ProductionDef) -> u64 {
        let queued: u64 = self
            .queue
            .iter()
            .map(|&kind| def.recipe(kind).map_or(0, |recipe| u64::from(total_of(recipe.frames))))
            .sum();
        queued.saturating_sub(u64::from(self.progress))
    }

    /// Advances the unit in production by `speed` (`PowerSummary::speed`), up to finished. Returns whether it is
    /// finished: ready to leave, or waiting for the doorway.
    pub(crate) fn advance(&mut self, def: &ProductionDef, speed: u32) -> bool {
        let Some(kind) = self.current() else {
            self.progress = 0;
            return false;
        };
        let total = def.recipe(kind).map_or(0, |recipe| total_of(recipe.frames));
        self.progress = self.progress.saturating_add(speed).min(total);
        self.progress >= total
    }

    /// The finished unit left: the next entry starts from nothing, its start check (design change 0143) coming in
    /// the producers' step of the next tick.
    pub(crate) fn release(&mut self) -> Option<ObjectKind> {
        if self.queue.is_empty() {
            return None;
        }
        self.progress = 0;
        self.blocked = false;
        Some(self.queue.remove(0))
    }

    /// "X: Cancel Production": takes the last entry off the queue. If that was the unit at the front (in production,
    /// finished and waiting inside, or waiting for room: design change 0143, "the player can still cancel it for a
    /// full refund"), its progress and its wait go with it.
    pub(crate) fn cancel_last(&mut self) -> Option<ObjectKind> {
        let kind = self.queue.pop()?;
        if self.queue.is_empty() {
            self.progress = 0;
            self.blocked = false;
        }
        Some(kind)
    }

    /// The producer leaves the game, or changes into a kind without its queue: the whole BuildQueue goes, to be
    /// refunded (design changes 0084 and 0085), with its progress and its wait.
    pub(crate) fn take_queue(&mut self) -> Vec<ObjectKind> {
        self.progress = 0;
        self.blocked = false;
        std::mem::take(&mut self.queue)
    }

    /// Drops a rally point on an object that is gone (`> RallyPoint behavior`: "If the RallyPoint references an
    /// ObjectInstance that no longer exists, the RallyPoint is reset to None").
    pub(crate) fn forget(&mut self, is_gone: impl Fn(ObjectId) -> bool) {
        if let Some(Rally::Object { id, .. }) = self.rally
            && is_gone(id)
        {
            self.rally = None;
        }
    }
}

impl StateHash for Production {
    fn hash(&self, h: &mut StateHasher) {
        let Production {
            queue,
            progress,
            rally,
            blocked,
        } = self;
        queue.hash(h);
        progress.hash(h);
        rally.hash(h);
        // (M3-P) Only while the producer waits: the states of games in which none waits hash as they did before the
        // field existed (the goldens stand). A waiting producer differs from the same one not waiting by the tag.
        if *blocked {
            h.tag(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::ExitRule;
    use crate::defs::Recipe;
    use crate::sides::SideLetter;

    const BARRACKS: ProductionDef = ProductionDef {
        exits: ExitRule::Side(SideLetter::B),
        recipes: &[Recipe {
            kind: ObjectKind::Peacekeeper,
            cost: 50,
            frames: 80,
        }],
    };

    fn queued(n: usize) -> Production {
        Production {
            queue: vec![ObjectKind::Peacekeeper; n],
            ..Production::default()
        }
    }

    /// `> Produces`: 80 frames at full power; at the ratio of 30/51 exactly 136 ticks (plan decision T6).
    #[test]
    fn a_unit_takes_its_frames_at_full_power_and_longer_at_a_ratio() {
        let ticks = |speed: u32| {
            let mut p = queued(1);
            (1..).find(|_| p.advance(&BARRACKS, speed)).unwrap()
        };
        assert_eq!(ticks(RATIO_ONE), 80);
        assert_eq!(ticks(38_551), 136, "30/51, rounded up");
        assert_eq!(ticks(RATIO_ONE / 2), 160);
    }

    /// Design change 0085: a finished unit stays at 100% until it is released; the next starts from nothing.
    #[test]
    fn a_finished_unit_waits_at_full_progress() {
        let mut p = queued(2);
        for _ in 0..200 {
            p.advance(&BARRACKS, RATIO_ONE);
        }
        let status = p.status(&BARRACKS, None).unwrap();
        assert_eq!(
            (status.progress, status.total, status.waiting),
            (80 << 16, 80 << 16, true)
        );
        assert_eq!(p.in_production(), Some(ObjectKind::Peacekeeper), "it still counts");
        assert_eq!(p.release(), Some(ObjectKind::Peacekeeper));
        assert_eq!((p.queue.len(), p.progress), (1, 0));
        assert!(!p.status(&BARRACKS, None).unwrap().waiting);
        assert_eq!(queued(0).status(&BARRACKS, None), None);
    }

    /// "X: Cancel Production" takes the last entry; the unit in production last of all, with its progress.
    #[test]
    fn cancelling_takes_the_last_entry() {
        let mut p = queued(2);
        p.advance(&BARRACKS, RATIO_ONE);
        assert_eq!(p.cancel_last(), Some(ObjectKind::Peacekeeper));
        assert_eq!(
            (p.queue.len(), p.progress),
            (1, RATIO_ONE),
            "the unit in production goes on"
        );
        assert_eq!(p.cancel_last(), Some(ObjectKind::Peacekeeper));
        assert_eq!((p.queue.len(), p.progress), (0, 0));
        assert_eq!(p.cancel_last(), None);
    }

    /// Design change 0143: the unit at the front is in production, and takes its army cost, once the building has
    /// begun progress on it: not while it is queued, nor at 0%, whether it is about to start or waits for room.
    #[test]
    fn a_unit_is_in_production_once_progress_has_begun() {
        let mut p = queued(3);
        assert_eq!(p.in_production(), None, "queued, not started");
        p.blocked = true;
        assert_eq!(p.in_production(), None, "waiting at 0%");
        p.blocked = false;
        p.advance(&BARRACKS, 0);
        assert_eq!(p.in_production(), None, "no progress at a power ratio of 0");
        p.advance(&BARRACKS, 1);
        assert_eq!(p.in_production(), Some(ObjectKind::Peacekeeper));
        assert_eq!(queued(0).in_production(), None);
    }

    /// Design change 0143: a waiting unit at the front shows the cap it waits for; the wait ends with the unit's
    /// place at the front: cancelled ("the player can still cancel it for a full refund"), refunded with the whole
    /// queue, or, once it started, released. A cancel of an entry behind it leaves it waiting.
    #[test]
    fn the_wait_goes_with_the_unit_at_the_front() {
        let cap = Some(ArmyResource::UnitControl);
        let waiting = |n| Production {
            blocked: true,
            ..queued(n)
        };
        let status = waiting(2).status(&BARRACKS, cap).unwrap();
        assert_eq!((status.progress, status.blocked), (0, cap));
        assert_eq!(queued(2).status(&BARRACKS, cap).unwrap().blocked, None, "not waiting");
        let mut p = waiting(2);
        p.cancel_last();
        assert!(p.blocked, "the unit behind it went");
        p.cancel_last();
        assert!(!p.blocked && p.queue.is_empty());
        let mut p = waiting(3);
        assert_eq!(p.take_queue().len(), 3);
        assert_eq!((p.queue.len(), p.progress, p.blocked), (0, 0, false));
        let mut p = queued(2);
        while !p.advance(&BARRACKS, RATIO_ONE) {}
        p.blocked = true;
        p.release();
        assert!(!p.blocked, "the next unit starts its own check");
    }

    /// The wait is hashed only while it lasts (the goldens of games without one stand), and left out of files while
    /// it doesn't, so that a state saved before it existed loads.
    #[test]
    fn the_wait_is_hashed_and_saved_only_while_it_lasts() {
        let hash = |p: &Production| {
            let mut h = StateHasher::new();
            p.hash(&mut h);
            h.finish()
        };
        let (idle, waiting) = (
            queued(1),
            Production {
                blocked: true,
                ..queued(1)
            },
        );
        assert_ne!(hash(&idle), hash(&waiting));
        let json = serde_json::to_string(&idle).unwrap();
        assert!(!json.contains("blocked"), "{json}");
        let old: Production =
            serde_json::from_str(r#"{"queue": ["Peacekeeper"], "progress": 0, "rally": null}"#).unwrap();
        assert_eq!(old, idle);
        let saved: Production = serde_json::from_str(&serde_json::to_string(&waiting).unwrap()).unwrap();
        assert_eq!(saved, waiting);
    }

    /// Design change 0083: the work queued is the time still ahead, the unit in production counted by what is left
    /// of it.
    #[test]
    fn the_work_queued_is_the_production_time_still_ahead() {
        assert_eq!(queued(0).remaining_work(&BARRACKS), 0);
        assert_eq!(queued(3).remaining_work(&BARRACKS), 3 * (80 << 16));
        let mut nearly = queued(1);
        for _ in 0..79 {
            nearly.advance(&BARRACKS, RATIO_ONE);
        }
        assert_eq!(nearly.remaining_work(&BARRACKS), 1 << 16);
        assert!(queued(QUEUE_LIMIT).is_full() && !queued(QUEUE_LIMIT - 1).is_full());
    }

    #[test]
    fn a_rally_on_an_object_that_is_gone_is_cleared() {
        let mut p = Production {
            rally: Some(Rally::Object {
                id: ObjectId(7),
                at: glam::Vec2::ONE,
            }),
            ..Production::default()
        };
        p.forget(|id| id == ObjectId(8));
        assert!(p.rally.is_some());
        p.forget(|id| id == ObjectId(7));
        assert_eq!(p.rally, None);
        let mut ground = Production {
            rally: Some(Rally::Point(glam::Vec2::ONE)),
            ..Production::default()
        };
        ground.forget(|_| true);
        assert!(ground.rally.is_some());
    }
}
