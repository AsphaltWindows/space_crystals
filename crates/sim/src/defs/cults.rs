//! The Cults — `factions/cults/`: its own rules, and one module per unit (`units/`, `factions/cults/units/*.md`) and
//! per structure (`structures/`: the buildings of `factions/cults/structures/*.md`, the Recruitment Hall, the Storage
//! and the Armory so far).
//!
//! Slice M5-A put the faction's six designed kinds in the game's model with their numbers (plan decision T55): the
//! Recruit, the Militant, the Gunner, the Recruitment Hall, the Storage and the Armory. Each system's slice adds its
//! own field to their defs (plan §3.2, "one field per slice"): the Recruit's gathering and building, the Hall's
//! production, the Armory's pool and training, and every structure's site. A Cults unit's Unit Control cost is the
//! Recruits consumed to make it (`factions/cults/structures/recruitment_hall.md > Unit Control Tracking`), kept in
//! `UnitDef::unit_control_cost`; each point of it counts against the Hall its Recruit came from (`Object::lineage`,
//! plan decision T56). No rule of the faction's own yet: the Recruits' Construct menu comes with slice M5-J1.

pub(crate) mod structures;
pub(crate) mod units;

#[cfg(test)]
mod tests;
