# drag_glider_movement_systems

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-unit_bases_movement_collision_r1.md

## Task

Implement two new movement systems for momentum/physics-based units and register them in `UnitsPlugin`:

### 1. `drag_movement_system` (HoverVehicle, HoverCraft)

Movement system for entities with `DragMovementParams` component. Behavior per design doc:
- Unit accelerates with thrust: `OmniDirectionalAcceleration` in any direction + `ForwardAcceleration` in facing direction only
- Drag continuously opposes movement proportional to velocity (`drag_ratio`)
- Effective max speed = total_thrust / drag_ratio
- Turn rate from params (`turn_rate`) — unit rotates toward target heading
- Ground hover units (HoverVehicle, DomainEnum::Ground) actively thrust to decelerate and change direction — lower drag, more responsive
- Air units (HoverCraft, DomainEnum::Air) have high drag, rely on drag to stop — more slidey/momentum-based
- Ground collision via `OccupancyMap` for ground-domain units only
- Air units skip ground collision (existing pattern)
- Path following via `MoveTarget` + `Path` components
- When idle (no MoveTarget), drag slows unit to zero (ground) or near-zero (air)

### 2. `glider_movement_system` (Glider)

Movement system for entities with `GliderMovementParams` component. Behavior per design doc:
- Unit must always maintain movement to stay airborne
- When idle (no MoveTarget/command), unit circles at `idle_speed` in tight loops
- When given orders, accelerates to `max_speed`
- Turn radius governed by centripetal acceleration: `radius = speed^2 / max_centripetal_acceleration`
- Higher speeds -> wider turns; lower speeds -> tighter circles
- Acceleration/deceleration from params
- Air domain — no ground collision, uses `SeparationRadius` for soft separation (handled by existing `air_unit_separation_system`)
- Path following: unlike other systems, the glider cannot stop at waypoints — it flies through them and curves toward the next one

### Registration

Add both systems to `UnitsPlugin` in Phase 3 (movement systems). Ensure `grid_position_sync_system` runs after them.

## Technical Context

### Primary file to modify
- **`artifacts/developer/src/game/units/systems/core.rs`** — Add both new movement system functions here, alongside `turn_rate_movement_system` (line 1227) and `unit_movement_system` (line 1083). This is the canonical location for all MoveTarget/Path-driven movement systems.

### Registration file
- **`artifacts/developer/src/game/units/mod.rs`** — Register both systems in `UnitsPlugin::build()`. Add them to the Phase 3 block (lines 44-54) alongside existing movement systems. Also add `.after()` constraints for `grid_position_sync_system` (lines 57-61) to ensure grid sync runs after the new systems.

### Movement param types (read-only reference)
- **`artifacts/developer/src/game/units/types/movement.rs`**:
  - `DragMovementParams` (line 158): fields are `forward_acceleration: f32`, `non_forward_acceleration: f32`, `drag_ratio: f32`, `turn_rate: f32`
  - `GliderMovementParams` (line 167): fields are `idle_speed: f32`, `max_speed: f32`, `acceleration: f32`, `deceleration: f32`, `max_centripetal_acceleration: f32`
  - Helper methods already exist: `DragMovementParams::max_speed()` returns `(non_forward_acceleration + forward_acceleration) / drag_ratio`; `GliderMovementParams::turn_radius(speed)` returns `speed^2 / max_centripetal_acceleration`
  - Constraint methods: `DragMovementParams::locomotion_orientation_constraint()` — same pattern as TurnRate (FixedRate turn_rate for all Turning, Invalid for Reversing); `GliderMovementParams::locomotion_orientation_constraint()` — only Moving+Turning (SpeedDependent) and Moving+Maintaining are valid (all others Invalid — glider must always be moving)

### Pattern to follow: `turn_rate_movement_system` (core.rs:1227-1385)
This is the closest reference implementation. Key patterns to replicate:

1. **Query structure**: `Query<(Entity, &mut Transform, &mut Velocity, &PARAMS_TYPE, &MoveTarget, Option<&mut Path>, Option<&AttackState>, Option<&Turret>, &mut UnitCommand, Option<&Silhouette>, Option<&DomainEnum>), (With<Unit>, Without<HoldingPosition>)>`
2. **Delta guard**: Early return if `delta < 0.0001`
3. **Attack phase constraints**: Check `AttackState.phase.base_action_constraints(is_turret_source)` — if `!constraints.base_can_move`, set velocity to zero and skip
4. **Waypoint logic**: Use `Path.waypoints[current_waypoint]`, advance when within `WAYPOINT_ARRIVAL_THRESHOLD` (0.5), remove `MoveTarget` + `Path` when all waypoints consumed, set `UnitCommand::Idle` on completion for Move commands
5. **Ground collision**: Check `occupancy.check_movement_collision(entity, proposed_x, proposed_z, half_w, half_h)` — on collision, set velocity zero, remove `Path`, insert `NeedsRepath` (ground-domain units only)
6. **Y-axis handling**: Ground units at y=0.5, air units at y=1.5

### Drag movement system specifics
- **Physics model**: Each tick: (1) apply drag force to velocity: `velocity -= velocity * drag_ratio * delta`, (2) compute thrust direction from facing vs desired direction, (3) apply forward thrust: `velocity += facing_dir * forward_acceleration * delta`, (4) apply omni-directional thrust toward target: `velocity += desired_dir * non_forward_acceleration * delta`, (5) clamp velocity if needed
- **Turn rate**: Fixed turn rate from `params.turn_rate` (same as TurnRate model), unit rotates toward next waypoint using `Quat::from_rotation_y` pattern (see turn_rate_movement_system line 1323-1331)
- **Idle behavior**: When no MoveTarget, do NOT apply thrust — only drag decelerates the unit. The drag formula `velocity -= velocity * drag_ratio * delta` will naturally slow the unit to near-zero.
- **No reverse**: DragMovement does NOT support reversing (constraint returns Invalid for Reversing). Unit always thrusts forward or omni-directionally.
- **Domain awareness**: Ground hover units (DomainEnum::Ground) get ground collision checks. Air units (DomainEnum::Air) skip collision. Use the same `is_ground` pattern from turn_rate_movement_system (line 1366-1378).

### Glider movement system specifics
- **Always moving**: Glider never stops. When no MoveTarget, unit circles at `idle_speed` in tight loops. When given orders, accelerates toward `max_speed`.
- **Idle circling**: When no MoveTarget/Path, maintain current speed (decelerating toward `idle_speed` if above it) and apply a constant turn rate to create circling behavior. Turn rate for circling = `idle_speed / turn_radius(idle_speed)` = `max_centripetal_acceleration / idle_speed`.
- **Speed-dependent turn radius**: `turn_radius = speed^2 / max_centripetal_acceleration`, so `max_turn_rate = max_centripetal_acceleration / speed`. At low speeds (idle_speed=5, max_centripetal=10), radius=2.5 (tight circles). At max speed (15), radius=22.5 (wide turns).
- **Waypoint fly-through**: Unlike other movement systems, the glider should NOT stop at waypoints. When within `WAYPOINT_ARRIVAL_THRESHOLD` of a waypoint, advance to the next one immediately without decelerating. Only remove MoveTarget/Path when ALL waypoints are consumed (and even then, transition to idle circling rather than stopping).
- **Acceleration/deceleration**: Use `params.acceleration` to speed up toward `max_speed` when ordered, `params.deceleration` to slow toward `idle_speed` when idle.
- **Air domain only**: Glider is always air — no ground collision needed. Always set y=1.5. Separation handled by existing `air_unit_separation_system` (Glider spawns with `SeparationRadius(1.25)` — see core.rs line 166).
- **No reverse**: Glider only supports Moving locomotion (constraint returns Invalid for everything else).

### Key imports (core.rs line 8)
- `DragMovementParams` and `GliderMovementParams` are **already imported** from `crate::game::units::types::movement` (line 8).
- All other needed types (`OccupancyMap`, `NeedsRepath`, `Velocity`, `MoveTarget`, `Path`, `Unit`, `HoldingPosition`, `WAYPOINT_ARRIVAL_THRESHOLD`, etc.) are already available via existing imports.

### Filter updates on existing systems
- **`unit_movement_system`** (core.rs:1090-1092): Already has `Without<DragMovementParams>, Without<GliderMovementParams>` in its filter — no changes needed.
- **`unit_rotation_system`** (core.rs:1205-1207): Already has `Without<DragMovementParams>, Without<GliderMovementParams>` — no changes needed.
- **`channel_fallback_locomotion_system`** (core.rs:1935-1937): Already has `Without<DragMovementParams>, Without<GliderMovementParams>` — no changes needed.

### Registration in mod.rs
Add to Phase 3 block (line 43-55):
```rust
systems::core::drag_movement_system,
systems::core::glider_movement_system,
```

Add `grid_position_sync_system` ordering (after line 61):
```rust
.after(systems::core::drag_movement_system)
.after(systems::core::glider_movement_system)
```

### Test spawner reference
- **core.rs lines 141-167**: HoverVehicle entities spawn with `DragMovementParams { forward_acceleration: 6.0, non_forward_acceleration: 4.0, drag_ratio: 2.0, turn_rate: 2.5 }` — derived max_speed = (4+6)/2 = 5.0
- **core.rs lines 157-167**: Glider entities spawn with `GliderMovementParams { idle_speed: 5.0, max_speed: 15.0, acceleration: 3.0, deceleration: 6.0, max_centripetal_acceleration: 10.0 }` and `SeparationRadius(1.25)`
- All test units also have `Velocity(Vec3::ZERO)`, `MoveTarget`/`Path` not initially present, `DomainEnum` from `unit_base.data().domain`

### Design doc reference
- **`artifacts/designer/design/units.md`** lines 72-87: DragMovement and GliderMovement specifications

## Dependencies

- **Existing `OccupancyMap` and `rebuild_occupancy_map`**: The drag_movement_system reads the occupancy map for ground collision checks. Already runs in Phase 1, so Phase 3 ordering is correct.
- **Existing `collision_repath_system`**: Handles `NeedsRepath` markers that drag_movement_system will insert on ground collision. Already in Phase 3.
- **Existing `air_unit_separation_system`**: Provides soft separation for air units (Glider, HoverCraft). Already in Phase 3, runs after all movement.
- **`MoveTarget`/`Path` components**: Standard movement target and pathfinding waypoint components — already spawned on units when given move commands.
- **Sibling task `vehicle_turn_movement_systems`**: Shares the same parent feature. No hard dependency — each can be implemented independently. The `unit_movement_system` and `channel_fallback_locomotion_system` filters already exclude both `DragMovementParams` and `GliderMovementParams`, so no filter coordination is needed.
- **Sibling task `unit_crushing_mechanic`**: Independent — no dependency.
