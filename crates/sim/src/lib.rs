//! # sc_sim — Space Crystals game simulation
//!
//! Deterministic, engine-free game logic (no Bevy). The client (`crates/game`) reads
//! this state to draw it and feeds player commands in through [`TickInputs`].
//! Read `ARCHITECTURE.md` at the repository root before changing anything here.
//!
//! Hard rules for this crate:
//! - Same scenario + same inputs ⇒ bit-identical state. No wall-clock time, no
//!   thread-dependent behavior, no `HashMap`/`HashSet` iteration, no platform
//!   transcendental float functions (`f32::sin` etc. — use [`math`]).
//! - Fixed 16 ticks per second; all durations are counted in ticks.
//! - Every gameplay number cites its design section (`path/file.md > Section`).

mod combat;
pub mod command;
pub mod defs;
pub mod events;
pub(crate) mod hash;
pub mod ids;
pub mod map;
pub mod math;
mod movement;
pub mod mover;
pub mod nav;
pub mod object;
pub mod orders;
mod paths;
pub mod player;
pub mod replay;
pub mod rng;
pub mod scenario;
pub mod scenarios;
pub mod sim;
pub mod symmetry;
pub mod testing;
pub mod units;
mod vision;

pub use command::{Command, DebugCommand, PlayerCommand, TickInputs};
pub use defs::{Category, Footprint, ObjectDef, ObjectKind};
pub use events::SimEvent;
pub use glam::Vec2;
pub use ids::{ObjectId, PlayerId};
pub use map::{MAX_ELEVATION, MAX_TRAVERSIBLE_SLOPE, Map, Tile, TilePreset, TilePresetId};
pub use mover::{MoveGoal, MoveStatus, Mover};
pub use object::{Object, Placement, ResourceState, Rotation};
pub use orders::{OrderKind, Orders, Target};
pub use player::{Faction, Player};
pub use replay::Replay;
pub use rng::Rng;
pub use scenario::{
    PlayerSetup, Scenario, ScenarioProblem, SettingsProblem, Spawn, SpawnAt, SpawnProblem, SpawnSettings,
};
pub use sim::{Reject, Sim, SimConfig};
pub use symmetry::{MapTransform, Orientation, Symmetry};
pub use units::{CornerPos, SU_PER_GU, TICKS_PER_SECOND, Tick, TilePos, gu, seconds};
