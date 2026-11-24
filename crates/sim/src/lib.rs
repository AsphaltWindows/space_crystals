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

pub mod combat;
pub mod command;
pub mod construction;
pub mod defs;
pub mod economy;
pub mod elevation;
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
pub mod placement;
pub mod player;
pub mod production;
pub mod replay;
pub mod riot_shield;
pub mod rng;
pub mod scenario;
pub mod scenarios;
pub mod shape;
pub mod sides;
pub mod sim;
pub mod symmetry;
pub mod testing;
pub mod transform;
pub mod units;
pub mod vision;

pub use combat::{AttackPhase, AttackState};
pub use command::{Ability, Command, DebugCommand, PlayerCommand, TickInputs};
pub use construction::{ConstructionCommand, Constructor};
pub use defs::{
    Armor, Category, Footprint, MiningDef, ObjectDef, ObjectKind, ProductionDef, Recipe, StructureDef, SymmetryType,
};
pub use economy::{
    ArmyCap, ArmyResource, BEACON_CAPACITY_CAP, BEACON_CAPACITY_PER_BUILDING, Extractor, Outcome, PowerSummary,
    UNIT_CONTROL_CAP,
};
pub use events::SimEvent;
pub use glam::Vec2;
pub use ids::{ObjectId, PlayerId, SlotId};
pub use map::{MAX_ELEVATION, MAX_TRAVERSIBLE_SLOPE, Map, Tile, TileLook, TilePreset, TilePresetId};
pub use movement::MoveCounters;
pub use mover::{MoveGoal, MoveStatus, Mover};
pub use object::{Object, Placement, ResourceState, Rotation};
pub use orders::{OrderKind, Orders, Target};
pub use placement::{BuildArea, PlacementCheck, PlacementProblem, TileProblem, WholeProblem};
pub use player::{Banks, Faction, Player};
pub use production::{Doorway, Production, ProductionCommand, ProductionStatus, QUEUE_LIMIT, Rally, Refusal};
pub use replay::{Checkpoint, Desync, Replay, ReplayError};
pub use riot_shield::RiotShield;
pub use rng::Rng;
pub use scenario::{
    BuildingOrientation, GameSetup, LayoutObject, MapDef, MapSettings, MapType, PlayerSetup, Scenario, ScenarioProblem,
    SettingsProblem, Slot, Spawn, SpawnAt, SpawnProblem, SpawnSettings, StartingConditions, StartingLayouts,
};
pub use shape::Shape;
pub use sides::{SideLetter, sides};
pub use sim::{Phase, Reject, Sim, SimConfig};
pub use symmetry::{EditableRegion, MapTransform, Orientation, Symmetry};
pub use transform::Transform;
pub use units::{CornerPos, SU_PER_GU, TICKS_PER_SECOND, Tick, TilePos, gu, seconds};
pub use vision::{VisibilityState, Vision};
