//! Stable identifiers.

use serde::{Deserialize, Serialize};

/// Identity of an object instance (unit, structure, resource…). Allocated in
/// increasing order and never reused within a game, so ascending `ObjectId`
/// order is the canonical deterministic iteration order.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ObjectId(pub u32);

/// A player's number, `1..=n` (`core/entities.md > Player > PlayerNumber`).
/// Neutral objects have no owner (`Option<PlayerId>::None`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PlayerId(pub u8);

impl PlayerId {
    /// The player at `index` (0-based) of a game setup's player list, if it can be numbered.
    pub fn from_index(index: usize) -> Option<PlayerId> {
        u8::try_from(index.checked_add(1)?).ok().map(PlayerId)
    }

    /// Its index (0-based) in the game setup's player list; `None` for the invalid number 0.
    pub fn index(self) -> Option<usize> {
        usize::from(self.0).checked_sub(1)
    }
}

/// A player slot of a map (`core/map_editor.md > Map Creation`: one per start position), `1..=n` in the map's slot
/// order. Which player takes it is part of the game setup, not of the map.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SlotId(pub u8);

impl SlotId {
    /// The slot at `index` (0-based) of a map's slot list, if it can be numbered.
    pub fn from_index(index: usize) -> Option<SlotId> {
        u8::try_from(index.checked_add(1)?).ok().map(SlotId)
    }

    /// Its index (0-based) in the map's slot list; `None` for the invalid number 0.
    pub fn index(self) -> Option<usize> {
        usize::from(self.0).checked_sub(1)
    }
}
