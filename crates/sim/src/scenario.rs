//! Everything needed to start a game: map, players, initial objects, RNG seed.
//!
//! # Scenario files
//!
//! A `Scenario` is also the map editor's file format. It serializes as an object whose
//! first field is `version` ([`SCENARIO_FORMAT_VERSION`]), followed by `name`, `map` (the
//! compact row form in the [`crate::map`] docs), `players`, `spawns` and `seed`. Written with
//! `serde_json::to_string_pretty`, a file shows the map as one line per row, so it stays
//! short, diffable and editable by hand. Reading rejects other versions and unknown fields.
//! Reading checks the format only: run [`Scenario::validate`] (or [`crate::Sim::try_new`])
//! before using a loaded scenario.

mod edit;
mod validate;

use glam::Vec2;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::defs::ObjectKind;
use crate::ids::PlayerId;
use crate::map::Map;
use crate::player::Faction;
use crate::units::TilePos;

pub use edit::ResizeReport;
pub use validate::{ScenarioProblem, SpawnProblem};
pub(crate) use validate::{locate, place_spawn};

/// Version of the scenario file format. Bump it on any incompatible change.
pub const SCENARIO_FORMAT_VERSION: u32 = 1;

/// Serializes as a versioned scenario file (see the module docs).
#[derive(Clone, Debug, PartialEq)]
pub struct Scenario {
    pub name: String,
    pub map: Map,
    pub players: Vec<PlayerSetup>,
    /// Spawned in order at tick 0, so they get ascending `ObjectId`s in this order.
    pub spawns: Vec<Spawn>,
    pub seed: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlayerSetup {
    pub id: PlayerId,
    pub name: String,
    pub faction: Faction,
    /// `core/camera.md > Starting Position`: map-defined starting camera center, if any.
    pub camera_start: Option<Vec2>,
    /// The tile where this slot's starting base will go. Placeholder map model until
    /// design question Q0004 (player slots, start locations and starting-base layout)
    /// is answered; nothing is spawned there yet.
    pub start_location: Option<TilePos>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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

/// The serialized form of [`Scenario`], borrowed for writing.
#[derive(Serialize)]
struct ScenarioFileRef<'a> {
    version: u32,
    name: &'a str,
    map: &'a Map,
    players: &'a [PlayerSetup],
    spawns: &'a [Spawn],
    seed: u64,
}

/// The serialized form of [`Scenario`], owned for reading.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ScenarioFile {
    version: u32,
    name: String,
    map: Map,
    players: Vec<PlayerSetup>,
    spawns: Vec<Spawn>,
    seed: u64,
}

impl Serialize for Scenario {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let Scenario {
            name,
            map,
            players,
            spawns,
            seed,
        } = self;
        ScenarioFileRef {
            version: SCENARIO_FORMAT_VERSION,
            name,
            map,
            players,
            spawns,
            seed: *seed,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Scenario {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let ScenarioFile {
            version,
            name,
            map,
            players,
            spawns,
            seed,
        } = ScenarioFile::deserialize(deserializer)?;
        if version != SCENARIO_FORMAT_VERSION {
            return Err(D::Error::custom(format!(
                "unsupported scenario format version {version} (this build reads version {SCENARIO_FORMAT_VERSION})"
            )));
        }
        Ok(Scenario {
            name,
            map,
            players,
            spawns,
            seed,
        })
    }
}
