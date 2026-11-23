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
