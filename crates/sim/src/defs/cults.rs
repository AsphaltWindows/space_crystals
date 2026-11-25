//! The Cults — `factions/cults/`: its own rules, and one module per unit (`units/`, `factions/cults/units/*.md`) and
//! per structure (`structures/`: the economy buildings of `factions/cults/economy.md`, the Recruitment Hall and the
//! Storage, and the buildings of `factions/cults/structures/*.md`, the Armory so far).
//!
//! Slice M5-A put the faction's six designed kinds in the game's model with their numbers (plan decision T55): the
//! Recruit, the Militant, the Gunner, the Recruitment Hall, the Storage and the Armory. Each system's slice adds its
//! own field to their defs (plan §3.2, "one field per slice"): the Recruit's gathering and building, the Hall's
//! production, the Armory's pool and training, and every structure's site. A Cults unit's Unit Control cost is the
//! Recruits consumed to make it (`factions/cults/economy.md > RecruitmentHall > Unit Control Tracking`), kept in
//! `UnitDef::unit_control_cost`; the units count against the Hall their Recruit came from (`Object::lineage`, plan
//! decision T56). No rule of the faction's own yet: the Recruits' Construct menu comes with slice M5-J1.

pub(crate) mod structures;
pub(crate) mod units;

#[cfg(test)]
mod tests;
