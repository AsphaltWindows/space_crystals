//! Pool producers in a game (slice M5-F2, plan decisions T70 and T71; `crate::production`'s module docs): a producer
//! that also holds units (`StructureDef::holds`), the Armory first, makes its units of the units inside
//! (`factions/cults/structures/armory.md > Training`). One path for every such type, its numbers its def's, so that
//! the Cathedral, the Chop Shop and the Airstrip reuse it as data.
//!
//! - **The units an order uses** ([`Sim::free_units`]): those of its kind (design change 0337, `crate::augmentation`:
//!   Plain, Constitution, Metabolism or Both) in the pool, neither reserved nor on their way out (`Sim::pool`), those
//!   that entered first (`factions/cults/structures/armory.md > Training`: "an order reserves the matching unit(s)
//!   that **entered the building first** (first in, first trained)").
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
//!   trains ([`Sim::train`]), a Release's unchanged; the entry then leaves the queue, and the next starts in the next
//!   tick, as at any producer.
//! - **X and the refunds** (`Sim::apply_production`, `Sim::refund_production`): an entry's Space Crystals go back to
//!   the player in full, and its units to the pool ([`Sim::unreserve`]): "returns its reserved Recruit to
//!   StoredRecruits" (`factions/cults/structures/armory.md > ObjectInterfaceState[Armory]`). A building that leaves the
//!   game refunds every entry but a finished one, whose unit comes out as the unit it trained, unrefunded (design
//!   change 0349 point 3), and the others' units come out as they are, Recruits (`sim/holding/evacuation.rs`).

use crate::augmentation::{Augmentation, RecruitKind};
use crate::defs::ObjectKind;
use crate::ids::ObjectId;
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
        (self.pool(building).into_iter())
            .filter(|held| held.object.augmentation.kind() == kind)
            .map(|held| held.object.id)
            .collect()
    }

    /// How many free units of each kind `building`'s pool has, in `RecruitKind::ALL`'s order: what the DefaultState's
    /// kind buttons count (design change 0337: "Each kind's button shows [...] the count inside (unreserved); it is
    /// greyed if there are none").
    pub fn pool_counts(&self, building: ObjectId) -> [u32; 4] {
        let mut counts = [0; 4];
        for held in self.pool(building) {
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
    ///   damage staying).
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
        let Some((hp, max_before)) = self.objects.get(&unit).map(|o| (o.hp, o.max_hp())) else {
            return;
        };
        self.change_kind(unit, kind, None);
        let Some(o) = self.objects.get_mut(&unit) else {
            return;
        };
        o.lineage.extend(joined);
        let infantry = is_infantry(kind);
        o.hp = match (hp, max_before, o.max_hp()) {
            (Some(hp), Some(before), Some(after)) if infantry => Some(transform::follow_max(hp, before, after)),
            (.., max) => max,
        };
        if !infantry {
            o.augmentation = Augmentation::NONE;
        }
        if let Some(worker) = &mut o.worker {
            worker.load = 0;
        }
    }
}

#[cfg(test)]
mod tests;
