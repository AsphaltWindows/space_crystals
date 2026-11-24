//! Everything needed to start a game: a map ([`MapDef`]) and who plays it ([`GameSetup`]).
//!
//! # Maps and game setups
//!
//! A map (`core/map_editor.md`) holds only the map: its type ([`MapType`]), name and description, the terrain (a
//! [`Map`]), its symmetry, its player slots ([`Slot`]: a start position, an optional camera start, and one Starting
//! Layout per faction) and its pre-placed objects ([`Spawn`]s, neutral or owned by a slot). Who plays is not part of
//! it: a [`GameSetup`] lists the players, each with a name, a faction and the slot it takes, and holds the RNG seed.
//! A [`Scenario`] is the pair, everything [`crate::Sim::try_new`] needs: "a map and its players", as it was before
//! the two were split, so replays, the client and tests still start games from a `Scenario`.
//!
//! # Starting Conditions and Starting Layouts
//!
//! A game starts each player with their faction's Starting Conditions ([`StartingConditions`]: the banks, and the
//! objects their faction's Starting Layout at their slot places). Validation holds every layout to its faction's
//! placement rules, and on a Skirmish Default map to its Starting Conditions, complete; "outdated" layouts are
//! derived (see `starting.rs`).
//!
//! # Map files
//!
//! A [`MapDef`] is the map editor's file format. It serializes as an object whose first field is `version`
//! ([`MAP_FORMAT_VERSION`]), followed by `map_type`, `name`, `description`, `symmetry`, `terrain` (the compact row
//! form in the [`crate::map`] docs: custom presets, tile rows and corner height rows), `slots` (each with its `start`,
//! `camera_start` and per-faction `layouts`) and `spawns` (each with its per-object `settings`, [`SpawnSettings`]).
//! Written with `serde_json::to_string_pretty`, a file shows the map as one line per row, so it stays short,
//! diffable and editable by hand. Reading rejects other versions and unknown fields, and checks the format only: run
//! [`MapDef::validate`] or [`Scenario::validate`] (or [`crate::Sim::try_new`]) before using a loaded map.

mod create;
mod edit;
mod generate;
mod mapdef;
mod problems;
mod settings;
mod setup;
mod starting;
mod validate;

use serde::{Deserialize, Serialize};

pub use create::{CreationProblem, MAX_SLOTS, MIN_MAP_SIDE, MapSettings};
pub use edit::ResizeReport;
pub use generate::{Asymmetry, image_location};
pub use mapdef::{
    BuildingOrientation, LayoutObject, MAP_FORMAT_VERSION, MapDef, MapType, Slot, Spawn, SpawnAt, StartingLayouts,
};
pub use problems::{ScenarioProblem, SpawnProblem};
pub use settings::{
    DEFAULT_DELIVERY_INTERVAL, DEFAULT_DELIVERY_SIZE, DEFAULT_STARTING_AMOUNT, DEFAULT_STARTING_SUPPLIES,
    SettingsProblem, SpawnSettings,
};
pub use setup::{DEFAULT_SEED, GameSetup, PlayerSetup};
pub use starting::StartingConditions;
pub(crate) use validate::{locate_oriented, place_spawn};

use crate::map::Map;

/// A map and who plays it (see the module docs). Serializes as `{ "map": …, "setup": … }`, the map in its versioned
/// file format.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    pub map: MapDef,
    pub setup: GameSetup,
}

impl Scenario {
    /// `map` with its default setup ([`GameSetup::default_for`]): a GDO player in every slot.
    pub fn new(map: MapDef) -> Self {
        let setup = GameSetup::default_for(&map);
        Self { map, setup }
    }

    /// The map's name.
    pub fn name(&self) -> &str {
        &self.map.name
    }

    /// The terrain.
    pub fn terrain(&self) -> &Map {
        &self.map.terrain
    }
}
