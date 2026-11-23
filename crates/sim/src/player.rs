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

impl Faction {
    /// Every faction, in declaration order, so tools such as the map editor's faction picker list new ones
    /// automatically. A test checks that it stays complete.
    pub const ALL: &'static [Faction] = &[Faction::Gdo, Faction::Syndicate, Faction::Cults, Faction::Colonists];
}

/// Per-player simulation state. Faction resources (Space Crystals, Power, …) are added
/// here as the economy milestones land.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub faction: Faction,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde::de::IntoDeserializer;
    use serde::de::value::{Error as ValueError, U32Deserializer};

    #[test]
    fn all_lists_every_faction_in_declaration_order() {
        // Serde's derived `Deserialize` accepts a unit variant by its declaration index.
        let declared: Vec<Faction> = (0u32..)
            .map_while(|i| {
                let index: U32Deserializer<ValueError> = i.into_deserializer();
                Faction::deserialize(index).ok()
            })
            .collect();
        assert_eq!(Faction::ALL, declared.as_slice());
    }
}
