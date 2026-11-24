//! Factions and players — `core/entities.md > Faction`, `> Player`.

use serde::{Deserialize, Serialize};

use crate::ids::PlayerId;
use crate::units::Tick;

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

/// Per-player simulation state: who the player is, their banks (`factions/gdo/overview.md > GDOResources`) and
/// whether they are defeated. Power and Unit Control are not stored: they follow from the player's objects
/// (`crate::economy`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub faction: Faction,
    /// `> Space Crystals`: the bank. A game starts it at 0 until the Starting Conditions set it (slice M2-F;
    /// `factions/gdo/economy.md > Starting Conditions`); `DebugCommand::SetResources` sets it. Optional in files, so
    /// that states saved before it existed still load.
    #[serde(default)]
    pub crystals: u32,
    /// `> Supplies`: a counter only. The GDO supply loop is deferred, Starting Supplies are 0 and nothing costs
    /// Supplies yet (design change 0077), so only `DebugCommand::SetResources` changes it.
    #[serde(default)]
    pub supplies: u32,
    /// The tick in which the player was defeated (`core/game_rules.md > Defeat Condition`, plan decision T8:
    /// their owned-structure count dropped to 0, `economy::defeat`); `None` while they are in the game. Final: a
    /// defeated player stays defeated (`> On Defeat`). Optional in files, so that states saved before it existed
    /// still load.
    #[serde(default)]
    pub defeated_at: Option<Tick>,
}

impl Player {
    /// Whether the player has been defeated ([`Player::defeated_at`]): the gate refuses all their commands.
    pub fn is_defeated(&self) -> bool {
        self.defeated_at.is_some()
    }

    /// Pays `amount` Space Crystals into the bank (mining, refunds). A bank already near the top saturates instead
    /// of overflowing: only a debug command fills one that far (`ARCHITECTURE.md > Commands, events and replays`:
    /// numbers that come from input never panic the sim). So, unlike plan decision T7's sketch, no `debug_assert`
    /// guards the saturation: `DebugCommand::SetResources` may set a bank to `u32::MAX`, and the plates then pay
    /// into it.
    pub fn credit_crystals(&mut self, amount: u32) {
        self.crystals = self.crystals.saturating_add(amount);
    }

    /// Takes `cost` Space Crystals from the bank if it holds that many, and says whether it did; a bank that is
    /// short is left as it was (plan decision T7: a spend is refused, never taken below 0).
    #[must_use]
    pub fn spend_crystals(&mut self, cost: u32) -> bool {
        match self.crystals.checked_sub(cost) {
            Some(left) => {
                self.crystals = left;
                true
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde::de::IntoDeserializer;
    use serde::de::value::{Error as ValueError, U32Deserializer};

    fn player(crystals: u32) -> Player {
        Player {
            id: PlayerId(1),
            name: "P".into(),
            faction: Faction::Gdo,
            crystals,
            supplies: 0,
            defeated_at: None,
        }
    }

    /// Plan decision T7: a spend the bank can't cover is refused and changes nothing; income saturates.
    #[test]
    fn spends_are_checked_and_income_saturates() {
        let mut p = player(100);
        assert!(p.spend_crystals(75));
        assert_eq!(p.crystals, 25);
        assert!(!p.spend_crystals(26), "short by one");
        assert_eq!(p.crystals, 25);
        assert!(p.spend_crystals(25));
        assert_eq!(p.crystals, 0);
        assert!(p.spend_crystals(0));
        p.credit_crystals(10);
        assert_eq!(p.crystals, 10);
        let mut full = player(u32::MAX - 3);
        full.credit_crystals(10);
        assert_eq!(full.crystals, u32::MAX);
    }

    /// States saved before the banks and the defeat existed load with empty banks, the player in the game.
    #[test]
    fn banks_are_optional_in_files() {
        let json = r#"{"id": 2, "name": "Old", "faction": "Gdo"}"#;
        let p: Player = serde_json::from_str(json).unwrap();
        assert_eq!((p.id, p.crystals, p.supplies, p.defeated_at), (PlayerId(2), 0, 0, None));
    }

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
