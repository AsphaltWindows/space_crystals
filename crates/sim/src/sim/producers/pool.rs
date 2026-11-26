//! Pool producers in a game (slice M5-F2, plan decisions T70 and T71; `crate::production`'s module docs): a producer
//! that also holds units (`StructureDef::holds`), the Armory first, makes its units of the units inside
//! (`factions/cults/structures/armory.md > Training`). One path for every such type, its numbers its def's, so that
//! the Cathedral, the Chop Shop and the Airstrip reuse it as data.
//!
//! - **The units an order uses** ([`Sim::free_units`]): those of its kind (design change 0337, `crate::augmentation`:
//!   Plain, Constitution, Metabolism or Both) in the pool, neither reserved nor on their way out (`Sim::pool`), those
//!   that entered first (`factions/cults/structures/armory.md > Training`: "an order reserves the matching unit(s)
//!   that **entered the building first** (first in, first trained)"); at the Altar, which holds four types, of its type
//!   too ([`Sim::free_units_of`], slice M5-N1b).
//! - **Queuing** (`ProductionCommand::TrainFrom`, `ProductionCommand::Release`; the gate is `sim/producers.rs`'s): an
//!   order pays its Space Crystals, a Release nothing, and reserves its units at once (`Production::reserved`): as
//!   many as the unit's Unit Control cost (`QueueEntry::units_reserved`), one for a Release. Without enough free units
//!   of its kind it
//!   is refused (`Refusal::NoRecruit`). The capacity alone bounds the queue, the reserved units being inside (design
//!   change 0347 point 5: "Queues are limited by capacity, not by a separate 20").
//! - **Training** is serial, the front entry alone advancing, a frame a tick whatever the power (the Cults have none).
//!   Finished, its first unit is ready to leave from that tick ([`Sim::ready_to_leave`], `Held::leaving`), and the
//!   entry waits at 100% while units ready before it go first.
//! - **Leaving** (`Sim::step_holding`, `sim/holding.rs`): the building lets its ready units out one at a time through
//!   its exit doorway, in the order they are ready (design change 0348), a training's unit coming out as the unit it
//!   trains ([`Sim::train`]), an augmentation order's augmented ([`Sim::augment`], slice M5-N1b), a Release's
//!   unchanged ([`Sim::complete`]); the entry then leaves the queue, and the next starts in the next tick, as at any
//!   producer.
//! - **X and the refunds** (`Sim::apply_production`, `Sim::refund_production`): an entry's Space Crystals go back to
//!   the player in full, and its units to the pool ([`Sim::unreserve`]): "returns its reserved Recruit to
//!   StoredRecruits" (`factions/cults/structures/armory.md > ObjectInterfaceState[Armory]`). A building that leaves the
//!   game refunds every entry but a finished one, whose unit comes out as the unit it trained, unrefunded (design
//!   change 0349 point 3), or augmented, and the others' units come out as they are, Recruits
//!   (`sim/holding/evacuation.rs`).

use crate::augmentation::{self, AugmentOrder, Augmentation, RecruitKind, Research};
use crate::defs::ObjectKind;
use crate::ids::ObjectId;
use crate::object::Object;
use crate::production::QueueEntry;
use crate::sim::Sim;
use crate::transform;

/// Whether units of `kind` are infantry (`systems/units.md > UnitBase`: LightInfantry and HeavyInfantry,
/// `UnitBase::is_infantry`), which keep the damage and the Augmentations of the unit they are trained from
/// ([`Sim::train`]).
fn is_infantry(kind: ObjectKind) -> bool {
    (kind.def().unit.as_ref()).is_some_and(|unit| unit.base.is_infantry())
}

impl Sim {
    /// The free units of kind `kind` in `building`'s pool, longest inside first (module docs): those an order of that
    /// kind takes, in that order. None for anything but a building that holds units.
    pub fn free_units(&self, building: ObjectId, kind: RecruitKind) -> Vec<ObjectId> {
        self.free_where(building, |o| o.augmentation.kind() == kind)
    }

    /// (M5-N1b) The free units of type `unit` and kind `kind` in `building`'s pool, longest inside first: those an
    /// Altar's order or Release for them takes, in that order (`factions/cults/augmentation.md > Altar > Augmentation
    /// Orders`: "reserves one unit of the selected type and AugmentationState from StoredUnits").
    pub fn free_units_of(&self, building: ObjectId, unit: ObjectKind, kind: RecruitKind) -> Vec<ObjectId> {
        self.free_where(building, |o| o.kind == unit && o.augmentation.kind() == kind)
    }

    fn free_where(&self, building: ObjectId, keep: impl Fn(&Object) -> bool) -> Vec<ObjectId> {
        (self.pool(building).into_iter())
            .filter(|held| keep(&held.object))
            .map(|held| held.object.id)
            .collect()
    }

    /// How many free units of each kind `building`'s pool has, in `RecruitKind::ALL`'s order: what the DefaultState's
    /// kind buttons count (design change 0337: "Each kind's button shows [...] the count inside (unreserved); it is
    /// greyed if there are none").
    pub fn pool_counts(&self, building: ObjectId) -> [u32; 4] {
        self.counts_where(building, |_| true)
    }

    /// (M5-N1b) How many free units of type `unit` of each kind `building`'s pool has, in `RecruitKind::ALL`'s order:
    /// what an Altar's AugmentMenu\[UnitType\] counts (`factions/cults/augmentation.md > Altar >
    /// ObjectInterfaceState[Altar]`: "Each tile shows the count of unreserved units of that type and state in
    /// StoredUnits; disabled if there are none"), and their sum its DefaultState's tile of the type.
    pub fn pool_counts_of(&self, building: ObjectId, unit: ObjectKind) -> [u32; 4] {
        self.counts_where(building, |o| o.kind == unit)
    }

    fn counts_where(&self, building: ObjectId, keep: impl Fn(&Object) -> bool) -> [u32; 4] {
        let mut counts = [0; 4];
        for held in self.pool(building).into_iter().filter(|held| keep(&held.object)) {
            counts[held.object.augmentation.kind() as usize] += 1;
        }
        counts
    }

    /// The units `units` are reserved no longer (X, a refund): back in their building's pool, the one an entry had
    /// made ready to leave staying inside.
    pub(in crate::sim) fn unreserve(&mut self, units: &[ObjectId]) {
        for unit in units {
            if let Some(held) = self.held.get_mut(unit) {
                held.leaving = None;
            }
        }
    }

    /// What a finished entry `entry` does to its first unit, `unit`, as the unit comes out of the building, let out
    /// (`Sim::let_out`) or evacuated (`Sim::evacuate`): a training's becomes the unit it trains, of the units `consumed`
    /// too ([`Sim::train`]), an augmentation order's gains its Augmentations ([`Sim::augment`], slice M5-N1b), and a
    /// Release's stays as it is.
    pub(in crate::sim) fn complete(&mut self, unit: ObjectId, entry: QueueEntry, consumed: &[ObjectId]) {
        match entry {
            QueueEntry::Unit(kind) => self.train(unit, kind, consumed),
            QueueEntry::Augment { order } => self.augment(unit, order),
            QueueEntry::Release => {}
        }
    }

    /// Pool producer `building`'s front entry is finished, in the producers' step: its first unit is ready to leave
    /// from this tick, if it wasn't already (module docs).
    pub(in crate::sim) fn ready_to_leave(&mut self, building: ObjectId) {
        let tick = self.tick;
        let first = (self.objects.get(&building))
            .and_then(|o| o.production.as_ref())
            .and_then(|production| production.reserved_by_current().first().copied());
        if let Some(held) = first.and_then(|unit| self.held.get_mut(&unit)) {
            held.leaving.get_or_insert(tick);
        }
    }

    /// `factions/cults/structures/armory.md > Training`: "When an order finishes training, its reserved Recruit is
    /// consumed and the trained unit exits from the exit side". Unit `unit`, just out in the doorway
    /// (`Sim::step_holding`), becomes a `kind`, the same object (plan decision T15, `Sim::change_kind`: its id and its
    /// lineage), and the units `consumed`, the others its entry reserved, still held, leave the game into it, without
    /// a death, their lineages joining its own (plan decision T56 as amended, `Object::lineage`: one entry per Recruit
    /// consumed, so that the Halls' usage stays as it was).
    /// - **HP** (`> Training`): "infantry trained from a Recruit **keep the Recruit's damage** (the missing HP carries
    ///   over); vehicles and aircraft come out at full HP" (`transform::follow_max`: the HP moves with the MaxHP, the
    ///   damage staying). (M5-N1a) The MaxHP before and after are the unit's with its Augmentations
    ///   (`Object::max_hp_with`): a Constitution Recruit's 60, a vehicle's own.
    /// - **Augmentations** (`factions/cults/augmentation.md > Persistence`): infantry keep them ("a trained unit
    ///   inherits its Recruit's AugmentationState unchanged"), the others lose them ("if augmented Recruits are
    ///   consumed to produce a vehicle, their Augmentations are lost").
    /// - **A load** goes (design change 0347 point 3, `factions/cults/structures/README.md > Choosing Recruits,
    ///   Release and Eject All`: "a Recruit consumed by training loses it").
    pub(in crate::sim) fn train(&mut self, unit: ObjectId, kind: ObjectKind, consumed: &[ObjectId]) {
        let joined: Vec<ObjectId> = (consumed.iter())
            .filter_map(|id| self.held.get(id))
            .flat_map(|held| held.object.lineage.iter().copied())
            .collect();
        self.remove_held(consumed);
        let research = self.research();
        let Some((hp, max_before)) = self.objects.get(&unit).map(|o| (o.hp, o.max_hp_with(research))) else {
            return;
        };
        self.change_kind(unit, kind, None);
        let research = Research::new(&self.players);
        let Some(o) = self.objects.get_mut(&unit) else {
            return;
        };
        o.lineage.extend(joined);
        let infantry = is_infantry(kind);
        if !infantry {
            o.augmentation = Augmentation::NONE;
        }
        o.hp = match (hp, max_before, o.max_hp_with(research)) {
            (Some(hp), Some(before), Some(after)) if infantry => Some(transform::follow_max(hp, before, after)),
            (.., max) => max,
        };
        if let Some(worker) = &mut o.worker {
            worker.load = 0;
        }
    }

    /// (M5-N1b) `factions/cults/augmentation.md > Altar > Augmentation Orders`: "When an order finishes, the unit's
    /// AugmentationState is updated (its amounts follow the current research level; see Augmentation > Research
    /// applies to everyone)". Unit `unit`, just out ([`Sim::complete`]), gains the Augmentations `order` applies
    /// (`AugmentOrder::onto`), its HP following its MaxHP (`augmentation::set`, design changes 0330 and 0331: a rise
    /// keeps its damage, a fall caps its HP). Its owner's levels are read whenever its stats are, so none is stamped
    /// on it (design change 0336: "Temple research is **retroactive**").
    pub(in crate::sim) fn augment(&mut self, unit: ObjectId, order: AugmentOrder) {
        let research = Research::new(&self.players);
        if let Some(o) = self.objects.get_mut(&unit) {
            let augmentation = order.onto(o.augmentation);
            augmentation::set(o, augmentation, research);
        }
    }
}

#[cfg(test)]
mod tests;
