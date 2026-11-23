# entity-migration-syndicate-units

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-declarative-entity-definitions.md

## Task

Migrate both Syndicate units into per-entity definition files.

**Create 2 files:**
- src/game/entities/syndicate/units/agent.rs
- src/game/entities/syndicate/units/guard.rs

For each, implement EntityDefinition + UnitDefinition traits. Migrate from:
- agent_type_data(), agent_attack_data(), guard_type_data(), guard_attack_data() in unit_data.rs
- Agent-specific constants: AGENT_MINING_DURATION, AGENT_CRYSTAL_CARRY, AGENT_CONTROL_COST, AGENT_TUNNEL_SPACE_COST, etc.
- Guard-specific constants: GUARD_CONTROL_COST, GUARD_TUNNEL_SPACE_COST
- Command panel slots from grid_layout.rs (Agent has multiple interface states: Default, AgentMenu variants)
- Spawn logic from spawn_syndicate_agent(), spawn_syndicate_guard() in utils.rs
- Production costs from HeadquartersState::production_cost()

**Note:** Agent is more complex than most entities — it has gathering, tunnel-building, and carrying capabilities with associated constants. Ensure all Agent-specific constants are consolidated in the entity file.

Register both in build_registry(). Keep old functions as thin wrappers during transition.

**Tests per entity file:**
- Stats match current type_data values
- Attack data matches current attack_data values
- All entity-specific constants preserved with correct values
- Command slots match current grid_layout.rs entries
- Cost matches HeadquartersState::production_cost() values
- Spawn produces same components

**Reference:** Architecture doc Phase 4 (Syndicate Entities)

## Technical Context

### Pilot Implementation to Follow: Peacekeeper
The Peacekeeper at `src/game/entities/factions/gdo/units/peacekeeper.rs` is the **canonical reference** for this task. Follow its structure exactly:
- Marker struct (e.g., `pub struct SyndicateAgent;`)
- Constants section (control cost, rugged bonus, move speed, rotation speed, turn rate)
- Static `COMMAND_SLOTS` array
- `EntityDefinition` impl (OBJECT, object_type, base_stats, cost, command_slots, spawn)
- `UnitDefinition` impl (unit_stats, attack_data, movement_params)
- `#[cfg(test)] mod tests` with per-field assertions

### Files to Create

**`src/game/entities/factions/syndicate/units/agent.rs`**
- Marker: `pub struct SyndicateAgentDef;` (avoid name collision with any existing `SyndicateAgent` if needed)
- Constants to define (migrate from `unit_data.rs` lines 184-208):
  - `pub const AGENT_CONTROL_COST: u32 = 1;`
  - `pub const AGENT_TUNNEL_SPACE_COST: u32 = 2;`
  - `pub const AGENT_RUGGED_BONUS: f32 = 0.5;`
  - `pub const AGENT_MINING_DURATION: u32 = 48;`
  - `pub const AGENT_PICKUP_DURATION: u32 = 48;`
  - `pub const AGENT_DROPOFF_DURATION: u32 = 48;`
  - `pub const AGENT_CRYSTAL_CARRY: u32 = 50;`
  - `pub const AGENT_SUPPLY_CARRY: u32 = 1;`
  - `pub const AGENT_TUNNEL_BUILD_FRAMES: u32 = 480;`
  - Movement: `MOVE_SPEED = 6.0 * FRAMES_PER_SECOND / SPACE_UNITS_PER_GRID_UNIT` (= 1.5 GU/sec)
  - Rotation: `ROTATION_SPEED = 10.0`
  - Turn rate: `TURN_RATE = 180.0_f32.to_radians() * FRAMES_PER_SECOND` (note: Peacekeeper uses `PI * FPS`, same value but different expression — use the `to_radians()` form to match utils.rs spawn code)
- `EntityDefinition::OBJECT = ObjectEnum::SyndicateAgent`
- `base_stats()`: faction=TheSyndicate, max_hp=75, point_armor=1, full_armor=1
- `cost()`: space_crystals=100, build_frames=160 (from `HeadquartersState::production_cost`, structures.rs:274-277)
- `command_slots()`: Agent uses **two interface states** with different command grids:
  - `ObjectInterfaceState::Default`: Move(0,0), HoldPosition(0,2), Attack(1,0), Patrol(1,1), Stop(2,1) — same as standard units, no Reverse (HeavyInfantry can_reverse=false), has_attack=true, can_target_ground check needed (Agent is Melee/Ground, so AttackGround may not apply — check current behavior; Guard has Ranged so likely yes)
  - `ObjectInterfaceState::AgentMenu(AgentDefault)`: AgentBuildTunnel(0,0), AgentDropOff(0,1) (from grid_layout.rs:159-162)
  - `ObjectInterfaceState::AgentMenu(AgentAwaitingPlacement)`: no slots (empty, line 164-167)
- `spawn()`: **Critical differences from Peacekeeper:**
  1. Attack range uses `MELEE_RANGE` constant (from `crate::game::combat::types::MELEE_RANGE`), NOT `attack.range as f32`
  2. Must insert `AgentCarryState::default()` component (from `crate::game::units::types::state::AgentCarryState`)
  3. Must insert `TunnelSpaceCost(AGENT_TUNNEL_SPACE_COST)` component (Guard has this too)
  4. Capsule mesh: `Capsule3d::new(0.28, 0.8)` — larger than Peacekeeper's `(0.2, 0.7)`
  5. Uses `ObjectEnum::SyndicateAgent`
- `unit_stats()`: unit_base=HeavyInfantry, silhouette 36x36, control_cost=1
- `attack_data()`: FullyConnected/Melee, damage=6, range=0 (melee), aim=2, firing=4, cooldown=1, reload=9
- `movement_params()`: TurnRate with max_speed=1.5, accel/decel=MAX, turn_rate=(pi*16 effectively)

**`src/game/entities/factions/syndicate/units/guard.rs`**
- Marker: `pub struct SyndicateGuardDef;`
- Constants:
  - `pub const GUARD_CONTROL_COST: u32 = 1;`
  - `pub const GUARD_TUNNEL_SPACE_COST: u32 = 2;`
  - `pub const GUARD_RUGGED_BONUS: f32 = 0.5;`
  - Movement: `MOVE_SPEED = 5.0 * FRAMES_PER_SECOND / SPACE_UNITS_PER_GRID_UNIT` (= 1.25 GU/sec)
  - Same ROTATION_SPEED and TURN_RATE as Agent
- `EntityDefinition::OBJECT = ObjectEnum::SyndicateGuard`
- `base_stats()`: faction=TheSyndicate, max_hp=80, point_armor=1, full_armor=1
- `cost()`: space_crystals=125, build_frames=120 (structures.rs:278-281)
- `command_slots()`: Guard uses only `ObjectInterfaceState::Default` — standard combat unit layout: Move(0,0), HoldPosition(0,2), Attack(1,0), Patrol(1,1), AttackGround(1,2), Stop(2,1). No Reverse (HeavyInfantry). Guard is Ranged so AttackGround(1,2) applies.
- `spawn()`: Differences from Agent:
  1. Attack range uses `attack_data.range as f32` (= 3.0), NOT MELEE_RANGE
  2. NO `AgentCarryState` component
  3. Still has `TunnelSpaceCost(GUARD_TUNNEL_SPACE_COST)`
  4. Uses `ObjectEnum::SyndicateGuard`
- `unit_stats()`: unit_base=HeavyInfantry, silhouette 36x36, control_cost=1
- `attack_data()`: FullyConnected/Ranged, damage=6, range=3, aim=2, firing=1, cooldown=1, reload=4
- `movement_params()`: TurnRate with max_speed=1.25, accel/decel=MAX

### Files to Modify

**`src/game/entities/factions/syndicate/units/mod.rs`** — Currently just a doc comment. Add:
```rust
pub mod agent;
pub mod guard;
```

**`src/game/entities/registry.rs`** — Add imports and registration:
```rust
use super::factions::syndicate::units::agent::SyndicateAgentDef;
use super::factions::syndicate::units::guard::SyndicateGuardDef;
// In build_registry():
registry.register_unit::<SyndicateAgentDef>();
registry.register_unit::<SyndicateGuardDef>();
```
Update the test `test_build_registry_has_peacekeeper` — registry.len() changes from 1 to 3.

**`src/game/units/types/unit_data.rs`** — Convert Agent/Guard functions to thin wrappers (same pattern as Peacekeeper at lines 124-148):
- `agent_type_data()` delegates to `SyndicateAgentDef::base_stats()` + `SyndicateAgentDef::unit_stats()`
- `agent_attack_data()` delegates to `SyndicateAgentDef::attack_data().expect()`
- `guard_type_data()` / `guard_attack_data()` same pattern with `SyndicateGuardDef`
- Constants become `pub use` re-exports from the entity definition files
- **Keep the existing constant names** (AGENT_CONTROL_COST etc.) as pub use re-exports so all existing consumers continue to compile

### Key Import Paths
- `crate::game::entities::types::{EntityDefinition, UnitDefinition, BaseStats, UnitStats, CommandSlotDef, SpawnParams}`
- `crate::game::units::types::unit_data::{AttackAttributesData, UnitType, UnitControlCost, RuggedTerrainDefenseBonus, TunnelSpaceCost, frames_to_seconds}`
- `crate::game::units::types::movement::{MovementParams, TurnRateMovementParams, MovementSpeed, RotationSpeed, Velocity}`
- `crate::game::units::types::state::behavior::{BaseBehaviorState, LocomotionChannel, OrientationChannel, BaseAttackChannel}`
- `crate::game::units::types::state::commands::{BaseCommandState, CommandQueue}`
- `crate::game::units::types::state::AgentCarryState` (Agent only)
- `crate::game::combat::types::{AttackCapability, AttackState, AttackType, Armor, Silhouette, MELEE_RANGE}` (Agent uses MELEE_RANGE)
- `crate::shared::types::{Unit, Selectable, SelectionBounds, GridPosition, SightRange, VisibilityStateEnum}`
- `crate::game::types::ObjectInstance`
- `crate::game::units::types::UnitCommand`
- `crate::simulation::{FRAMES_PER_SECOND, SPACE_UNITS_PER_GRID_UNIT}`
- `crate::ui::types::{ObjectInterfaceState, CommandButtonAction, AgentMenuState}` (Agent needs AgentMenuState)

### ECS Considerations
- `EntityDefinition::spawn()` takes `&mut EntityCommands` (not `&mut Commands`). The old spawn functions take `&mut Commands` and call `commands.spawn()`. In the new pattern, the caller has already spawned the entity — `spawn()` just `.insert()`s components. See Peacekeeper spawn (peacekeeper.rs:102-174).
- Old spawn functions create Mesh3d + MeshMaterial3d — the new `spawn()` does NOT insert visual components (SpawnParams has no asset handles). Old functions remain as thin wrappers that create visuals + call registry spawn.
- The old functions return `Entity` — they can delegate to the entity commands pattern internally.
- Agent's `AgentCarryState::default()` is a critical component — without it, gathering behavior queries will miss the entity.
- `TunnelSpaceCost` is needed by both Agent and Guard for tunnel entry capacity checks.

## Dependencies

- **entity-registry-grid-layout** (sibling task): If that task changes how `CommandSlotDef` works or adds slot conditions, the command_slots definitions here may need adjustment. However, both can proceed independently since they use the same `CommandSlotDef` struct.
- **entity-definition-cleanup** (sibling task): That task removes old scattered code after all migrations are done. This task must land first — it creates the new files that cleanup will reference.
- **Peacekeeper pilot** (already complete): The Peacekeeper definition at `entities/factions/gdo/units/peacekeeper.rs` is the pattern to follow. It's already registered in `build_registry()`.
