//! A Cults infantry unit's Augmentations (`factions/cults/augmentation.md > AugmentationState (per unit instance)`:
//! "EnhancedConstitution: boolean", "ImprovedMetabolism: boolean"), the four kinds of units that the buildings
//! holding them choose by them (design change 0337, `factions/cults/structures/README.md > Choosing Recruits, Release
//! and Eject All`: "The four **kinds** are: **Plain** (unaugmented), **Constitution** (Enhanced Constitution only),
//! **Metabolism** (Improved Metabolism only) and **Both**"), and what they do.
//!
//! - **The state** (slice M5-F2): which Augmentations a unit has ([`Augmentation`]), and its kind
//!   ([`Augmentation::kind`]). A unit keeps it inside a building and through a change to another augmentable kind,
//!   such as its training into infantry (`factions/cults/augmentation.md > Persistence`: "a trained unit inherits its
//!   Recruit's AugmentationState unchanged"), and loses it through a change to any other ("if augmented Recruits are
//!   consumed to produce a vehicle, their Augmentations are lost"; `Sim::change_kind`).
//! - **The amounts** (slice M5-N1a, design change 0336): from its owner's current research levels, read whenever a
//!   stat is ([`Research`], [`Levels`]; `levels.rs`). No research yet: every amount is the table's "No research"
//!   column until the Temple (slice M5-N3).
//! - **The effects** (slice M5-N1a, plan decision T73; `stats.rs`): a unit's MaxHP, speed and weapon are read through
//!   accessors that apply its net increments ([`Increments`]): `Object::max_hp_with`, `Object::movement`,
//!   `Object::max_speed` and `combat::weapon`. Its HP follows its MaxHP when its Augmentations change ([`follow_max`]):
//!   the Altar sets them as its orders complete (slice M5-N1b, `Sim::augment`), and `DebugCommand::SetAugmentation`
//!   for tests and QA runs.
//!   `tests/augmentation/raw_reads.rs` keeps the stats' raw reads out of the code outside the accessors
//!   (`docs/architecture/object-model-state.md > Object model > Augmented stats`).
//! - **The Altar's orders** (slice M5-N1b, `orders.rs`): Enhanced Constitution, Improved Metabolism or Both
//!   ([`AugmentOrder`]), each for a unit missing every Augmentation it applies.

mod levels;
mod orders;
mod stats;
#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

use crate::hash::{StateHash, StateHasher};

pub use levels::{Level, Levels, Research};
pub use orders::AugmentOrder;
pub use stats::{Increments, follow_max, is_augmentable};
pub(crate) use stats::{augmented, max_hp_of, set};

/// `factions/cults/augmentation.md > AugmentationState (per unit instance)`: which of the two Augmentations a unit has
/// ("A unit may have neither, either, or both Augmentations"). The default, neither, is every unit's but a Cults
/// infantry unit's that the Altar augmented.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Augmentation {
    /// EnhancedConstitution.
    #[serde(default, skip_serializing_if = "is_false")]
    pub constitution: bool,
    /// ImprovedMetabolism.
    #[serde(default, skip_serializing_if = "is_false")]
    pub metabolism: bool,
}

fn is_false(b: &bool) -> bool {
    !*b
}

impl Augmentation {
    /// Neither Augmentation: a Plain unit.
    pub const NONE: Augmentation = Augmentation {
        constitution: false,
        metabolism: false,
    };

    /// Whether it has neither (`Object::augmentation`: left out of files and the hash then).
    #[inline]
    pub fn is_none(&self) -> bool {
        *self == Augmentation::NONE
    }

    /// The kind of unit it makes (design change 0337).
    pub fn kind(self) -> RecruitKind {
        match (self.constitution, self.metabolism) {
            (false, false) => RecruitKind::Plain,
            (true, false) => RecruitKind::Constitution,
            (false, true) => RecruitKind::Metabolism,
            (true, true) => RecruitKind::Both,
        }
    }

    /// The Augmentations of a unit of `kind`: its kind's, as [`Augmentation::kind`] reads them back.
    pub fn of(kind: RecruitKind) -> Augmentation {
        let (constitution, metabolism) = match kind {
            RecruitKind::Plain => (false, false),
            RecruitKind::Constitution => (true, false),
            RecruitKind::Metabolism => (false, true),
            RecruitKind::Both => (true, true),
        };
        Augmentation {
            constitution,
            metabolism,
        }
    }
}

impl StateHash for Augmentation {
    fn hash(&self, h: &mut StateHasher) {
        let Augmentation {
            constitution,
            metabolism,
        } = self;
        constitution.hash(h);
        metabolism.hash(h);
    }
}

/// The kind of a unit in a building that holds units, by its Augmentations (design change 0337,
/// `factions/cults/structures/README.md > Choosing Recruits, Release and Eject All`): what a pool building's
/// DefaultState chooses first (Q, W, E, R), and its orders then use units of (`ProductionCommand::TrainFrom`,
/// `ProductionCommand::Release`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RecruitKind {
    /// Unaugmented.
    Plain,
    /// Enhanced Constitution only.
    Constitution,
    /// Improved Metabolism only.
    Metabolism,
    /// Both Augmentations.
    Both,
}

impl RecruitKind {
    /// The four, in the order of their buttons: "**Q** Plain, **W** Constitution, **E** Metabolism, **R** Both".
    pub const ALL: [RecruitKind; 4] = [
        RecruitKind::Plain,
        RecruitKind::Constitution,
        RecruitKind::Metabolism,
        RecruitKind::Both,
    ];

    /// Its name, as the design gives it.
    pub fn name(self) -> &'static str {
        match self {
            RecruitKind::Plain => "Plain",
            RecruitKind::Constitution => "Constitution",
            RecruitKind::Metabolism => "Metabolism",
            RecruitKind::Both => "Both",
        }
    }
}

impl StateHash for RecruitKind {
    fn hash(&self, h: &mut StateHasher) {
        // Fieldless enum: the discriminant is the declaration index.
        (*self as u32).hash(h);
    }
}
