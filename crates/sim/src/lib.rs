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

pub mod aircraft;
pub mod augmentation;
pub mod buffs;
pub mod combat;
pub mod command;
pub mod construction;
pub mod conversion;
pub mod cooldown;
pub mod defs;
pub mod deploy;
pub mod economy;
pub mod elevation;
pub mod energy;
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
pub mod recruitment;
pub(crate) mod regeneration;
pub mod repair;
pub mod replay;
pub mod riot_shield;
pub mod rng;
pub mod ruleset;
pub mod scenario;
pub mod scenarios;
pub mod shape;
pub mod shields;
pub mod shroud;
pub mod sides;
pub mod sim;
pub mod spot_work;
pub mod stock;
pub mod suppress;
pub mod symmetry;
pub mod targets;
pub mod tech;
pub mod testing;
pub mod tether;
pub mod toggle;
pub mod transform;
pub mod tuning;
pub mod tunnel;
pub mod units;
pub mod vehicle_deploy;
pub mod vision;

pub use augmentation::{AugmentOrder, Augmentation, Increments, Level, Levels, RecruitKind, Research};
pub use combat::{AttackPhase, AttackState};
pub use command::{Ability, Command, DebugCommand, HoldCommand, PlayerCommand, TickInputs, TunnelCommand};
pub use construction::{ConstructionCommand, Constructor};
pub use defs::{
    AbilityArea, AreaAbility, Armor, Category, ExitRule, FactionRules, Footprint, MiningDef, ObjectDef, ObjectKind,
    ProductionDef, Recipe, RecruitmentDef, ResearchDef, ResearchLine, StructureDef, SymmetryType, Tech, TechDef,
    TierDef, Transit, TunnelDef, UpgradeDef,
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
    Door, Doorway, EnergyDoorway, ExitDoor, Production, ProductionCommand, ProductionStatus, QueueEntry, Rally, Refusal,
};
pub use progress::Progress;
pub use recruitment::{Capacity, Claims, HallState};
pub use replay::{Checkpoint, Desync, Replay, ReplayError};
pub use riot_shield::{RiotShield, ShieldMode};
pub use rng::Rng;
pub use ruleset::{Ruleset, rules};
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
pub use units::{CornerPos, SU_PER_GU, TICKS_PER_SECOND, Tick, TilePos, gu, seconds};
pub use vision::{VisibilityState, Vision};
