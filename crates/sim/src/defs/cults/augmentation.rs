//! The Cults' augmentation numbers — `factions/cults/augmentation.md` (slice M5-N1a, plan decision T73): what one
//! increment of each kind does to a unit ([`IncrementEffects`]), and each Augmentation's increments at each Temple
//! research level ([`AugmentationTable`]). Data only: `crate::augmentation` derives a unit's increments from them and
//! its owner's levels, and its accessors apply them. Which kinds take them is each kind's own
//! (`UnitDef::augmentable`).

use serde::{Deserialize, Serialize};

use crate::augmentation::Increments;

/// `factions/cults/augmentation.md > Increments`: what one increment does. "Increments can be negative. A negative
/// increment applies the opposite effect".
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IncrementEffects {
    /// "**Constitution increment**: +10 MaxHP", every augmentable unit's.
    pub max_hp: f32,
    /// "+1 melee Damage (unarmed units only)".
    pub melee_damage: f32,
    /// "**Metabolism increment**: **+0.5 space units/frame MaxSpeed** (placeholder)", every augmentable unit's,
    /// before its mode's `Mover::speed_factor` (plan decision T73).
    pub max_speed: f32,
    /// "for unarmed units also **−1 frame of FiringDuration** per increment (placeholder ...; a negative increment
    /// adds 1 frame)".
    pub firing_ticks: u32,
    /// "FiringDuration never below 2 frames".
    pub min_firing_ticks: u32,
}

/// The two Augmentations' increments by research level (`factions/cults/augmentation.md > Augmentation Effects`: "Each
/// Augmentation grants a positive amount of its own increment and a negative amount of the other"), each row's three
/// columns "No research", "Level 1 researched" and "Level 2 researched", in that order (`crate::augmentation::Level`).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AugmentationTable {
    pub increments: IncrementEffects,
    /// Enhanced Constitution: "+2 Constitution / −1 Metabolism | +4 Constitution / −2 Metabolism | +6 Constitution /
    /// −3 Metabolism".
    pub enhanced_constitution: [Increments; 3],
    /// Improved Metabolism: "+2 Metabolism / −1 Constitution | +4 Metabolism / −2 Constitution | +6 Metabolism / −3
    /// Constitution".
    pub improved_metabolism: [Increments; 3],
}

/// The Cults' augmentation numbers (module docs), the built-in rules' (`CultsRules::augmentation`).
pub const AUGMENTATION: AugmentationTable = AugmentationTable {
    increments: IncrementEffects {
        max_hp: 10.0,
        melee_damage: 1.0,
        max_speed: 0.5,
        firing_ticks: 1,
        min_firing_ticks: 2,
    },
    enhanced_constitution: [Increments::new(2, -1), Increments::new(4, -2), Increments::new(6, -3)],
    improved_metabolism: [Increments::new(-1, 2), Increments::new(-2, 4), Increments::new(-3, 6)],
};
