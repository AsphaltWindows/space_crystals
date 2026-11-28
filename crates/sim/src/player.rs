//! Factions and players — `core/entities.md > Faction`, `> Player`.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::defs::Tech;
use crate::ids::PlayerId;
use crate::scenario::StartingConditions;
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

/// Per-player simulation state: who the player is, their banks (`factions/gdo/overview.md > GDOResources`,
/// `factions/colonists/overview.md > ColonistResources`) and whether they are defeated. Power and the army caps are
/// not stored: they follow from the player's objects (`crate::economy`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub faction: Faction,
    /// `factions/gdo/overview.md > Space Crystals` (every faction's): the bank. A game starts it at the player's
    /// faction's Starting Conditions (slice M2-F1; `factions/gdo/economy.md > Starting Conditions`: 100,
    /// `crate::StartingConditions`); `DebugCommand::SetResources` sets it. Optional in files, so that states saved
    /// before it existed still load.
    #[serde(default)]
    pub crystals: u32,
    /// `factions/gdo/overview.md > Supplies`, `factions/syndicate/overview.md > Supplies`: the bank. A game starts it
    /// at the player's faction's Starting Conditions (`crate::StartingConditions`: the Syndicate's 1 Supply,
    /// `factions/syndicate/tunnels.md > Tunnel > Starting Conditions`, slice M4-B; 0 for the GDO, whose supply loop is
    /// deferred, design change 0077, and for the Colonists); each Supplies delivery of an Agent adds to it
    /// (`economy::gathering`, slice M4-I), an Agent's Build order pays its price from it and a refund goes back to it
    /// (`factions/syndicate/tunnels.md > Cost Scaling > Construction Cost`, slice M4-E), and
    /// `DebugCommand::SetResources` sets it. Optional in files, so that states saved before it existed still load.
    #[serde(default)]
    pub supplies: u32,
    /// The Colonists' refined resources (`factions/colonists/overview.md > ColonistResources > Alloys`,
    /// `factions/colonists/overview.md > Essence`, `factions/colonists/overview.md > Flux`; plan decision T21):
    /// counters only, at 0 from the start (`factions/colonists/economy.md > Starting Conditions`: "Starting Alloys,
    /// Essence and Flux: 0"), until refining exists (question Q0043 point 1); only `DebugCommand::SetResources` changes
    /// them. Every player has them, as every player has Supplies. Optional in files, so that states saved before they
    /// existed still load.
    #[serde(default)]
    pub alloys: u32,
    #[serde(default)]
    pub essence: u32,
    #[serde(default)]
    pub flux: u32,
    /// The tick in which the player was defeated (`core/game_rules.md > Defeat Condition`, plan decision T8:
    /// their owned-structure count dropped to 0, `economy::defeat`); `None` while they are in the game. Final: a
    /// defeated player stays defeated (`core/game_rules.md > On Defeat`). Optional in files, so that states saved
    /// before it existed still load.
    #[serde(default)]
    pub defeated_at: Option<Tick>,
    /// (M5-N3) The technologies the player has completed (`core/tech_and_research.md > Research > Shared Rules` 1: "a
    /// technology is researched once [...] and on completion applies to all of that player's eligible objects"; plan
    /// decision T74): the Temple's (`crate::defs::Tech`), whose levels the amounts of the Augmentations follow
    /// (`augmentation::Levels::of`). Empty as a game starts; a research that completes adds its technology
    /// (`Sim::complete_research`) and nothing takes one away (Shared Rules 8: "completed research is permanent.
    /// Losing the structure afterward does not revoke it"). Optional in files and left out of them while empty, so that
    /// states saved before it existed still load, and hashed only while not empty (`crate::hash`).
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub techs: BTreeSet<Tech>,
}

/// A player's five banks at once: what `DebugCommand::SetResources` sets ([`Player::set_banks`]).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Banks {
    pub crystals: u32,
    pub supplies: u32,
    pub alloys: u32,
    pub essence: u32,
    pub flux: u32,
}

impl Player {
    /// A player of `faction` as a game starts them: in the game, their Space Crystals and Supplies at the faction's
    /// Starting Conditions (`factions/gdo/economy.md > Starting Conditions`, `factions/colonists/economy.md >
    /// Starting Conditions`: 100 Space Crystals each; `factions/syndicate/tunnels.md > Tunnel > Starting Conditions`:
    /// 100 and 1 Supply, slice M4-B; `factions/cults/economy.md > Starting Conditions`: 0, slice M5-B), and the
    /// refined resources at 0 (T21).
    pub fn starting(id: PlayerId, name: String, faction: Faction) -> Self {
        let start = StartingConditions::of(faction);
        Self {
            id,
            name,
            faction,
            crystals: start.crystals,
            supplies: start.supplies,
            alloys: 0,
            essence: 0,
            flux: 0,
            defeated_at: None,
            techs: BTreeSet::new(),
        }
    }

    /// The player's banks.
    pub fn banks(&self) -> Banks {
        Banks {
            crystals: self.crystals,
            supplies: self.supplies,
            alloys: self.alloys,
            essence: self.essence,
            flux: self.flux,
        }
    }

    /// Sets all of the player's banks (`DebugCommand::SetResources`).
    pub fn set_banks(&mut self, banks: Banks) {
        let Banks {
            crystals,
            supplies,
            alloys,
            essence,
            flux,
        } = banks;
        (self.crystals, self.supplies) = (crystals, supplies);
        (self.alloys, self.essence, self.flux) = (alloys, essence, flux);
    }

    /// (M5-N3) Whether the player has completed `tech` ([`Player::techs`]).
    pub fn has(&self, tech: Tech) -> bool {
        self.techs.contains(&tech)
    }

    /// Whether the player has been defeated ([`Player::defeated_at`]): the gate refuses all their commands.
    pub fn is_defeated(&self) -> bool {
        self.defeated_at.is_some()
    }

    /// Pays `amount` Space Crystals into the bank (mining, refunds). A bank already near the top saturates instead
    /// of overflowing: only a debug command fills one that far (`docs/architecture/commands-events-replays.md > Commands, events and replays`:
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

    /// Pays `amount` Supplies into the bank (a refund), saturating as [`Player::credit_crystals`] does.
    pub fn credit_supplies(&mut self, amount: u32) {
        self.supplies = self.supplies.saturating_add(amount);
    }

    /// Takes `cost` Supplies from the bank if it holds that many, and says whether it did, as
    /// [`Player::spend_crystals`] does.
    #[must_use]
    pub fn spend_supplies(&mut self, cost: u32) -> bool {
        match self.supplies.checked_sub(cost) {
            Some(left) => {
                self.supplies = left;
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
            crystals,
            ..Player::starting(PlayerId(1), "P".into(), Faction::Gdo)
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
        assert_eq!(p.banks(), Banks::default());
    }

    /// A game starts each player at their faction's Starting Conditions, the refined resources at 0 (plan decision
    /// T21): the Syndicate with 100 Space Crystals and 1 Supply (slice M4-B), and the GDO alike
    /// (`factions/gdo/economy.md > Starting Conditions`); the Cults
    /// with nothing, "**0 Space Crystals**" (`factions/cults/economy.md > Starting Conditions`, slice M5-B).
    /// `set_banks` sets all five.
    #[test]
    fn players_start_with_their_factions_banks() {
        let start = |faction| Player::starting(PlayerId(1), "P".into(), faction).banks();
        let hundred = Banks {
            crystals: 100,
            ..Banks::default()
        };
        assert_eq!(start(Faction::Gdo), Banks { supplies: 1, ..hundred });
        assert_eq!(start(Faction::Colonists), hundred);
        assert_eq!(start(Faction::Syndicate), Banks { supplies: 1, ..hundred });
        assert_eq!(start(Faction::Cults), Banks::default());
        let mut p = player(5);
        let set = Banks {
            crystals: 1,
            supplies: 2,
            alloys: 3,
            essence: 4,
            flux: 5,
        };
        p.set_banks(set);
        assert_eq!(p.banks(), set);
        assert_eq!((p.alloys, p.essence, p.flux), (3, 4, 5));
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
