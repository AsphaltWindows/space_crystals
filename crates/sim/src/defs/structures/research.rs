//! (M5-N3) The research capability of structure types (`StructureDef::research`, plan decision T74;
//! `core/tech_and_research.md > Research`: "A structure type may offer **Research**: technologies that a player pays for
//! once at that structure and that then apply to that player's eligible objects"): which technologies a type offers,
//! and how many its queue holds. The technologies themselves are `defs/cults/research.rs`'s.

use serde::{Deserialize, Serialize};

use crate::defs::Tech;

/// What a research structure researches (`StructureDef::research`): the Temple's
/// (`factions/cults/structures/temple.md > Research (first set)`). Its research runs on its production queue (plan
/// decision T70, `QueueEntry::Research`), so a type with one has a `ProductionDef` too, which produces no unit
/// (`defs_are_consistent`); a type that also holds units researches only while its staff are inside, as fast as their
/// number (`core/tech_and_research.md > Research > Faction Exceptions > Cults — research is staffed by Recruits`;
/// `Sim::step_production`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResearchDef {
    /// The technologies it offers, in the order of the design's table.
    #[serde(deserialize_with = "crate::defs::leak::slice")]
    pub techs: &'static [Tech],
    /// Its research queue: how many technologies it holds queued, the one in progress included
    /// (`factions/cults/structures/temple.md > Placeholder stats`: "research queue 5"; plan decision T70: "5 for the
    /// Temple").
    pub queue: usize,
}

impl ResearchDef {
    /// Whether it offers `tech`.
    pub fn offers(&self, tech: Tech) -> bool {
        self.techs.contains(&tech)
    }
}
