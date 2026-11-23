//! Who plays a map: the game setup. Factions are picked per slot when a game is set up, never by the map
//! (`core/map_editor.md > Map Types > Skirmish Default`: any faction can play any slot).

use serde::{Deserialize, Serialize};

use super::MapDef;
use crate::ids::{PlayerId, SlotId};
use crate::player::Faction;
use crate::sim::SimConfig;

/// The RNG seed of a default setup.
pub const DEFAULT_SEED: u64 = 1;

/// The players of a game, its RNG seed and its game options.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameSetup {
    /// `core/entities.md > Player`: player `n` is entry `n - 1`, so players are numbered `1..=n`.
    pub players: Vec<PlayerSetup>,
    pub seed: u64,
    /// The game options, such as whether debug commands are accepted.
    pub config: SimConfig,
}

/// One player of a game.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlayerSetup {
    pub name: String,
    pub faction: Faction,
    /// The slot the player takes: its start position, camera start and Starting Layout, and the objects the map
    /// gives that slot.
    pub slot: SlotId,
}

impl GameSetup {
    /// A player in every slot of `map`, in slot order, each with the faction `factions` gives its slot (GDO where it
    /// gives none) and named "Player n". Player `n` takes slot `n`. The default game options: no debug commands.
    pub fn per_slot(map: &MapDef, factions: &[Faction], seed: u64) -> Self {
        let players = map
            .slot_ids()
            .enumerate()
            .map(|(i, slot)| PlayerSetup {
                name: format!("Player {}", i + 1),
                faction: factions.get(i).copied().unwrap_or(Faction::Gdo),
                slot,
            })
            .collect();
        Self {
            players,
            seed,
            config: SimConfig::default(),
        }
    }

    /// The default setup: a GDO player in every slot, player `n` in slot `n`, [`DEFAULT_SEED`] and no debug
    /// commands.
    pub fn default_for(map: &MapDef) -> Self {
        Self::per_slot(map, &[], DEFAULT_SEED)
    }

    /// The ids of the players, in order.
    pub fn player_ids(&self) -> impl Iterator<Item = PlayerId> + '_ {
        (0..self.players.len()).filter_map(PlayerId::from_index)
    }

    pub fn player(&self, id: PlayerId) -> Option<&PlayerSetup> {
        self.players.get(id.index()?)
    }

    /// The first player who takes `slot`.
    pub fn player_at(&self, slot: SlotId) -> Option<PlayerId> {
        let index = self.players.iter().position(|p| p.slot == slot)?;
        PlayerId::from_index(index)
    }
}
