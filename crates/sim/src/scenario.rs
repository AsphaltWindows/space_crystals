//! Everything needed to start a game: map, players, initial objects, RNG seed.

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::defs::ObjectKind;
use crate::ids::PlayerId;
use crate::map::Map;
use crate::player::Faction;
use crate::units::TilePos;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Scenario {
    pub name: String,
    pub map: Map,
    pub players: Vec<PlayerSetup>,
    /// Spawned in order at tick 0, so they get ascending `ObjectId`s in this order.
    pub spawns: Vec<Spawn>,
    pub seed: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlayerSetup {
    pub id: PlayerId,
    pub name: String,
    pub faction: Faction,
    /// `core/camera.md > Starting Position`: map-defined starting camera center, if any.
    pub camera_start: Option<Vec2>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Spawn {
    pub kind: ObjectKind,
    pub owner: Option<PlayerId>,
    pub at: SpawnAt,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SpawnAt {
    /// Tile-based objects: top-left tile of the (unrotated) footprint.
    Tile(TilePos),
    /// Units: center position in space units.
    Point(Vec2),
}
