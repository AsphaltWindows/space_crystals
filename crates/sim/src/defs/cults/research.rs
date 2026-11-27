//! The Cults' research (slice M5-N3, plan decision T74): `factions/cults/structures/temple.md > Research (first set)`,
//! "Each Augmentation has its own two-level improvement line. Each level raises the effect of that Augmentation for
//! **every** unit that has it, including units already augmented". The technologies ([`Tech`]), the two lines they
//! form ([`ResearchLine`]) and each one's numbers ([`TechDef`]): data only. The Temple offers them
//! (`StructureDef::research`), a player holds the completed ones (`Player::techs`), and the amounts of the
//! Augmentations follow them (`augmentation::Levels::of`).

use serde::{Deserialize, Serialize};

use crate::augmentation::{Augmentation, Level};
use crate::defs::ObjectKind;
use crate::hash::{StateHash, StateHasher};

/// A technology (`core/tech_and_research.md > Research`: "technologies that a player pays for once at that structure
/// and that then apply to that player's eligible objects"): the four of the Temple's table, each line's two levels, "a
/// chain of separate technologies, each requiring the previous level" (`core/tech_and_research.md > Research > Shared
/// Rules` 5). Its numbers are its [`TechDef`] ([`Tech::def`]). In files by its name, hashed by its place in the list.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Tech {
    /// "Enhanced Constitution Level 1".
    EnhancedConstitutionL1,
    /// "Enhanced Constitution Level 2".
    EnhancedConstitutionL2,
    /// "Improved Metabolism Level 1".
    ImprovedMetabolismL1,
    /// "Improved Metabolism Level 2".
    ImprovedMetabolismL2,
}

impl Tech {
    /// The four, in the order of the Temple's table.
    pub const ALL: [Tech; 4] = [
        Tech::EnhancedConstitutionL1,
        Tech::EnhancedConstitutionL2,
        Tech::ImprovedMetabolismL1,
        Tech::ImprovedMetabolismL2,
    ];

    /// Its numbers, in the current rules (`crate::ruleset::rules`).
    #[inline]
    pub fn def(self) -> &'static TechDef {
        &crate::ruleset::rules().factions.cults.techs[self]
    }
}

impl crate::ruleset::Key for Tech {
    const ALL: &'static [Tech] = &Tech::ALL;

    #[inline]
    fn index(self) -> usize {
        self as usize
    }
}

/// Every technology's numbers, by technology (`CultsRules::techs`).
pub type Techs = crate::ruleset::Table<Tech, TechDef, 4>;

/// The built-in technologies' numbers, in [`Tech::ALL`]'s order.
pub const TECHS: Techs = Techs::new([
    ENHANCED_CONSTITUTION_L1,
    ENHANCED_CONSTITUTION_L2,
    IMPROVED_METABOLISM_L1,
    IMPROVED_METABOLISM_L2,
]);

impl StateHash for Tech {
    fn hash(&self, h: &mut StateHasher) {
        // Fieldless enum: the discriminant is the declaration index.
        (*self as u32).hash(h);
    }
}

/// A research line (`factions/cults/structures/temple.md > Research (first set)`: "Each Augmentation has its own
/// two-level improvement line"): the Augmentation whose amounts its levels raise (`factions/cults/augmentation.md >
/// Augmentation Effects`, a row of the table).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ResearchLine {
    EnhancedConstitution,
    ImprovedMetabolism,
}

impl ResearchLine {
    /// The two, in the order of the Temple's keys: "**Q** Enhanced Constitution (next level), **W** Improved
    /// Metabolism (next level)".
    pub const ALL: [ResearchLine; 2] = [ResearchLine::EnhancedConstitution, ResearchLine::ImprovedMetabolism];

    /// Its two technologies, Level 1 first.
    pub fn techs(self) -> [Tech; 2] {
        match self {
            ResearchLine::EnhancedConstitution => [Tech::EnhancedConstitutionL1, Tech::EnhancedConstitutionL2],
            ResearchLine::ImprovedMetabolism => [Tech::ImprovedMetabolismL1, Tech::ImprovedMetabolismL2],
        }
    }

    /// Whether a unit with `augmentation` has the line's Augmentation: the units whose amounts a level of it changes
    /// (`factions/cults/augmentation.md > Research applies to everyone`: "**every unit that has that Augmentation**
    /// gets the new amounts at once").
    pub fn improves(self, augmentation: Augmentation) -> bool {
        match self {
            ResearchLine::EnhancedConstitution => augmentation.constitution,
            ResearchLine::ImprovedMetabolism => augmentation.metabolism,
        }
    }

    /// Its name, its Augmentation's.
    pub fn name(self) -> &'static str {
        match self {
            ResearchLine::EnhancedConstitution => "Enhanced Constitution",
            ResearchLine::ImprovedMetabolism => "Improved Metabolism",
        }
    }
}

/// A technology's numbers (plan decision T74): `factions/cults/structures/temple.md > Research (first set)`, a row of
/// its table, and `> Placeholder stats`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TechDef {
    /// Its name, as the table has it.
    #[serde(deserialize_with = "crate::defs::leak::str")]
    pub name: crate::defs::StaticStr,
    /// The line it is a level of.
    pub line: ResearchLine,
    /// The level of its line it gives once completed: the effects table's column (`augmentation::Level`).
    pub level: Level,
    /// Space Crystals, "deducted when the technology is queued" (`core/tech_and_research.md > Research > Shared Rules`
    /// 2), refunded in full by a cancel or the structure's destruction (6 and 7).
    pub cost: u32,
    /// Its BaseResearchTime, frames: "the time with exactly 1 Recruit inside" (`core/tech_and_research.md > Faction
    /// Exceptions > Cults — research is staffed by Recruits`), which "scales down linearly with the number of Recruits
    /// inside".
    pub frames: u32,
    /// The technology it requires, the previous level of its line (`core/tech_and_research.md > Research > Shared
    /// Rules` 5: "each requiring the previous level"), which must have **completed** before it can be queued
    /// (`core/tech_and_research.md > Tech Prerequisites > Research Prerequisites`: "A next level can only be queued
    /// once the previous level has **completed**"). `None` for a Level 1.
    pub requires: Option<Tech>,
    /// The structure types its player must own when it is queued, each at least once (`core/tech_and_research.md > Tech
    /// Prerequisites > Research Prerequisites`: "**checked only when it is queued**"; `crate::tech`'s counting rule).
    /// Empty for none.
    #[serde(deserialize_with = "crate::defs::leak::slice")]
    pub requires_structures: &'static [ObjectKind],
}

/// "Enhanced Constitution Level 1 | — | Enhanced Constitution becomes +4 Constitution / −2 Metabolism | 100 SC / 960
/// frames (placeholder)"; `> Placeholder stats`: "Enhanced Constitution L1 and Improved Metabolism L1: **100 SC, 960
/// frames (60 s)** each".
const ENHANCED_CONSTITUTION_L1: TechDef = TechDef {
    name: "Enhanced Constitution Level 1",
    line: ResearchLine::EnhancedConstitution,
    level: Level::L1,
    cost: 100,
    frames: 960,
    requires: None,
    requires_structures: &[],
};

/// "Enhanced Constitution Level 2 | Enhanced Constitution Level 1; player owns at least one Cathedral | Enhanced
/// Constitution becomes +6 Constitution / −3 Metabolism | 150 SC / 1440 frames (placeholder)"; `> Placeholder stats`:
/// "L2 of each: **150 SC, 1440 frames (90 s)**".
const ENHANCED_CONSTITUTION_L2: TechDef = TechDef {
    name: "Enhanced Constitution Level 2",
    line: ResearchLine::EnhancedConstitution,
    level: Level::L2,
    cost: 150,
    frames: 1440,
    requires: Some(Tech::EnhancedConstitutionL1),
    requires_structures: &[ObjectKind::Cathedral],
};

/// "Improved Metabolism Level 1 | — | Improved Metabolism becomes +4 Metabolism / −2 Constitution | 100 SC / 960 frames
/// (placeholder)".
const IMPROVED_METABOLISM_L1: TechDef = TechDef {
    name: "Improved Metabolism Level 1",
    line: ResearchLine::ImprovedMetabolism,
    level: Level::L1,
    cost: 100,
    frames: 960,
    requires: None,
    requires_structures: &[],
};

/// "Improved Metabolism Level 2 | Improved Metabolism Level 1; player owns at least one Cathedral | Improved Metabolism
/// becomes +6 Metabolism / −3 Constitution | 150 SC / 1440 frames (placeholder)".
const IMPROVED_METABOLISM_L2: TechDef = TechDef {
    name: "Improved Metabolism Level 2",
    line: ResearchLine::ImprovedMetabolism,
    level: Level::L2,
    cost: 150,
    frames: 1440,
    requires: Some(Tech::ImprovedMetabolismL1),
    requires_structures: &[ObjectKind::Cathedral],
};

#[cfg(test)]
mod tests {
    use super::*;

    /// `factions/cults/structures/temple.md > Research (first set)` and `> Placeholder stats`: the four technologies,
    /// each line's Level 1 for 100 Space Crystals and 960 frames, and its Level 2, behind its Level 1 and a Cathedral
    /// ("Level 2 of both lines is researched at the Temple, but also requires the player to own a Cathedral"), for 150
    /// and 1440. Each is its line's level, and its line has it at that level.
    #[test]
    fn the_technologies_follow_the_design() {
        let row = |tech: Tech| {
            let def = tech.def();
            (def.name, def.cost, def.frames, def.requires, def.requires_structures)
        };
        let cathedral: &[ObjectKind] = &[ObjectKind::Cathedral];
        assert_eq!(
            Tech::ALL.map(row),
            [
                ("Enhanced Constitution Level 1", 100, 960, None, &[][..]),
                (
                    "Enhanced Constitution Level 2",
                    150,
                    1440,
                    Some(Tech::EnhancedConstitutionL1),
                    cathedral
                ),
                ("Improved Metabolism Level 1", 100, 960, None, &[][..]),
                (
                    "Improved Metabolism Level 2",
                    150,
                    1440,
                    Some(Tech::ImprovedMetabolismL1),
                    cathedral
                ),
            ]
        );
        for line in ResearchLine::ALL {
            let [l1, l2] = line.techs();
            assert_eq!((l1.def().line, l1.def().level), (line, Level::L1));
            assert_eq!((l2.def().line, l2.def().level), (line, Level::L2));
            assert_eq!(l2.def().requires, Some(l1), "each requires the previous level");
            assert!(l1.def().name.starts_with(line.name()));
        }
        let constitution = Augmentation::of(crate::augmentation::RecruitKind::Constitution);
        let both = Augmentation::of(crate::augmentation::RecruitKind::Both);
        assert!(ResearchLine::EnhancedConstitution.improves(constitution));
        assert!(!ResearchLine::ImprovedMetabolism.improves(constitution));
        assert!(ResearchLine::ALL.iter().all(|line| line.improves(both)));
        assert!(!ResearchLine::ALL.iter().any(|line| line.improves(Augmentation::NONE)));
    }
}
