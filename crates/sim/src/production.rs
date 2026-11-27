//! Unit production (slice M2-D2) — `factions/gdo/structures/barracks.md`, and the Colonists' portals (slice M3-D,
//! `factions/colonists/structures/robo_portal.md`, `factions/colonists/structures/bio_portal.md`): a producing
//! structure trains units from a queue, one at a time, and releases each through an exit doorway with the order its
//! rally point gives.
//!
//! - **State** ([`Production`], `Object::production`): the BuildQueue, whose head is CurrentBuild (design change
//!   0081: the limit of `Production::queue_limit` includes it), CurrentBuildProgress in the economy's fixed point
//!   (`economy::RATIO_ONE` a frame at full power, plan decision T6) and the RallyPoint ([`Rally`], `rally.rs`), which
//!   an unfinished building of a type that has one keeps alone until it completes ([`takes_rally`], design change
//!   0396).
//!   What a type produces, at what cost and through which sides, is its `ProductionDef`
//!   (`StructureDef::production`).
//! - **Commands** ([`ProductionCommand`], sent as `Command::Production`; the gate and their application are in
//!   `sim/producers.rs`): `Train`, `CancelProduction`, `SetRally`; a pool building's `TrainFrom` and `Release` (slice
//!   M5-F2), an Altar's `Augment` and `ReleaseOf` (slice M5-N1b), and a Temple's `Research` (slice M5-N3).
//! - **Research** (slice M5-N3, plan decisions T70 and T74; `sim/producers/research.rs`): a Temple researches on its
//!   queue ([`QueueEntry::Research`]), paid at queue and refunded as any entry is, its progress gaining a
//!   [`RATIO_ONE`] a frame for each Recruit staffing it, none without one.
//! - **Costs** (plan decision T7): a unit's Space Crystals are paid when it is queued, and only they and room in the
//!   queue are checked then. Its army cost (the GDO's Unit Control, the Colonists' Beacon Capacity: `economy::army`)
//!   is taken when production starts (design change 0143, which replaces 0082's reservation at queue time, for every
//!   faction): the unit at the front starts only while its owner's army cap has room for it, and counts from then
//!   on ([`Production::in_production`]); without room the producer waits with it at 0% ([`Production::blocked`]) and
//!   starts by itself once there is. A cancelled unit is refunded in full (`economy::Refund::Full`), and so is
//!   everything a producer holds when it leaves the game (design changes 0084 and 0085, `Sim::remove_objects`).
//! - **Recruiting** (slice M5-C, plan decision T57; `crate::recruitment`, `sim/recruitment.rs`): a Recruitment Hall
//!   trains nothing on command. It queues its Recruit by itself, for free, while its lineage uses less Unit Control
//!   than its capacity, and its progress gains the tiles it claims each frame instead of the power ratio
//!   ([`total_of`]).
//! - **Pool producers** (slice M5-F2, plan decisions T70 and T71; `sim/producers/pool.rs`): a producer that also holds
//!   units (`StructureDef::holds`), the Armory first, makes its units of the units inside. Each entry of its
//!   TrainingQueue reserves units of the pool as it is queued ([`Production::reserved`]): as many as the product's
//!   Unit Control cost ("a trained unit's Unit Control cost equals the number of Recruits consumed",
//!   `factions/cults/structures/README.md > Cults Vehicles`), or one for a Release ([`QueueEntry::Release`], design
//!   change 0337) or an Altar's augmentation order ([`QueueEntry::Augment`], slice M5-N1b). Its queue is bounded by its
//!   capacity alone, the reserved units being inside (design change 0347 point 5: "Queues are limited by capacity, not
//!   by a separate 20"), so `Production::queue_limit` isn't its.
//! - **The step** (`Sim::step_production`, in the economy phase): a unit at 0% starts if there is room for it, or
//!   waits (0143); then the unit in production advances by its owner's power ratio (design change 0089). When it is
//!   complete it is released into the building's doorway (`exit.rs`): half inside the middle of an exit side
//!   ([`Doorway`], design change 0080: the Barracks), or at the centre of a portal's energy doorway, heading out
//!   through an open side ([`EnergyDoorway`], design change 0145); or it is held at 100% while a unit the building
//!   released is still in its doorway.

pub mod exit;
pub mod queue;
pub mod rally;

use serde::{Deserialize, Serialize};

use crate::augmentation::{AugmentOrder, RecruitKind};
use crate::defs::{ObjectKind, ProductionDef, Tech};
use crate::economy::{ArmyResource, RATIO_ONE};
use crate::hash::{StateHash, StateHasher};
use crate::ids::ObjectId;
use crate::orders::Target;
use crate::ruleset::rules;
use crate::units::Tick;

pub use exit::{Door, Doorway, EnergyDoorway, ExitDoor, Runway};
pub use queue::QueueEntry;
pub use rally::{Rally, default_rally, takes_rally};

/// What the player tells producing structures (`factions/gdo/structures/barracks.md >
/// ObjectInterfaceState[Barracks]`), sent as `Command::Production` to every producer of the selection
/// (`systems/controls/selection.md > SelectionGroup > No smart-casting`).
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
    /// (M5-F2) A pool producer's Train, from KindMenu\[`from`\] (design change 0337,
    /// `factions/cults/structures/README.md > Choosing Recruits, Release and Eject All`: "the building's production
    /// options, using Recruits of that kind (e.g. Armory: **Q** Train Militant [...])"): pay for a unit of `kind` and
    /// add it to the TrainingQueue, reserving the units of kind `from` inside that it is made of, those that entered
    /// first ("Within a kind, the unit that entered first is used first (0332)"). A producer without a pool takes
    /// `Train` instead.
    TrainFrom { kind: ObjectKind, from: RecruitKind },
    /// (M5-F2) "**A: Release**" in KindMenu\[kind\] (design change 0337): queue a Release, free, which reserves the
    /// unit of this kind inside that entered first, and lets it out unchanged once it completes.
    Release(RecruitKind),
    /// (M5-N1b) An Altar's order, from AugmentMenu\[UnitType, State\] (`factions/cults/augmentation.md > Altar >
    /// ObjectInterfaceState[Altar]`: "Each click queues one order"): pay for `order` and add it to the
    /// AugmentationQueue, reserving the unit of type `kind` and kind `from` inside that entered first ("an order
    /// reserves the matching unit(s) that **entered the building first**"), which must be missing every Augmentation
    /// the order applies.
    Augment {
        kind: ObjectKind,
        from: RecruitKind,
        order: AugmentOrder,
    },
    /// (M5-N1b) "**A: Release**" in an Altar's AugmentMenu\[UnitType, State\] ("In every state: **A: Release** (free
    /// no-op, 4 frames, placeholder; the unit walks out unchanged"): a Release, as [`ProductionCommand::Release`]'s, of
    /// the unit of type `kind` and kind `from` inside that entered first. The Armory's names the kind alone: it holds
    /// Recruits only.
    ReleaseOf { kind: ObjectKind, from: RecruitKind },
    /// (M5-N3) A Temple's **Q** and **W** (`factions/cults/structures/temple.md > Placeholder stats`: "**Q** Enhanced
    /// Constitution (next level), **W** Improved Metabolism (next level)"): pay for `tech` and add it to the research
    /// queue (`core/tech_and_research.md > Research > Shared Rules` 2 and 3). With several Temples, one takes it, the
    /// one with the least work queued (design change 0083's order): "a technology cannot be queued or in progress at
    /// more than one structure at the same time" (4).
    Research(Tech),
}

/// Why a producer that received a `Train` didn't queue the unit, while others of the same command did
/// (`SimEvent::ProductionRefused`; design change 0083: "the others don't queue, and the player gets
/// insufficient-resources feedback"). The army cap is no reason: queuing doesn't check it (design change 0143,
/// `systems/controls/selection.md > Group production with limited resources`: "unit capacity is not checked
/// when queuing"); a unit without room waits at the front of its queue instead ([`Production::blocked`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Refusal {
    /// Its BuildQueue holds `Production::queue_limit` units.
    QueueFull,
    /// The player's Space Crystals don't cover the unit, after what the producers before it took.
    Crystals,
    /// (M5-F2) A pool producer has fewer free units of the kind inside than the order needs (plan decision T70: "fewer
    /// free Recruits than the order needs"): the Unit Control cost of the unit to train, one for a Release.
    NoRecruit,
    /// (M6-GD) The player's Supplies don't cover the unit (`Recipe::supplies`), after what the producers before it took:
    /// "crystals or any other resource paid at queue time" (`systems/controls/selection.md > Group production with
    /// limited resources`).
    Supplies,
}

/// `factions/gdo/structures/barracks.md > BarracksInstanceState`: a producing structure's state.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Production {
    /// BuildQueue, at most `Production::queue_limit` long (a pool producer's TrainingQueue, by its capacity: module
    /// docs). Its first entry is CurrentBuild, the unit in production.
    pub queue: Vec<QueueEntry>,
    /// CurrentBuildProgress of the first entry, in [`RATIO_ONE`]s: a frame at full power adds one `RATIO_ONE`, a
    /// frame at negative power the power ratio. A Recruitment Hall's in tiles: a frame adds the tiles it claims (plan
    /// decision T57). 0 without a queue. At the recipe's frames in those units ([`total_of`]) the unit is finished; it
    /// stays there while the unit waits inside for the doorway (design change 0085: "its production shows 100%").
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
    /// The tick its building was completed, by which a Recruitment Hall claims the tiles it contests with another
    /// before it or after it (`factions/cults/structures/recruitment_hall.md > TileClaiming`: "the hall that was
    /// constructed first has priority"; plan decision T57, `crate::recruitment`): 0 for a Hall in the game from the
    /// start, and for every producer that recruits nothing, which has no use for it. A Hall that comes into the game
    /// later has the tick it comes in, complete (a debug spawn's), or the tick its functions start, held back until
    /// its completion (`Sim::start_functions`; slice M5-E's sites). Optional in files and left out of them while 0;
    /// hashed only while set, so that the states of games without such a Hall hash as they did before it existed (the
    /// goldens).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub completed_at: Tick,
    /// (M5-F2) A pool producer's reservations, one list per entry of its queue, in the same order: the units inside
    /// each entry is made of, or lets out (`factions/cults/structures/armory.md > Training`: queuing an order "reserves
    /// one Recruit from StoredRecruits (removing it from the internal pool for the duration it sits in the queue)"),
    /// longest inside first. Empty for a producer without a pool. Optional in files and left out of them
    /// while empty, and hashed only while it has an entry, so that the states of games without a pool producer at work
    /// hash as they did before it existed (the goldens).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reserved: Vec<Vec<ObjectId>>,
}

fn is_false(b: &bool) -> bool {
    !*b
}

fn is_zero(tick: &Tick) -> bool {
    *tick == 0
}

/// What a producer is working on, for the interface ([`Production::status`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ProductionStatus {
    /// CurrentBuild: the unit in production, or a pool producer's Release.
    pub entry: QueueEntry,
    /// CurrentBuildProgress, in [`RATIO_ONE`]s (a Recruitment Hall's in tiles: `total_of`).
    pub progress: u32,
    /// The progress at which the unit is finished: its recipe's frames, in [`RATIO_ONE`]s (a Recruitment Hall's in
    /// tiles, `total_of`).
    pub total: u32,
    /// The unit is finished and waits inside, because the exit doorway is occupied
    /// (`core/entities.md > Structure Type > Exit Doorway`, point 3), or, at a pool producer, behind the units ready
    /// before it (design change 0348: "units sharing the exit doorway come out **in the order they are ready**").
    pub waiting: bool,
    /// The army cap whose room the unit waits for at 0% ([`Production::blocked`], design change 0143): its owner's
    /// Unit Control or Beacon Capacity. `None` while it is in production or about to start.
    pub blocked: Option<ArmyResource>,
}

/// The progress at which a unit of `frames` frames of a `def` producer is finished: a frame at full power adds one
/// [`RATIO_ONE`], so `frames × RATIO_ONE`. A Recruitment Hall's frame adds the tiles it claims, so `frames ×` its
/// area's tiles (plan decision T57): its Recruit takes BaseRecruitProductionTime / Effectiveness frames, rounded up,
/// exactly at every effectiveness (`factions/cults/structures/recruitment_hall.md > RecruitmentHallInstanceState`:
/// "CurrentProductionTime: number (BaseRecruitProductionTime / Effectiveness)"; 384 frames at 50 tiles).
pub(crate) fn total_of(def: &ProductionDef, frames: u32) -> u32 {
    let per_frame = def.recruitment.map_or(RATIO_ONE, |recruitment| recruitment.tiles());
    frames.saturating_mul(per_frame)
}

impl Production {
    /// CurrentBuild: the entry at the front.
    pub fn current(&self) -> Option<QueueEntry> {
        self.queue.first().copied()
    }

    /// The units the entry at the front reserved, the first of which it trains or lets out (slice M5-F2): none at a
    /// producer without a pool.
    pub fn reserved_by_current(&self) -> &[ObjectId] {
        self.reserved.first().map_or(&[], Vec::as_slice)
    }

    /// Whether unit `id` is reserved by one of the entries (slice M5-F2).
    pub fn reserves(&self, id: ObjectId) -> bool {
        self.reserved.iter().any(|units| units.contains(&id))
    }

    /// Whether the BuildQueue is full (design change 0081: 20 with the unit in production).
    pub fn is_full(&self) -> bool {
        self.queue.len() >= rules().tuning.production.queue_limit
    }

    /// The unit in production (design change 0143): the one at the front once the building has begun progress on it,
    /// a finished one waiting inside for the doorway included. It counts against its owner's army cap
    /// (`economy::army`) from then on, through production and for as long as it lives. `None` while the queue is
    /// empty or the unit at the front is at 0%: about to start in the producers' step, waiting for room
    /// ([`Production::blocked`]), or not getting on (a consumer at a power ratio of 0 begins no progress). A Release
    /// produces nothing.
    pub fn in_production(&self) -> Option<ObjectKind> {
        self.current().and_then(QueueEntry::unit).filter(|_| self.progress > 0)
    }

    /// What the producer, of a type with these recipes, is working on; `None` with an empty queue. `cap` is its
    /// owner's army cap, which a unit at the front that is [`Production::blocked`] waits for.
    pub fn status(&self, def: &ProductionDef, cap: Option<ArmyResource>) -> Option<ProductionStatus> {
        let entry = self.current()?;
        let total = total_of(def, entry.frames(def));
        Some(ProductionStatus {
            entry,
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
            .map(|entry| u64::from(total_of(def, entry.frames(def))))
            .sum();
        queued.saturating_sub(u64::from(self.progress))
    }

    /// Advances the unit in production by `speed` (`PowerSummary::speed`), up to finished. Returns whether it is
    /// finished: ready to leave, or waiting for the doorway.
    pub(crate) fn advance(&mut self, def: &ProductionDef, speed: u32) -> bool {
        let Some(entry) = self.current() else {
            self.progress = 0;
            return false;
        };
        let total = total_of(def, entry.frames(def));
        self.progress = self.progress.saturating_add(speed).min(total);
        self.progress >= total
    }

    /// Adds `entry` at the end of the queue, with the units it reserves at a pool producer (none elsewhere).
    pub(crate) fn push(&mut self, entry: QueueEntry, reserved: Vec<ObjectId>) {
        debug_assert_eq!(
            self.reserved.len(),
            self.queue.len() * usize::from(!self.reserved.is_empty())
        );
        if !reserved.is_empty() {
            debug_assert_eq!(
                self.reserved.len(),
                self.queue.len(),
                "a pool producer reserves for every entry"
            );
            self.reserved.push(reserved);
        }
        self.queue.push(entry);
    }

    /// The finished unit left: the next entry starts from nothing, its start check (design change 0143) coming in
    /// the producers' step of the next tick. Returns the entry, and the units it had reserved.
    pub(crate) fn release(&mut self) -> Option<(QueueEntry, Vec<ObjectId>)> {
        if self.queue.is_empty() {
            return None;
        }
        self.progress = 0;
        self.blocked = false;
        let reserved = if self.reserved.is_empty() {
            Vec::new()
        } else {
            self.reserved.remove(0)
        };
        Some((self.queue.remove(0), reserved))
    }

    /// "X: Cancel Production": takes the last entry off the queue, with the units it reserved, which go back to the
    /// pool (`factions/cults/structures/armory.md > ObjectInterfaceState[Armory]`: X "returns its reserved Recruit to
    /// StoredRecruits"). If that was the unit at the front (in production, finished and waiting inside, or waiting
    /// for room: design change 0143, "the player can still cancel it for a full refund"), its progress and its wait
    /// go with it.
    pub(crate) fn cancel_last(&mut self) -> Option<(QueueEntry, Vec<ObjectId>)> {
        let entry = self.queue.pop()?;
        let reserved = if self.reserved.len() > self.queue.len() {
            self.reserved.pop().unwrap_or_default()
        } else {
            Vec::new()
        };
        if self.queue.is_empty() {
            self.progress = 0;
            self.blocked = false;
        }
        Some((entry, reserved))
    }

    /// The producer leaves the game, or changes into a kind without its queue: the whole BuildQueue goes, to be
    /// refunded (design changes 0084 and 0085), with its progress and its wait, each entry with the units it
    /// reserved.
    pub(crate) fn take_queue(&mut self) -> Vec<(QueueEntry, Vec<ObjectId>)> {
        self.progress = 0;
        self.blocked = false;
        let mut reserved = std::mem::take(&mut self.reserved).into_iter();
        (std::mem::take(&mut self.queue).into_iter())
            .map(|entry| (entry, reserved.next().unwrap_or_default()))
            .collect()
    }

    /// Drops a rally point on an object that is gone (`factions/gdo/structures/barracks.md > RallyPoint behavior`: "If
    /// the RallyPoint references an ObjectInstance that no longer exists, the RallyPoint is reset to None").
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
            completed_at,
            reserved,
        } = self;
        queue.hash(h);
        progress.hash(h);
        rally.hash(h);
        // (M3-P) Only while the producer waits: the states of games in which none waits hash as they did before the
        // field existed (the goldens stand). A waiting producer differs from the same one not waiting by the tag.
        if *blocked {
            h.tag(1);
        }
        // (M5-C) The same for a Recruitment Hall's completion tick, only while it isn't 0, under a tag of its own.
        if *completed_at != 0 {
            h.tag(2);
            completed_at.hash(h);
        }
        // (M5-F2) The same for a pool producer's reservations, only while it has an entry.
        if !reserved.is_empty() {
            h.tag(3);
            reserved.hash(h);
        }
    }
}

#[cfg(test)]
mod tests;
