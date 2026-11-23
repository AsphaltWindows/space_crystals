//! Factions and players — `core/entities.md > Faction`, `> Player`.

use serde::{Deserialize, Serialize};

use crate::ids::PlayerId;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Faction {
    /// `factions/gdo/overview.md`
    Gdo,
    /// `factions/syndicate/overview.md`
    Syndicate,
    /// `factions/cults/overview.md`
    Cults,
    /// `factions/colonists/overview.md`
    Colonists,
}

/// Per-player simulation state. Faction resources (Space Crystals, Power, …) are added
/// here as the economy milestones land.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub faction: Faction,
}
