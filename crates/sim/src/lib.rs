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
pub mod conversion;
pub mod defs;
pub mod deploy;
pub mod economy;
pub mod elevation;
pub mod events;
pub(crate) mod hash;
pub mod holding;
pub mod ids;
pub mod map;
pub mod math;
mod movement;
pub mod mover;
pub mod nav;
pub mod network;
pub mod object;
pub mod orders;
mod paths;
pub mod placement;
pub mod player;
pub mod popup;
pub mod production;
pub mod progress;
pub mod replay;
pub mod riot_shield;
pub mod rng;
pub mod scenario;
pub mod scenarios;
pub mod shape;
pub mod shields;
pub mod sides;
pub mod sim;
pub mod symmetry;
pub mod targets;
pub mod tech;
pub mod testing;
pub mod toggle;
pub mod transform;
pub mod tuning;
pub mod tunnel;
pub mod units;
pub mod vision;

pub use combat::{AttackPhase, AttackState};
pub use command::{Ability, Command, DebugCommand, HoldCommand, PlayerCommand, TickInputs, TunnelCommand};
pub use construction::{ConstructionCommand, Constructor};
pub use defs::{
    Armor, BEACON_CAPACITY_CAP, BEACON_CAPACITY_PER_BUILDING, Category, ExitRule, Footprint, MiningDef, ObjectDef,
    ObjectKind, ProductionDef, Recipe, StructureDef, SymmetryType, TUNNEL_SPACE_CAP, TierDef, Transit, TunnelDef,
    UNIT_CONTROL_CAP, UpgradeDef,
};
pub use economy::{ArmyCap, ArmyResource, Extractor, Outcome, PowerSummary};
pub use events::SimEvent;
pub use glam::Vec2;
pub use ids::{ObjectId, PlayerId, SlotId};
pub use map::{MAX_ELEVATION, MAX_TRAVERSIBLE_SLOPE, Map, Tile, TileLook, TilePreset, TilePresetId};
pub use movement::MoveCounters;
pub use mover::{MoveGoal, MoveStatus, Mover};
pub use object::{Object, Placement, ResourceState, Rotation};
pub use orders::build::PendingBuild;
pub use orders::{OrderKind, Orders, Target};
pub use placement::{BuildArea, PlacementCheck, PlacementProblem, TileProblem, WholeProblem};
pub use player::{Banks, Faction, Player};
pub use production::{
    Door, Doorway, EnergyDoorway, ExitDoor, Production, ProductionCommand, ProductionStatus, Rally, Refusal,
};
pub use progress::Progress;
pub use replay::{Checkpoint, Desync, Replay, ReplayError};
pub use riot_shield::{RiotShield, ShieldMode};
pub use rng::Rng;
pub use scenario::{
    BuildingOrientation, GameSetup, LayoutObject, MapDef, MapSettings, MapType, PlayerSetup, Scenario, ScenarioProblem,
    SettingsProblem, Slot, Spawn, SpawnAt, SpawnProblem, SpawnSettings, StartingConditions, StartingLayouts,
};
pub use shape::Shape;
pub use sides::{SideLetter, sides};
pub use sim::{Phase, Reject, Sim, SimConfig};
pub use symmetry::{EditableRegion, MapTransform, Orientation, Symmetry};
pub use targets::TargetView;
pub use transform::Transform;
pub use tuning::production::QUEUE_LIMIT;
pub use units::{CornerPos, SU_PER_GU, TICKS_PER_SECOND, Tick, TilePos, gu, seconds};
pub use vision::{VisibilityState, Vision};
