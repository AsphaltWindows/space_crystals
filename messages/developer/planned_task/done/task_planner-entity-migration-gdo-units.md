# entity-migration-gdo-units

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-declarative-entity-definitions.md

## Task

Migrate remaining GDO unit (SupplyChopper) into per-entity definition file following the pattern established by the Peacekeeper pilot.

**Create:** src/game/entities/gdo/units/supply_chopper.rs

Migrate from current sources:
- ObjectType/stats from objects.rs
- Type data, attack data (if any), constants from unit_data.rs
- Command panel slots from grid_layout.rs
- Spawn logic from spawn_supply_chopper() in utils.rs
- Production cost if applicable

Implement EntityDefinition + UnitDefinition traits. Register in build_registry(). Keep old functions as thin wrappers during transition.

**Tests per entity file:**
- Stats match current hardcoded values
- Attack data matches (if applicable)
- Command slots match current grid_layout.rs entries
- Cost matches current values
- Spawn produces same components

**Reference:** Architecture doc Phase 3 (GDO Entities)

## Technical Context

### File to Create

- **`artifacts/developer/src/game/entities/factions/gdo/units/supply_chopper.rs`** — New file implementing `EntityDefinition` + `UnitDefinition` for the SupplyChopper.

### Files to Modify

- **`artifacts/developer/src/game/entities/factions/gdo/units/mod.rs`** (line 1-2) — Currently just a doc comment. Add `pub mod supply_chopper;` and re-export the definition struct.
- **`artifacts/developer/src/game/entities/registry.rs`** (line 8-11) — Add `registry.register_unit::<SupplyChopperDef>()` in `build_registry()`. Update the empty-registry test (`test_build_registry_is_empty` at line 28) to expect 1 entry (or however many are registered at that point).

### Trait Pattern to Follow

Defined in `artifacts/developer/src/game/entities/types.rs`:

- **`EntityDefinition`** trait (line 81-88): `const OBJECT`, `object_type()`, `base_stats()`, `cost()`, `command_slots()`, `spawn()`
- **`UnitDefinition`** trait (line 91-95): `unit_stats()`, `attack_data()`, `movement_params()`
- **`EntityRegistry::register_unit::<T>()`** (line 128-149): Extracts all trait data into a `RegistryEntry`

### Current SupplyChopper Data to Consolidate

**ObjectType** — `artifacts/developer/src/game/types/objects.rs` line 282-288:
```rust
ObjectType { name: "Supply Chopper", size: (1,1), destructible: true, sight_range: 5, groupable: true }
```

**BaseStats** — `artifacts/developer/src/game/types/structures.rs` lines 436-438 (constants in `supply_tower_stats` module):
```rust
SC_MAX_HP = 150.0, SC_POINT_ARMOR = 1, SC_FULL_ARMOR = 1
// Faction: GlobalDefenseOrdinance
```

**UnitStats** — from spawn function `artifacts/developer/src/game/utils.rs` lines 909-947:
```rust
UnitBaseEnum::HoverCraft, control_cost: 0
// Silhouette: width=60/SPACE_UNITS_PER_GRID_UNIT, height=60/SPACE_UNITS_PER_GRID_UNIT
```
Note: silhouette_width/height in UnitStats are u32, but spawn uses `60.0 / SPACE_UNITS_PER_GRID_UNIT as f32`. Store raw pixel values (60, 60) in UnitStats.

**MovementParams** — `artifacts/developer/src/game/utils.rs` lines 896-907 (DragMovementParams):
```rust
DragMovementParams {
    forward_acceleration: 0.9 * FPS^2 / SUPGU,
    non_forward_acceleration: 0.1 * FPS^2 / SUPGU,
    drag_ratio: 0.1 * FPS,
    turn_rate: 10.0_f32.to_radians() * FPS,
}
```
Where `FRAMES_PER_SECOND` and `SPACE_UNITS_PER_GRID_UNIT` are from `crate::simulation`. Return `MovementParams::Drag(DragMovementParams { ... })`.

**AttackData** — None. SupplyChopper is unarmed (utils.rs line 934 comment).

**Production Cost** — `artifacts/developer/src/game/types/structures.rs` line 342-345:
```rust
StructureCost { space_crystals: 100, build_frames: 160 }
```

**Command Slots** — `artifacts/developer/src/ui/command_panel/grid_layout.rs` lines 137-146. These are under `ObjectInterfaceState::Default` with `caps.is_chopper` guard:
```
(0,0) UnitMove, (0,1) ChopperPickUpSupplies, (0,2) ChopperAttachToTower,
(1,0) ChopperDropOffSupplies, (1,2) UnitHoldPosition, (2,1) UnitStop
```
Map these to `CommandSlotDef` array with `state: ObjectInterfaceState::Default`.

### Spawn Function

Current: `spawn_supply_chopper()` at `artifacts/developer/src/game/utils.rs` line 878-948.

The `EntityDefinition::spawn()` receives `&mut EntityCommands` (not `&mut Commands`), so it **cannot** create mesh/material assets. The spawn impl should insert all gameplay ECS components:
- `Unit`, `ObjectInstance::destructible(ObjectEnum::SupplyChopper, SC_MAX_HP)`, `owner` (from params)
- `UnitType { name: "Supply Chopper" }`, `Selectable`, `SelectionBounds::unit()`
- `GridPosition` (derive from `params.position`), `UnitBaseEnum::HoverCraft`
- `MovementSpeed(max_speed)`, `RotationSpeed(10.0)`, `Velocity(Vec3::ZERO)`, `UnitCommand::Idle`
- `DragMovementParams` (computed), `DomainEnum::Air`
- `SupplyChopperState::default()`, `CommandQueue::new()`, `BaseCommandState::default()`
- `BaseBehaviorState::default()`, `LocomotionChannel::default()`, `OrientationChannel::default()`
- `Armor { point_armor, full_armor, directional_armor: false }`
- `Silhouette { width, height }`, `SeparationRadius(1.25)`
- `SightRange(5)`, `VisibilityStateEnum::Visible`
- `Transform::from_xyz(world_x, 1.5, world_z)` — note y=1.5 for hover height

Visual components (`Mesh3d`, `MeshMaterial3d`) are NOT inserted by spawn — caller handles those. The existing `spawn_supply_chopper()` in utils.rs should become a thin wrapper that creates the mesh/material, calls `commands.spawn((...))`, then calls `SupplyChopperDef::spawn()` on the entity commands, and returns the entity id.

### Key Imports Needed

```rust
use crate::game::entities::types::*;  // EntityDefinition, UnitDefinition, BaseStats, UnitStats, etc.
use crate::game::types::objects::{ObjectType, ObjectEnum};
use crate::game::types::structures::StructureCost;
use crate::game::units::types::movement::{DragMovementParams, MovementParams};
use crate::types::{FactionEnum, UnitBaseEnum};
use crate::ui::types::{ObjectInterfaceState, CommandButtonAction};
use crate::simulation::{FRAMES_PER_SECOND, SPACE_UNITS_PER_GRID_UNIT};
```

### Testing Pattern

Follow the test patterns in `artifacts/developer/src/game/entities/types.rs` (lines 189-391). Each test should:
1. Construct the definition via the trait methods
2. Assert values match the hardcoded constants
3. For command_slots: verify count and specific (row, col, action) tuples
4. For cost: verify `Some(StructureCost { space_crystals: 100, build_frames: 160 })`
5. For attack_data: verify `None`
6. For movement_params: verify `MovementParams::Drag(...)` with correct values

### Existing Module Wiring

The directory structure already exists:
- `artifacts/developer/src/game/entities/factions/gdo/units/mod.rs` — exists, just has a doc comment about SupplyChopper
- `artifacts/developer/src/game/entities/factions/gdo/units/types.rs` — exists (empty)
- `artifacts/developer/src/game/entities/factions/gdo/units/utils.rs` — exists (empty)

## Dependencies

- **Entity definition foundation** (`entity-definition-cleanup` or the foundation task from Phase 1) — The `EntityDefinition`/`UnitDefinition` traits and `EntityRegistry` must exist. These are already implemented in `artifacts/developer/src/game/entities/types.rs` and `artifacts/developer/src/game/entities/registry.rs`, so this dependency is **satisfied**.
- **No dependency on Peacekeeper pilot** — The task says "following the pattern established by the Peacekeeper pilot," but no Peacekeeper definition file exists yet (the `peacekeeper.rs` file hasn't been created). The pattern is fully specified by the traits in `types.rs` and the architecture doc. The SupplyChopper migration can proceed independently.
