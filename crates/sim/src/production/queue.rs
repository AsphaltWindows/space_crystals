//! The entries of a producer's queue (`crate::production`): a unit to train, every producer's, or a pool building's
//! Release (slice M5-F2, design change 0337: "Release is a new queue order type").

use serde::{Deserialize, Serialize};

use crate::defs::{ObjectKind, ProductionDef};
use crate::economy::ArmyResource;
use crate::hash::{StateHash, StateHasher};
use crate::tuning::production::RELEASE_FRAMES;

/// An entry of a BuildQueue (`factions/gdo/structures/barracks.md > BarracksInstanceState`) or of a pool building's
/// TrainingQueue (`factions/cults/structures/armory.md > ArmoryInstanceState`). In files a unit is its kind alone, as
/// every entry was before Releases existed, so that the states saved then still load; a Release is `null`.
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
}

impl QueueEntry {
    /// The kind of unit it produces or trains; `None` for a Release, which produces nothing.
    pub fn unit(self) -> Option<ObjectKind> {
        match self {
            QueueEntry::Unit(kind) => Some(kind),
            QueueEntry::Release => None,
        }
    }

    /// Its frames at a `def` producer: its recipe's, or a Release's [`RELEASE_FRAMES`]. 0 for a unit `def` has no
    /// recipe for.
    pub fn frames(self, def: &ProductionDef) -> u32 {
        match self {
            QueueEntry::Unit(kind) => def.recipe(kind).map_or(0, |recipe| recipe.frames),
            QueueEntry::Release => RELEASE_FRAMES,
        }
    }

    /// Its Space Crystals cost at a `def` producer, paid when it is queued: its recipe's; a Release is "**free**".
    pub fn cost(self, def: &ProductionDef) -> u32 {
        match self {
            QueueEntry::Unit(kind) => def.recipe(kind).map_or(0, |recipe| recipe.cost),
            QueueEntry::Release => 0,
        }
    }

    /// How many units it reserves at a building that trains the units inside it (`Production::reserved`, slice M5-F2):
    /// a unit's Unit Control cost, at least one (`factions/cults/structures/README.md > Cults Vehicles`: "a trained
    /// unit's Unit Control cost equals the number of Recruits consumed"; plan decision T70; a Seeker 2), and one for a
    /// Release.
    pub fn units_reserved(self) -> usize {
        match self {
            QueueEntry::Unit(kind) => ArmyResource::CultsUnitControl.cost_of(kind).max(1) as usize,
            QueueEntry::Release => 1,
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A unit is saved and hashed as its kind alone, as before Releases existed; a Release apart from every kind.
    #[test]
    fn a_unit_entry_is_its_kind_in_files_and_the_hash() {
        let entries = [QueueEntry::Unit(ObjectKind::Peacekeeper), QueueEntry::Release];
        let json = serde_json::to_string(&entries).unwrap();
        assert_eq!(json, r#"["Peacekeeper",null]"#);
        assert_eq!(serde_json::from_str::<Vec<QueueEntry>>(&json).unwrap(), entries);
        let hash = |f: &dyn Fn(&mut StateHasher)| {
            let mut h = StateHasher::new();
            f(&mut h);
            h.finish()
        };
        let kind = ObjectKind::Peacekeeper;
        assert_eq!(hash(&|h| QueueEntry::Unit(kind).hash(h)), hash(&|h| kind.hash(h)));
        assert_ne!(hash(&|h| QueueEntry::Release.hash(h)), hash(&|h| kind.hash(h)));
    }
}
