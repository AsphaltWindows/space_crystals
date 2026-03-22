# test_unit_spawner_all_bases

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-unit_bases_movement_collision_r1.md

## Task

Expand `spawn_test_units` in `game/units/systems/core.rs` to spawn one properly-configured representative unit for each of the 9 UnitBaseEnum variants, so QA can test all movement models and associated mechanics.

Currently spawns: 4 Peacekeepers (LightInfantry) + 3 placeholder enemy units (WheeledVehicle, TrackedVehicle, DrillUnit) with wrong ObjectEnum and placeholder stats.

Required changes:
1. For Player 0 (GDO), ensure at least a Peacekeeper (LightInfantry) is spawned — already done.
2. For Player 1 (Syndicate), ensure Agent (HeavyInfantry) and Guard (HeavyInfantry) are spawned — use `spawn_syndicate_agent` and `spawn_syndicate_guard` from `game/utils.rs`.
3. Replace the 3 placeholder enemy units with properly-configured test units covering the missing base types. For each, attach the correct movement param component:
   - **WheeledVehicle**: `FixedTurnRadiusMovementParams` component (use values from `UnitBaseEnum::WheeledVehicle.data()` movement model)
   - **TrackedVehicle**: `SpeedTurnRadiusMovementParams` component
   - **HoverVehicle**: `DragMovementParams` component (ground domain)
   - **Mech**: `TurnRateMovementParams` component + turret components (`TurretCommandState`, `TurretBehaviorState`, `TurretOrientationChannel`, `TurretAttackChannel`, `Turret`) + directional armor
   - **Glider**: `GliderMovementParams` component (air domain)
   - **DrillUnit**: keep, ensure underground domain
4. Each test unit must have: correct `UnitBaseEnum`, `DomainEnum`, `Armor` (with `directional_armor` from base data), `Silhouette`, movement params, `Velocity`, `CommandQueue`, `BaseCommandState`, `BaseBehaviorState`, channel components, `UnitControlCost`.
5. For air units (HoverCraft already exists via SupplyChopper, Glider), ensure `SeparationRadius` component is attached.
6. Spawn units at distinct grid positions so they don't overlap.

Note: HoverCraft (air drag movement) is already testable via SupplyChopper — no need to add another.

## Technical Context

### Primary file to change
- **`artifacts/developer/src/game/units/systems/core.rs`** — `spawn_test_units` function (lines 25-140)

### Current state of `spawn_test_units` (lines 25-140)
- Lines 30-37: Spawns 4 GDO Peacekeepers via `spawn_peacekeeper()` at (28-31, 32) — **keep as-is**
- Lines 40-47: Defines 3 enemy units in a tuple array with hardcoded stats — **replace this block**
- Lines 51-137: Loop that spawns each enemy with placeholder config — **replace with proper spawns**
- Bug: All 3 enemy units use `ObjectEnum::Peacekeeper` (line 72) regardless of actual type — must fix

### How to spawn Syndicate units (Agent + Guard)
- Import and call `crate::game::utils::spawn_syndicate_agent` and `crate::game::utils::spawn_syndicate_guard`
- Already imported on line 9: only `spawn_peacekeeper` — add the other two to the import
- Signature: `spawn_syndicate_agent(commands, meshes, materials, grid_x, grid_z, owner) -> Entity`
- These handle all components properly: correct ObjectEnum, TurnRateMovementParams, AttackCapability, Armor, Silhouette, SightRange, etc.
- Use `Owner::player(1)` for Syndicate units
- Place at distinct positions, e.g., (34, 32) and (35, 32)

### How to spawn vehicle/mech/glider test units (no existing spawn helpers)
The current placeholder loop (lines 51-137) is a reasonable template but needs these fixes per unit:

**For each test unit, use correct ObjectEnum**:
- There are NO dedicated ObjectEnum variants for WheeledVehicle, TrackedVehicle, HoverVehicle, Mech, Glider, or DrillUnit yet. The only unit ObjectEnums are: `Peacekeeper`, `SupplyChopper`, `SyndicateAgent`, `SyndicateGuard`.
- Continue using `ObjectEnum::Peacekeeper` as a placeholder ObjectEnum for these test units (they're just for movement testing), but use correct `UnitBaseEnum` and correct movement params. Add a TODO comment noting these need proper ObjectEnum variants later.

**Movement param components** (all types available via `use crate::game::units::types::*` on line 11):
- `FixedTurnRadiusMovementParams` — for WheeledVehicle. Fields: `minimum_turn_radius`, `forward_acceleration`, `forward_max_speed`, `reverse_acceleration`, `reverse_max_speed`, `deceleration`. Use placeholder values (e.g., sample values from movement.rs tests: radius=4.0, fwd_accel=5.0, fwd_max=10.0, rev_accel=3.0, rev_max=5.0, decel=8.0).
- `SpeedTurnRadiusMovementParams` — for TrackedVehicle and DrillUnit. Fields: `speed_to_turn_radius_ratio`, `acceleration`, `deceleration`, `max_speed`. Sample: ratio=0.5, accel=5.0, decel=10.0, max_speed=12.0.
- `DragMovementParams` — for HoverVehicle (ground domain). Fields: `forward_acceleration`, `non_forward_acceleration`, `drag_ratio`, `turn_rate`. Sample: fwd=6.0, non_fwd=4.0, drag=2.0, turn=2.5.
- `TurnRateMovementParams` — for Mech. Fields: `turn_rate`, `acceleration`, `deceleration`, `max_speed`. Already imported on line 8. Sample: turn=3.0, accel=5.0, decel=10.0, max_speed=8.0.
- `GliderMovementParams` — for Glider (air domain). Fields: `idle_speed`, `max_speed`, `acceleration`, `deceleration`, `max_centripetal_acceleration`. Sample: idle=5.0, max=15.0, accel=3.0, decel=6.0, max_cent=10.0.

**Turret handling** (already correctly done in current code, lines 117-134):
- `unit_base.data().has_turret` determines turret vs base attack channels
- `create_turret_for_unit(&unit_base)` returns `Option<Turret>` — already imported on line 7
- `spawn_turret_visual()` creates turret mesh child — already imported on line 7
- Turret-bearing units: WheeledVehicle, TrackedVehicle, DrillUnit, HoverVehicle, Mech, Glider
- Non-turret: LightInfantry, HeavyInfantry

**Armor with directional_armor** (already correctly done, line 109):
- `base_data.directional_armor` — true for vehicles/mechs, false for infantry/air

**SeparationRadius for air units** (from `crate::game::combat::types`, already imported on line 5):
- Only Glider needs `SeparationRadius` among new test units (HoverCraft is tested via SupplyChopper)
- Value: use 1.25 (same as SupplyChopper, see game/utils.rs line 939)

**Domain** (from `UnitBaseEnum::data().domain`):
- Ground: WheeledVehicle, TrackedVehicle, HoverVehicle, Mech
- Underground: DrillUnit
- Air: Glider

**SelectionBounds** (from `crate::types`):
- Use `SelectionBounds::unit()` for all test units (same as Peacekeeper/Agent/Guard spawn functions)

**SightRange** — use placeholder value (e.g., 5) since these test units have no ObjectEnum-based sight_range

### Suggested spawn positions (non-overlapping)
Keep Peacekeepers at (28-31, 32). Add:
- Agent: (33, 32), Guard: (34, 32)
- WheeledVehicle: (36, 32), TrackedVehicle: (37, 32), HoverVehicle: (38, 32)
- Mech: (39, 32), Glider: (40, 32), DrillUnit: (41, 32)
Or spread vertically for better visibility.

### Pattern to follow
Follow the spawn pattern in `spawn_peacekeeper` (game/utils.rs:396-490):
1. Compute world position from grid: `let world_x = (grid_x as f32 - 32.0) + 0.5;`
2. Create mesh + material
3. Spawn entity with: `Unit`, `ObjectInstance`, `Owner`, `UnitType`, `Selectable`, `SelectionBounds::unit()`, `GridPosition`, `UnitBaseEnum`, `DomainEnum`, `MovementSpeed`, `RotationSpeed`, `Velocity`
4. Insert: attack capability, `AttackState`, `UnitCommand::Idle`, movement params, `UnitControlCost`, `CommandQueue`, `BaseCommandState`, `BaseBehaviorState`, channels
5. Insert: `Armor`, `Silhouette`, `SightRange`
6. Conditionally insert turret components (channels + Turret component + visual)

### Key imports already available in core.rs
- Line 5: `AttackState`, `Turret`, `Armor`, `Silhouette`, `SeparationRadius`
- Line 7: `create_turret_for_unit`, `spawn_turret_visual`
- Line 8: `TurnRateMovementParams` — but other movement param types come via line 11's wildcard `use crate::game::units::types::*`
- Line 11: wildcard re-exports `movement::*` (all 5 param types) and `state::*` (channels, commands, behaviors)

### Import to add
- Line 9: Change `use crate::game::utils::spawn_peacekeeper;` to `use crate::game::utils::{spawn_peacekeeper, spawn_syndicate_agent, spawn_syndicate_guard};`

## Dependencies

- **`vehicle_turn_movement_systems`** (sibling task): Creates the movement consumer systems that will actually drive these test units. The spawner must attach the correct movement param components so those systems can query them. No code dependency — just component compatibility.
- **Existing `spawn_peacekeeper`/`spawn_syndicate_agent`/`spawn_syndicate_guard`** in `game/utils.rs`: Used directly for infantry units. Already complete and working.
- **`create_attack_capability`** in `game/units/utils.rs`: Already used in current code (line 89) — provides placeholder attack stats per UnitBaseEnum. Works for all 9 variants.
