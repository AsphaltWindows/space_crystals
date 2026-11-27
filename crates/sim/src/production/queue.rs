//! The entries of a producer's queue (`crate::production`): a unit to train, every producer's, a pool building's
//! Release (slice M5-F2, design change 0337: "Release is a new queue order type"), an Altar's augmentation order
//! (slice M5-N1b, plan decision T70: one queue engine), or a Temple's research (slice M5-N3).

use serde::{Deserialize, Serialize};

use crate::augmentation::AugmentOrder;
use crate::defs::{ObjectKind, ProductionDef, Tech};
use crate::economy::ArmyResource;
use crate::hash::{StateHash, StateHasher};
use crate::ruleset::rules;

/// An entry of a BuildQueue (`factions/gdo/structures/barracks.md > BarracksInstanceState`), of a pool building's
/// TrainingQueue (`factions/cults/structures/armory.md > ArmoryInstanceState`) or of an Altar's AugmentationQueue
/// (`factions/cults/augmentation.md > Altar > AltarInstanceState`), or of a Temple's research queue
/// (`factions/cults/structures/temple.md > Placeholder stats`: "research queue 5"). In files a unit is its kind alone,
/// as every entry was before Releases existed, so that the states saved then still load; a Release is `null`, an
/// augmentation order an object with its order (`{"order": "Both"}`), and a research one with its technology
/// (`{"tech": "EnhancedConstitutionL1"}`), which no kind's name can be taken for, nor each other.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QueueEntry {
    /// A unit of this kind: produced (the Barracks' Peacekeeper, a Recruitment Hall's Recruit), or trained from the
    /// units the entry reserved (`Production::reserved`; the Armory's Militant).
    Unit(ObjectKind),
    /// `factions/cults/structures/README.md > Choosing Recruits, Release and Eject All`: "**Release** (user,
    /// 2026-11-26): a **no-op order**: **free**, very fast (**4 frames**, placeholder), queued like any order; when it
    /// completes the reserved unit walks out of the exit **unchanged** and goes to the RallyPoint".
    Release,
    /// (M5-N1b) `factions/cults/augmentation.md > Altar > Augmentation Orders`: "Queuing an order immediately deducts
    /// its Space Crystals cost and reserves one unit of the selected type and AugmentationState from StoredUnits [...].
    /// When an order finishes, the unit's AugmentationState is updated [...], and the unit **automatically exits** from
    /// the exit side and moves to the RallyPoint": `order`, for the one unit the entry reserved.
    Augment { order: AugmentOrder },
    /// (M5-N3) `core/tech_and_research.md > Research > Shared Rules`: "each technology has a resource cost and a
    /// research time. The cost is deducted when the technology is queued"; "a research structure researches one
    /// technology at a time. Additional technologies can be queued behind it and are processed in queue order": `tech`,
    /// which reserves no unit (its staff are "not consumed").
    Research { tech: Tech },
}

impl QueueEntry {
    /// The kind of unit it produces or trains; `None` for a Release, an augmentation order or a research, which
    /// produce nothing.
    pub fn unit(self) -> Option<ObjectKind> {
        match self {
            QueueEntry::Unit(kind) => Some(kind),
            QueueEntry::Release | QueueEntry::Augment { .. } | QueueEntry::Research { .. } => None,
        }
    }

    /// (M5-N3) The technology it researches; `None` for every other entry.
    pub fn tech(self) -> Option<Tech> {
        match self {
            QueueEntry::Research { tech } => Some(tech),
            QueueEntry::Unit(_) | QueueEntry::Release | QueueEntry::Augment { .. } => None,
        }
    }

    /// Its frames at a `def` producer: its recipe's, a Release's `Production::release_frames`, its order's recipe's
    /// (`ProductionDef::augment`), or its technology's BaseResearchTime (`TechDef::frames`, the time with one Recruit
    /// inside). 0 for a unit or an order `def` has no recipe for.
    pub fn frames(self, def: &ProductionDef) -> u32 {
        match self {
            QueueEntry::Unit(kind) => def.recipe(kind).map_or(0, |recipe| recipe.frames),
            QueueEntry::Release => rules().tuning.production.release_frames,
            QueueEntry::Augment { order } => def.augment_recipe(order).map_or(0, |recipe| recipe.frames),
            QueueEntry::Research { tech } => tech.def().frames,
        }
    }

    /// Its Space Crystals cost at a `def` producer, paid when it is queued: its recipe's, its order's, or its
    /// technology's; a Release is "**free**".
    pub fn cost(self, def: &ProductionDef) -> u32 {
        match self {
            QueueEntry::Unit(kind) => def.recipe(kind).map_or(0, |recipe| recipe.cost),
            QueueEntry::Release => 0,
            QueueEntry::Augment { order } => def.augment_recipe(order).map_or(0, |recipe| recipe.cost),
            QueueEntry::Research { tech } => tech.def().cost,
        }
    }

    /// (M6-GD) Its Supplies cost at a `def` producer, paid with its Space Crystals when it is queued and refunded with
    /// them (`Recipe::supplies`; `factions/gdo/structures/barracks.md > ObjectInterfaceState[Barracks]`: "**R: Build
    /// Drone Operator** [...] deducts 100 Space Crystals and 1 Supply"): a unit's recipe's. Nothing else costs any.
    pub fn supplies(self, def: &ProductionDef) -> u32 {
        match self {
            QueueEntry::Unit(kind) => def.recipe(kind).map_or(0, |recipe| recipe.supplies),
            QueueEntry::Release | QueueEntry::Augment { .. } | QueueEntry::Research { .. } => 0,
        }
    }

    /// How many units it reserves at a building that trains the units inside it (`Production::reserved`, slice M5-F2):
    /// a unit's Unit Control cost, at least one (`factions/cults/structures/README.md > Cults Vehicles`: "a trained
    /// unit's Unit Control cost equals the number of Recruits consumed"; plan decision T70; a Seeker 2), and one for a
    /// Release or an augmentation order ("reserves one unit"). (M5-N3) None for a research: its staff are "not
    /// consumed" (`core/tech_and_research.md > Research > Faction Exceptions > Cults — research is staffed by
    /// Recruits`).
    pub fn units_reserved(self) -> usize {
        match self {
            QueueEntry::Unit(kind) => ArmyResource::CultsUnitControl.cost_of(kind).max(1) as usize,
            QueueEntry::Release | QueueEntry::Augment { .. } => 1,
            QueueEntry::Research { .. } => 0,
        }
    }
}

impl StateHash for QueueEntry {
    fn hash(&self, h: &mut StateHasher) {
        match self {
            // As the kind alone, which every entry was before Releases existed: the goldens stand.
            QueueEntry::Unit(kind) => kind.hash(h),
            // No kind's declaration index.
            QueueEntry::Release => u32::MAX.hash(h),
            // Neither, then the order.
            QueueEntry::Augment { order } => {
                (u32::MAX - 1).hash(h);
                order.hash(h);
            }
            // (M5-N3) None of these, then the technology.
            QueueEntry::Research { tech } => {
                (u32::MAX - 2).hash(h);
                tech.hash(h);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A unit is saved and hashed as its kind alone, as before Releases existed; a Release, an augmentation order
    /// (slice M5-N1b) and a research (slice M5-N3) apart from every kind and from each other.
    #[test]
    fn a_unit_entry_is_its_kind_in_files_and_the_hash() {
        let augment = QueueEntry::Augment {
            order: AugmentOrder::Both,
        };
        let research = QueueEntry::Research {
            tech: Tech::ImprovedMetabolismL2,
        };
        let entries = [
            QueueEntry::Unit(ObjectKind::Peacekeeper),
            QueueEntry::Release,
            augment,
            research,
        ];
        let json = serde_json::to_string(&entries).unwrap();
        assert_eq!(
            json,
            r#"["Peacekeeper",null,{"order":"Both"},{"tech":"ImprovedMetabolismL2"}]"#
        );
        assert_eq!(serde_json::from_str::<Vec<QueueEntry>>(&json).unwrap(), entries);
        let hash = |f: &dyn Fn(&mut StateHasher)| {
            let mut h = StateHasher::new();
            f(&mut h);
            h.finish()
        };
        let kind = ObjectKind::Peacekeeper;
        assert_eq!(hash(&|h| QueueEntry::Unit(kind).hash(h)), hash(&|h| kind.hash(h)));
        assert_ne!(hash(&|h| QueueEntry::Release.hash(h)), hash(&|h| kind.hash(h)));
        let hashes: Vec<u64> = [QueueEntry::Unit(kind), QueueEntry::Release]
            .into_iter()
            .chain(AugmentOrder::ALL.map(|order| QueueEntry::Augment { order }))
            .chain(Tech::ALL.map(|tech| QueueEntry::Research { tech }))
            .map(|entry| hash(&|h| entry.hash(h)))
            .collect();
        for (i, hash) in hashes.iter().enumerate() {
            assert!(!hashes[..i].contains(hash), "entry {i} hashes as one before it");
        }
    }
}
