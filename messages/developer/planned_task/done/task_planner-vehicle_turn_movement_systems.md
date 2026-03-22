# vehicle_turn_movement_systems

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-unit_bases_movement_collision_r1.md

## Task

Implement two new movement systems for vehicle-type units and register them in `UnitsPlugin`:

### 1. `fixed_turn_radius_movement_system` (WheeledVehicle)

Movement system for entities with `FixedTurnRadiusMovementParams` component. Behavior per design doc:
- Unit cannot turn in place — must be moving to change heading
- Turns at a fixed minimum radius regardless of speed (`minimum_turn_radius` field)
- Can stop and reverse (forward/reverse have separate acceleration and max speed)
- Forward acceleration/deceleration and reverse acceleration from params
- Ground collision via `OccupancyMap` (same pattern as `turn_rate_movement_system`)
- Path following via `MoveTarget` + `Path` components (same waypoint logic)
- When turning toward a waypoint, if angle to waypoint exceeds what can be achieved at min turn radius, the unit follows an arc rather than turning sharply

### 2. `speed_turn_radius_movement_system` (TrackedVehicle)

Movement system for entities with `SpeedTurnRadiusMovementParams` component. Behavior per design doc:
- Can rotate in place (spin treads in opposite directions) when stationary
- Turn radius increases with speed: `turn_radius = speed * speed_to_turn_radius_ratio`
- At zero/low speed, can make very tight turns; at high speed, turns are wide
- Sharp turns at speed require slowing down first
- Acceleration/deceleration from params
- Ground collision via `OccupancyMap`
- Path following via `MoveTarget` + `Path` components

### Registration

Add both systems to `UnitsPlugin` in Phase 3 (movement systems), alongside `unit_movement_system` and `turn_rate_movement_system`. Ensure `grid_position_sync_system` runs after them.

## Technical Context

### Primary file to modify
- **`artifacts/developer/src/game/units/systems/core.rs`** — Add both new movement system functions here, alongside `turn_rate_movement_system` (line 1223) and `unit_movement_system` (line 1083). This is the canonical location for all MoveTarget/Path-driven movement systems.

### Registration file
- **`artifacts/developer/src/game/units/mod.rs`** — Register both systems in `UnitsPlugin::build()`. Add them to the Phase 3 block (lines 44-54) alongside existing movement systems. Also add `.after()` constraints for `grid_position_sync_system` (lines 57-61) to ensure grid sync runs after the new systems.

### Movement param types (read-only reference)
- **`artifacts/developer/src/game/units/types/movement.rs`**:
  - `FixedTurnRadiusMovementParams` (line 138): fields are `minimum_turn_radius`, `forward_acceleration`, `forward_max_speed`, `reverse_acceleration`, `reverse_max_speed`, `deceleration`
  - `SpeedTurnRadiusMovementParams` (line 149): fields are `speed_to_turn_radius_ratio`, `acceleration`, `deceleration`, `max_speed`
  - Helper methods already exist: `FixedTurnRadiusMovementParams::max_turn_rate_at_speed(speed)` returns `speed / minimum_turn_radius`; `SpeedTurnRadiusMovementParams::max_turn_rate_at_speed(speed)` returns `speed / (speed * ratio)`
  - `UnitBaseData` (line 287): WheeledVehicle uses `FixedTurnRadius`, TrackedVehicle and DrillUnit use `SpeedTurnRadius`

### Pattern to follow: `turn_rate_movement_system` (core.rs:1223-1318)
This is the closest reference implementation. Key patterns to replicate:

1. **Query structure**: `Query<(Entity, &mut Transform, &mut Velocity, &PARAMS_TYPE, &MoveTarget, Option<&mut Path>, Option<&AttackState>, Option<&Turret>, &mut UnitCommand, Option<&Silhouette>, Option<&DomainEnum>), (With<Unit>, Without<HoldingPosition>)>`
2. **Query exclusion**: `unit_movement_system` uses `Without<TurnRateMovementParams>` to avoid overlap. The new systems must use their specific param component as the required filter (e.g., `With<FixedTurnRadiusMovementParams>` is implicit via `&FixedTurnRadiusMovementParams` in the query). Since `unit_movement_system` already excludes `TurnRateMovementParams` entities, it will also need `Without<FixedTurnRadiusMovementParams>` and `Without<SpeedTurnRadiusMovementParams>` added to its filter to avoid processing vehicle units. Same for `channel_fallback_locomotion_system` (line 1925) which has `Without<TurnRateMovementParams>`.
3. **Delta guard**: Early return if `delta < 0.0001`
4. **Attack phase constraints**: Check `AttackState.phase.base_action_constraints(is_turret_source)` — if `!constraints.base_can_move`, set velocity to zero and skip
5. **Waypoint logic**: Use `Path.waypoints[current_waypoint]`, advance when within `WAYPOINT_ARRIVAL_THRESHOLD` (0.5), remove `MoveTarget` + `Path` when all waypoints consumed, set `UnitCommand::Idle` on completion for Move commands
6. **Ground collision**: Check `occupancy.check_movement_collision(entity, proposed_x, proposed_z, half_w, half_h)` — on collision, set velocity zero, remove `Path`, insert `NeedsRepath`
7. **Y-axis handling**: Ground units at y=0.5, air units at y=1.5 (line 1317)

### FixedTurnRadius-specific behavior
- **Cannot turn in place**: When stationary and facing wrong direction, must move forward to begin turning. The turn rate while moving = `current_speed / minimum_turn_radius` (radians/sec) — use the existing `max_turn_rate_at_speed()` method.
- **Reverse support**: WheeledVehicle has `can_reverse: true`. When waypoint is behind the unit (angle > ~120°), consider reversing instead of making a wide U-turn. Reverse uses `reverse_acceleration` and `reverse_max_speed`.
- **Forward/reverse acceleration**: Use `forward_acceleration` and `forward_max_speed` for forward movement; `reverse_acceleration` and `reverse_max_speed` for reverse.
- **Arc following**: At speed, the max turn rate is `speed / minimum_turn_radius`. If the angle to the next waypoint exceeds what can be turned in one frame, the unit follows the arc naturally (clamp turn to max_turn).

### SpeedTurnRadius-specific behavior
- **Can rotate in place**: When stationary (`speed ≈ 0`), turn rate is unconstrained — spin freely toward target heading (the constraint table returns `Unconstrained` for Stationary+Turning).
- **Speed-dependent turning**: At speed, `turn_radius = speed * speed_to_turn_radius_ratio`, so `max_turn_rate = speed / turn_radius = 1 / speed_to_turn_radius_ratio`. Use existing `max_turn_rate_at_speed()`.
- **Slow for sharp turns**: When angle to waypoint is large and speed is high, reduce target speed to allow tighter turns. Pattern: if `angle_to_target > PI/2`, reduce speed significantly (similar to turn_rate_movement_system line 1278-1280).
- **Reverse support**: TrackedVehicle has `can_reverse: true`. Uses single `acceleration`/`deceleration`/`max_speed` (no separate forward/reverse params).

### Key imports needed (core.rs)
- `FixedTurnRadiusMovementParams` and `SpeedTurnRadiusMovementParams` from `crate::game::units::types::movement` — currently only `TurnRateMovementParams` is imported (line 8). Add both.
- All other types (`OccupancyMap`, `NeedsRepath`, `Velocity`, `MoveTarget`, `Path`, `Unit`, `HoldingPosition`, etc.) are already available via `crate::game::units::types::*` (line 11).

### Registration in mod.rs
Add to Phase 3 block (line 43-55):
```rust
systems::core::fixed_turn_radius_movement_system,
systems::core::speed_turn_radius_movement_system,
```

Add `grid_position_sync_system` ordering (after line 61):
```rust
.after(systems::core::fixed_turn_radius_movement_system)
.after(systems::core::speed_turn_radius_movement_system)
```

### Filter updates needed on existing systems
- **`unit_movement_system`** (core.rs:1090): Add `Without<FixedTurnRadiusMovementParams>, Without<SpeedTurnRadiusMovementParams>` to its filter tuple to prevent it from processing vehicle-type units.
- **`channel_fallback_locomotion_system`** (core.rs:1931): Add `Without<FixedTurnRadiusMovementParams>, Without<SpeedTurnRadiusMovementParams>` to prevent overlap with future channel-driven vehicle systems.

### Design doc reference
- **`artifacts/designer/design/units.md`** lines 54-70: FixedTurnRadiusMovement and SpeedTurnRadiusMovement specifications

## Dependencies

- **Existing `OccupancyMap` and `rebuild_occupancy_map`**: Both systems read the occupancy map for ground collision checks. Already runs in Phase 1, so Phase 3 ordering is correct.
- **Existing `collision_repath_system`**: Handles `NeedsRepath` markers that vehicle systems will insert on collision. Already in Phase 3.
- **`MoveTarget`/`Path` components**: Standard movement target and pathfinding waypoint components — already spawned on units when given move commands.
- **Sibling tasks `drag_glider_movement_systems` and `unit_crushing_mechanic`**: All share the same parent feature. No hard dependency between them — each can be implemented independently. However, the filter updates on `unit_movement_system` and `channel_fallback_locomotion_system` must also exclude `DragMovementParams` and `GliderMovementParams` if those tasks land first. Coordinate: whoever modifies the filter tuple first should add all 4 exclusions, or each task adds its own. Recommend adding all 4 at once in this task to avoid conflicts.
