//! A Cults infantry unit's Augmentations (`factions/cults/augmentation.md > AugmentationState (per unit instance)`:
//! "EnhancedConstitution: boolean", "ImprovedMetabolism: boolean"), and the four kinds of units that the buildings
//! holding them choose by them (design change 0337, `factions/cults/structures/README.md > Choosing Recruits, Release
//! and Eject All`: "The four **kinds** are: **Plain** (unaugmented), **Constitution** (Enhanced Constitution only),
//! **Metabolism** (Improved Metabolism only) and **Both**").
//!
//! Slice M5-F2 has the state, for the kinds ([`Augmentation::kind`]): a unit keeps it inside a building and when it is
//! trained into infantry (`factions/cults/augmentation.md > Persistence`: "a trained unit inherits its Recruit's
//! AugmentationState unchanged"), and loses it when it is trained into a vehicle or an aircraft ("if augmented Recruits
//! are consumed to produce a vehicle, their Augmentations are lost"; `Sim::train`). The Altar that gives it, and what
//! it does (`factions/cults/augmentation.md > Increments`), come with a later slice: until then only a test sets it.

use serde::{Deserialize, Serialize};

use crate::hash::{StateHash, StateHasher};

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

#[cfg(test)]
mod tests {
    use super::*;

    /// Design change 0337: the four kinds, from the two Augmentations, both ways.
    #[test]
    fn the_four_kinds_come_from_the_two_augmentations() {
        for kind in RecruitKind::ALL {
            assert_eq!(Augmentation::of(kind).kind(), kind);
        }
        assert_eq!(Augmentation::NONE.kind(), RecruitKind::Plain);
        assert!(Augmentation::default().is_none() && !Augmentation::of(RecruitKind::Both).is_none());
        let names: Vec<&str> = RecruitKind::ALL.iter().map(|kind| kind.name()).collect();
        assert_eq!(names, ["Plain", "Constitution", "Metabolism", "Both"]);
    }

    /// Left out of files while it has neither, so that states saved before it existed load.
    #[test]
    fn an_augmentation_saves_only_what_it_has() {
        let constitution = Augmentation::of(RecruitKind::Constitution);
        let json = serde_json::to_string(&constitution).unwrap();
        assert_eq!(json, r#"{"constitution":true}"#);
        assert_eq!(serde_json::from_str::<Augmentation>(&json).unwrap(), constitution);
        assert_eq!(serde_json::from_str::<Augmentation>("{}").unwrap(), Augmentation::NONE);
    }
}
