//! (M5-M1) The augmentation invariant A1, the M5 plan's "only augmentable kinds carry an augmentation" (slice M): an
//! object, on the map or held off it (`Sim::held`), has an Augmentation only if its kind can be augmented
//! (`UnitDef::augmentable`, `augmentation::is_augmentable`; `factions/cults/augmentation.md > Eligibility`: "Only Cults
//! infantry can be augmented"). A unit loses its Augmentations as it changes into a kind that can't have them
//! (`Sim::change_kind`, `Sim::train`: "if augmented Recruits are consumed to produce a vehicle, their Augmentations are
//! lost", `factions/cults/augmentation.md > Persistence`), and gets them only through `augmentation::set`, which gives
//! none to such a kind: the Altar's orders as the unit comes out (`Sim::augment`, slice M5-N1b), and
//! `DebugCommand::SetAugmentation`, which refuses such a kind (`Reject::NotAugmentable`).

use super::{Violation, every_object};
use crate::augmentation;
use crate::sim::Sim;

/// A1: no object of a kind that can't be augmented has an Augmentation (module docs).
pub(super) fn augmentations(sim: &Sim) -> Result<(), Violation> {
    for (o, held) in every_object(sim) {
        if !o.augmentation.is_none() && !augmentation::is_augmentable(o.kind) {
            let held = if held { " (held)" } else { "" };
            return Err(Violation {
                invariant: "A1",
                message: format!(
                    "#{} ({:?}){held} has {:?}, and its kind can't be augmented",
                    o.id.0, o.kind, o.augmentation
                ),
                at: o.pos,
            });
        }
    }
    Ok(())
}
