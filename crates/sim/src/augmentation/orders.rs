//! The Altar's orders (slice M5-N1b; `factions/cults/augmentation.md > Altar > Augmentation Orders`): "Order types:
//! **Enhanced Constitution**, **Improved Metabolism**, or **Both** (a single order that applies both). An order can
//! only target a unit that is missing every Augmentation the order applies."

use serde::{Deserialize, Serialize};

use super::{Augmentation, RecruitKind};
use crate::hash::{StateHash, StateHasher};

/// An order of the Altar's AugmentationQueue (`factions/cults/augmentation.md > Altar > AltarInstanceState`: "Order:
/// EnhancedConstitution | ImprovedMetabolism | Both"), which an entry applies to the unit it reserved
/// (`QueueEntry::Augment`), at its recipe's price and in its recipe's frames (`ProductionDef::augment`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AugmentOrder {
    /// Enhanced Constitution.
    EnhancedConstitution,
    /// Improved Metabolism.
    ImprovedMetabolism,
    /// "**Both** (a single order that applies both)".
    Both,
}

impl AugmentOrder {
    /// The three, in the order of their slots from Unaugmented (`factions/cults/augmentation.md > Altar >
    /// ObjectInterfaceState[Altar] > AugmentMenu[UnitType, State]`: "From Unaugmented: **Q: Enhanced Constitution**,
    /// **W: Improved Metabolism**, **E: Both**").
    pub const ALL: [AugmentOrder; 3] = [
        AugmentOrder::EnhancedConstitution,
        AugmentOrder::ImprovedMetabolism,
        AugmentOrder::Both,
    ];

    /// The Augmentations it applies.
    pub fn applies(self) -> Augmentation {
        let (constitution, metabolism) = match self {
            AugmentOrder::EnhancedConstitution => (true, false),
            AugmentOrder::ImprovedMetabolism => (false, true),
            AugmentOrder::Both => (true, true),
        };
        Augmentation {
            constitution,
            metabolism,
        }
    }

    /// Whether it may target a unit of kind `kind`: "An order can only target a unit that is missing every
    /// Augmentation the order applies". From Unaugmented the three, from Enhanced Constitution only Improved
    /// Metabolism, from Improved Metabolism only Enhanced Constitution, and none from Fully augmented ("only Release is
    /// possible").
    pub fn fits(self, kind: RecruitKind) -> bool {
        let (has, applies) = (Augmentation::of(kind), self.applies());
        !(has.constitution && applies.constitution || has.metabolism && applies.metabolism)
    }

    /// The Augmentations of a unit that had `before` once the order is done: those it had, and those the order applies
    /// ("When an order finishes, the unit's AugmentationState is updated").
    pub fn onto(self, before: Augmentation) -> Augmentation {
        let applies = self.applies();
        Augmentation {
            constitution: before.constitution || applies.constitution,
            metabolism: before.metabolism || applies.metabolism,
        }
    }

    /// Its name, as the design gives it.
    pub fn name(self) -> &'static str {
        match self {
            AugmentOrder::EnhancedConstitution => "Enhanced Constitution",
            AugmentOrder::ImprovedMetabolism => "Improved Metabolism",
            AugmentOrder::Both => "Both",
        }
    }
}

impl StateHash for AugmentOrder {
    fn hash(&self, h: &mut StateHasher) {
        // Fieldless enum: the discriminant is the declaration index.
        (*self as u32).hash(h);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use AugmentOrder::{Both, EnhancedConstitution, ImprovedMetabolism};

    /// `factions/cults/augmentation.md > Altar > ObjectInterfaceState[Altar] > AugmentMenu[UnitType, State]`: "From
    /// Unaugmented: **Q: Enhanced Constitution**, **W: Improved Metabolism**, **E: Both**"; "From Enhanced
    /// Constitution only: **W: Improved Metabolism**"; "From Improved Metabolism only: **Q: Enhanced Constitution**";
    /// from Fully augmented none, "only Release is possible". Each gives the unit what it had and what it applies.
    #[test]
    fn an_order_targets_a_unit_missing_every_augmentation_it_applies() {
        let fitting = |kind| -> Vec<AugmentOrder> { AugmentOrder::ALL.into_iter().filter(|o| o.fits(kind)).collect() };
        assert_eq!(
            fitting(RecruitKind::Plain),
            [EnhancedConstitution, ImprovedMetabolism, Both]
        );
        assert_eq!(fitting(RecruitKind::Constitution), [ImprovedMetabolism]);
        assert_eq!(fitting(RecruitKind::Metabolism), [EnhancedConstitution]);
        assert_eq!(fitting(RecruitKind::Both), []);
        for order in AugmentOrder::ALL {
            for kind in RecruitKind::ALL.into_iter().filter(|&kind| order.fits(kind)) {
                let after = order.onto(Augmentation::of(kind));
                let expected = match (kind, order) {
                    (RecruitKind::Plain, EnhancedConstitution) => RecruitKind::Constitution,
                    (RecruitKind::Plain, ImprovedMetabolism) => RecruitKind::Metabolism,
                    _ => RecruitKind::Both,
                };
                assert_eq!(after.kind(), expected, "{order:?} on {kind:?}");
            }
        }
        assert_eq!(
            AugmentOrder::ALL.map(AugmentOrder::name),
            ["Enhanced Constitution", "Improved Metabolism", "Both"]
        );
    }
}
