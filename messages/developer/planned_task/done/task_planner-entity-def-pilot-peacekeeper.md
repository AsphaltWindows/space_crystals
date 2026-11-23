# entity-def-pilot-peacekeeper

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-declarative-entity-definitions.md

## Task

Migrate Peacekeeper as the pilot entity to validate the EntityDefinition/UnitDefinition trait pattern end-to-end.

**Create:** src/game/entities/gdo/units/peacekeeper.rs

This file should contain:
- `pub struct Peacekeeper;`
- `impl EntityDefinition for Peacekeeper` — OBJECT, object_type(), base_stats(), cost(), command_slots(), spawn()
- `impl UnitDefinition for Peacekeeper` — unit_stats(), attack_data(), movement_params()
- All values migrated from: `peacekeeper_type_data()` in unit_data.rs, `peacekeeper_attack_data()` in unit_data.rs, `PEACEKEEPER_CONTROL_COST` constant, command panel slots from grid_layout.rs, production cost from BarracksState::production_cost(), spawn logic from `spawn_peacekeeper()` in utils.rs
- Entity-specific constants: PEACEKEEPER_CONTROL_COST, PEACEKEEPER_RUGGED_BONUS
- Per-entity tests in `#[cfg(test)]` module verifying stats, attack data, command slots, cost

**Register** Peacekeeper in registry.rs `build_registry()`

**Keep old functions as thin wrappers** that delegate to the new entity definition (for backward compatibility during migration). E.g., `peacekeeper_type_data()` can call `Peacekeeper::base_stats()` internally.

**This task validates the pattern.** If the trait design needs adjustment, adapt it here before other entities follow.

**Tests:**
- Stats match current peacekeeper_type_data() values (HP=50, armor 1/1, LightInfantry, GDO)
- Attack data matches current peacekeeper_attack_data() values
- Command slots match current grid_layout.rs entries for Peacekeeper
- Cost matches BarracksState::production_cost(Peacekeeper) (50 SC, 80 frames)
- Registry lookup for ObjectEnum::Peacekeeper returns correct entry
- Spawning via entity definition produces same components as current spawn_peacekeeper()

**Reference:** Architecture doc Phase 2 (Pilot Entity)

## Technical Context

### Files to Create

**`artifacts/developer/src/game/entities/gdo/units/peacekeeper.rs`** — The new Peacekeeper entity definition file. This is the primary deliverable.

### Files to Modify

1. **`artifacts/developer/src/game/entities/gdo/units/mod.rs`** (currently 1 line: `//! GDO unit definitions`) — Add `pub mod peacekeeper;`

2. **`artifacts/developer/src/game/entities/registry.rs`** (lines 1-40) — In `build_registry()` (line 8), replace the empty registry with:
   ```rust
   let mut registry = EntityRegistry::new();
   registry.register_unit::<gdo::units::peacekeeper::Peacekeeper>();
   registry
   ```
   Add the import: `use super::gdo::units::peacekeeper::Peacekeeper;`
   Update the test `test_build_registry_is_empty` (line 28) — it should now assert `registry.len() == 1` and verify the Peacekeeper entry exists.

3. **`artifacts/developer/src/game/units/types/unit_data.rs`** — Make `peacekeeper_type_data()` (line 120) and `peacekeeper_attack_data()` (line 133) thin wrappers that delegate to the new definition. Keep `PEACEKEEPER_CONTROL_COST` and `PEACEKEEPER_RUGGED_BONUS` constants re-exported from the new file for backward compat.

### Trait Pattern to Implement (defined in `types.rs` lines 80-95)

```rust
pub trait EntityDefinition {
    const OBJECT: ObjectEnum;
    fn object_type() -> ObjectType;
    fn base_stats() -> BaseStats;
    fn cost() -> Option<StructureCost> { None }
    fn command_slots() -> &'static [CommandSlotDef] { &[] }
    fn spawn(commands: &mut EntityCommands, params: SpawnParams);
}

pub trait UnitDefinition: EntityDefinition {
    fn unit_stats() -> UnitStats;
    fn attack_data() -> Option<AttackAttributesData> { None }
    fn movement_params() -> MovementParams;
}
```

### Data to Migrate

**BaseStats** (from `peacekeeper_type_data()` at unit_data.rs:120-130):
- `max_hp: 50`, `point_armor: 1`, `full_armor: 1`, `faction: FactionEnum::GlobalDefenseOrdinance`

**ObjectType** (from `ObjectEnum::Peacekeeper.object_type()` at objects.rs:219-225):
- `name: "Peacekeeper"`, `size: (24, 24)`, `destructible: true`, `sight_range: 5`, `groupable: true`

**UnitStats**:
- `unit_base: UnitBaseEnum::LightInfantry`, `silhouette_width: 24`, `silhouette_height: 24`, `control_cost: 1` (from `PEACEKEEPER_CONTROL_COST`)

**AttackAttributesData** (from `peacekeeper_attack_data()` at unit_data.rs:133-149):
- `attack_type: FullyConnected`, `fc_subtype: Some(Ranged)`, `target_domain: Ground`, `target_type: SingleTarget`, `damage: 10`, `range: 4`, `min_range: 0`, `aim_duration: 2`, `firing_duration: 1`, `cooldown_duration: 2`, `reload_duration: 12`, no projectile_speed, no aoe_radius

**MovementParams** (from `spawn_peacekeeper()` at utils.rs:444-449):
- `MovementParams::TurnRate(TurnRateMovementParams { turn_rate: 180.0_f32.to_radians() * 16.0, acceleration: f32::MAX, deceleration: f32::MAX, max_speed: 1.0 })`
- max_speed calculation: `4.0 * FRAMES_PER_SECOND(16) / SPACE_UNITS_PER_GRID_UNIT(64) = 1.0`

**Cost** (from `BarracksState::production_cost()` at structures.rs:150-155):
- `Some(StructureCost { space_crystals: 50, build_frames: 80 })`

**Additional constants**:
- `PEACEKEEPER_RUGGED_BONUS: f32 = 0.5` (unit_data.rs:155)

### Command Slots

Peacekeeper uses `ObjectInterfaceState::Default` (it's a standard groupable unit, not an agent/chopper). The command slots from grid_layout.rs:147-156 (filtered by Peacekeeper's capabilities: `has_attack=true`, `can_target_ground=true`, `can_reverse=false`, `is_chopper=false`):

```
(Default, 0, 0) => UnitMove
(Default, 0, 2) => UnitAttack          // has_attack=true
(Default, 1, 0) => UnitAttackMove      // has_attack=true
(Default, 1, 1) => UnitPatrol          // has_attack=true
(Default, 2, 0) => UnitHoldPosition
(Default, 2, 1) => UnitStop
```

Note: `UnitReverse` at (0,1) is NOT included (`can_reverse=false` for LightInfantry). `AttackGround` at (1,2) IS included (`can_target_ground=true`). Verify these against current grid_layout.rs. The `CommandSlotDef` entries should represent the superset of slots for Peacekeeper across all states — conditional enablement (e.g., has_attack) is a future Phase 2 concern per the TODO comment in types.rs:50-51. For now, include all slots that apply to Peacekeeper.

### Spawn Function

The `spawn` trait method takes `&mut EntityCommands` (not `&mut Commands`). The current `spawn_peacekeeper` in utils.rs:399-493 uses `commands.spawn((...))` which returns a new entity. The trait's `spawn` receives an already-spawned `EntityCommands`, so the implementation should use `commands.insert((...))` instead.

**Key difference**: The trait spawn does NOT receive `meshes`/`materials` resources. The current spawn_peacekeeper creates a `Capsule3d` mesh and `StandardMaterial`. Options:
1. Accept that spawn via EntityDefinition won't add visual components (defer to a separate visual setup system)
2. Add mesh/material creation inside spawn using `EntityCommands` — but this requires access to `Assets<Mesh>` and `Assets<StandardMaterial>` which aren't in `SpawnParams`

**Recommendation**: For the pilot, have `spawn()` insert all non-visual ECS components (the gameplay-relevant ones: Unit, ObjectInstance, owner, UnitType, Selectable, SelectionBounds, GridPosition, UnitBaseEnum, MovementSpeed, RotationSpeed, Velocity, attack_capability, AttackState, UnitCommand, TurnRateMovementParams, domain, UnitControlCost, RuggedTerrainDefenseBonus, CommandQueue, BaseCommandState, BaseBehaviorState, channels, Armor, Silhouette, SightRange, VisibilityStateEnum). Mesh/material can be added separately by callers that have asset access. Note this in tests.

**Components inserted by spawn_peacekeeper** (utils.rs:451-493):
- `Mesh3d`, `MeshMaterial3d`, `Transform` (visual — may need special handling)
- `Unit`, `ObjectInstance::destructible(Peacekeeper, 50.0)`, `owner`, `UnitType { name: "Peacekeeper" }`
- `Selectable`, `SelectionBounds::unit()`, `GridPosition { x, z }`
- `UnitBaseEnum::LightInfantry`, `MovementSpeed(1.0)`, `RotationSpeed(10.0)`, `Velocity(Vec3::ZERO)`
- `AttackCapability { ... }`, `AttackState::default()`, `UnitCommand::Idle`
- `TurnRateMovementParams { ... }`, `DomainEnum::Ground`
- `UnitControlCost(1)`, `RuggedTerrainDefenseBonus(0.5)`
- `CommandQueue::new()`, `BaseCommandState::default()`, `BaseBehaviorState::default()`
- `LocomotionChannel::default()`, `OrientationChannel::default()`, `BaseAttackChannel::default()`
- `Armor { point_armor: 1.0, full_armor: 1.0, directional_armor: ... }`
- `Silhouette { width: 24.0/64.0, height: 24.0/64.0 }`
- `SightRange(5)`, `VisibilityStateEnum::Visible`

### Key Imports

The new file will need imports from:
- `crate::types::{ObjectEnum, FactionEnum, UnitBaseEnum, Owner, DomainEnum, ...}`
- `crate::game::entities::types::{EntityDefinition, UnitDefinition, BaseStats, UnitStats, CommandSlotDef, SpawnParams}`
- `crate::game::types::objects::ObjectType`
- `crate::game::types::structures::StructureCost`
- `crate::game::units::types::unit_data::AttackAttributesData`
- `crate::game::units::types::movement::{MovementParams, TurnRateMovementParams}`
- `crate::ui::types::{ObjectInterfaceState, CommandButtonAction}`
- `crate::simulation::{FRAMES_PER_SECOND, SPACE_UNITS_PER_GRID_UNIT}` (for movement speed calculation)
- Combat types: `crate::game::combat::types::{AttackCapability, AttackType, AttackState}`
- Various unit components from `crate::game::units::types::*`

### Existing Test Patterns

The existing tests in `types.rs:189-391` show the pattern for registry tests — create a registry, register, look up, assert fields. The peacekeeper.rs tests should follow this pattern but use real Peacekeeper values instead of test stubs.

Existing peacekeeper tests in unit_data.rs (lines 271-456) verify specific stat values — these should still pass after the thin-wrapper refactor. The new tests in peacekeeper.rs should independently verify the same values through the EntityDefinition/UnitDefinition trait methods.

## Dependencies

1. **entity-definition-types-and-registry (already complete)** — The `EntityDefinition`, `UnitDefinition` traits, `BaseStats`, `UnitStats`, `CommandSlotDef`, `SpawnParams`, `EntityRegistry`, `RegistryEntry`, `EntityKind` types, and `EntitiesPlugin` are all defined in `artifacts/developer/src/game/entities/types.rs` and `registry.rs`. This task depends on these being in place, which they already are.

2. **No dependency on other migration tasks** — This is the pilot entity. It establishes the pattern that subsequent migration tasks (GDO units, buildings, Syndicate, Cults, neutral) will follow. Those tasks depend on this one, not vice versa.
