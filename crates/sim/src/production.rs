//! Unit production (slice M2-D2) — `factions/gdo/structures.md > Barracks`: a producing structure trains units
//! from a queue, one at a time, and releases each through its exit doorway with the order its rally point gives.
//!
//! - **State** ([`Production`], `Object::production`): the BuildQueue, whose head is CurrentBuild (design change
//!   0081: the limit of [`QUEUE_LIMIT`] includes it), CurrentBuildProgress in the economy's fixed point
//!   (`economy::RATIO_ONE` a frame at full power, plan decision T6) and the RallyPoint ([`Rally`], `rally.rs`).
//!   What a type produces, at what cost and through which side, is its `ProductionDef`
//!   (`StructureDef::production`).
//! - **Commands** ([`ProductionCommand`], sent as `Command::Production`; the gate and their application are in
//!   `sim/producers.rs`): `Train`, `CancelProduction`, `SetRally`.
//! - **Costs** (plan decision T7): a unit's Space Crystals are paid, and its Unit Control reserved (design change
//!   0082, `economy::unit_control`), when it is queued. A cancelled unit is refunded in full
//!   (`economy::Refund::Full`), and so is everything a producer holds when it leaves the game (design changes 0084
//!   and 0085, `Sim::remove_objects`).
//! - **The step** (`Sim::step_production`, in the economy phase): the head advances by its owner's power ratio
//!   (design change 0089). When it is complete the unit is released through the doorway ([`Doorway`], `exit.rs`,
//!   design change 0080), or held at 100% while the doorway is occupied.

pub mod exit;
pub mod rally;

use serde::{Deserialize, Serialize};

use crate::defs::{ObjectKind, ProductionDef};
use crate::economy::RATIO_ONE;
use crate::hash::{StateHash, StateHasher};
use crate::ids::ObjectId;
use crate::orders::Target;

pub use exit::Doorway;
pub use rally::Rally;

/// `factions/gdo/structures.md > Barracks > BarracksInstanceState`: "BuildQueue: array of ObjectEnum (max 20,
/// including the unit currently in production)" (design change 0081).
pub const QUEUE_LIMIT: usize = 20;

/// What the player tells producing structures (`factions/gdo/structures.md > Barracks >
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
/// insufficient-resources feedback").
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Refusal {
    /// Its BuildQueue holds [`QUEUE_LIMIT`] units.
    QueueFull,
    /// The player's Space Crystals don't cover the unit, after what the producers before it took.
    Crystals,
    /// The unit would take the player's Unit Control past the cap (design change 0082).
    UnitControl,
}

/// `factions/gdo/structures.md > Barracks > BarracksInstanceState`: a producing structure's state.
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

    /// What the producer, of a type with these recipes, is working on; `None` with an empty queue.
    pub fn status(&self, def: &ProductionDef) -> Option<ProductionStatus> {
        let kind = self.current()?;
        let total = def.recipe(kind).map_or(0, |recipe| total_of(recipe.frames));
        Some(ProductionStatus {
            kind,
            progress: self.progress.min(total),
            total,
            waiting: self.progress >= total,
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

    /// The finished unit left: the next entry starts from nothing.
    pub(crate) fn release(&mut self) -> Option<ObjectKind> {
        if self.queue.is_empty() {
            return None;
        }
        self.progress = 0;
        Some(self.queue.remove(0))
    }

    /// "X: Cancel Production": takes the last entry off the queue. If that was the unit in production (finished
    /// and waiting inside included), its progress goes with it.
    pub(crate) fn cancel_last(&mut self) -> Option<ObjectKind> {
        let kind = self.queue.pop()?;
        if self.queue.is_empty() {
            self.progress = 0;
        }
        Some(kind)
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
        let Production { queue, progress, rally } = self;
        queue.hash(h);
        progress.hash(h);
        rally.hash(h);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::Recipe;
    use crate::sides::SideLetter;

    const BARRACKS: ProductionDef = ProductionDef {
        exit: SideLetter::B,
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
        let status = p.status(&BARRACKS).unwrap();
        assert_eq!(
            (status.progress, status.total, status.waiting),
            (80 << 16, 80 << 16, true)
        );
        assert_eq!(p.release(), Some(ObjectKind::Peacekeeper));
        assert_eq!((p.queue.len(), p.progress), (1, 0));
        assert!(!p.status(&BARRACKS).unwrap().waiting);
        assert_eq!(queued(0).status(&BARRACKS), None);
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
