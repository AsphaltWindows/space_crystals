# action-channel-locomotion-orientation

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-action-channels.md

## Task

Create/refactor the systems that consume LocomotionChannel and OrientationChannel to drive actual unit movement and rotation. Currently, behavior systems write to these channels (e.g., `LocomotionChannel::Moving(path)`, `OrientationChannel::Turning(target)`) but the existing movement systems (`unit_movement_system`, `turn_rate_movement_system`, `unit_rotation_system` in `game/units/systems/core.rs`) still use the older `MoveTarget`/`Path` component pattern and do NOT read from the channels.

**What to implement:**

1. **Locomotion consumer system**: A system that reads `LocomotionChannel` and drives `Transform`/`Velocity`:
   - `Moving(path)`: move unit along the given waypoint path, applying speed and acceleration
   - `Reversing(path)`: move unit backward along path (CanReverse bases only)
   - `Stopping`: decelerate to zero velocity
   - `Stationary`: velocity = 0, no movement

2. **Orientation consumer system**: A system that reads `OrientationChannel` and rotates the unit base:
   - `Turning(target_pos)`: compute angle to target, apply base turn rate per tick. For FixedTurnRadiusMovement bases, constrain turning by locomotion state (use `locomotion_orientation_constraint()` from movement param structs)
   - `Maintaining`: hold current facing

3. **Refactor existing movement systems**: Replace or refactor `unit_movement_system`, `turn_rate_movement_system`, and `unit_rotation_system` so movement is driven by the channel values rather than `MoveTarget`/`Path` components. Remove `MoveTarget`/`Path` usage if fully superseded.

4. **Collision/occupancy integration**: Ensure the new channel-driven movement still respects `OccupancyMap` ground collision (existing AABB checks) and `air_unit_separation_system`.

5. **System ordering**: Register consumer systems in Phase 3 of UnitsPlugin (after behavior systems write to channels in Phase 2).

**Design reference:** `artifacts/designer/design/control_system.md` — BaseActionChannels, LocomotionChannel, OrientationChannel sections.

## Technical Context

### Files to Modify

1. **`artifacts/developer/src/game/units/systems/core.rs`** — Primary work file
   - `unit_movement_system` (line 759): Handles movement for non-TurnRate units via `MoveTarget`/`Path` components. Query: `(Entity, &mut Transform, &mut Velocity, &MovementSpeed, &MoveTarget, Option<&mut Path>, ...)` with `Without<TurnRateMovementParams>`.
   - `turn_rate_movement_system` (line 899): Handles TurnRate infantry movement. Uses turn-toward-waypoint logic with `params.turn_rate`, acceleration/deceleration, and `MoveTarget`/`Path`. This is the **primary reference** for channel-driven TurnRate locomotion+orientation.
   - `unit_rotation_system` (line 875): Simple rotation-toward-velocity for non-TurnRate units. Query: `(&mut Transform, &Velocity, &RotationSpeed)` with `Without<TurnRateMovementParams>`.
   - `collision_repath_system` (line 1113): Recomputes paths for units with `NeedsRepath`. Currently uses `MoveTarget` to determine destination. Will need to read target from `LocomotionChannel::Moving(path)` or similar instead.
   - `air_unit_separation_system` (line 1354): Modifies `Transform` directly for air unit separation. Independent of movement channels — no changes needed, just ensure compatibility.

2. **`artifacts/developer/src/game/units/mod.rs`** — Plugin registration
   - Phase 3 (line 31-37): Currently registers `unit_movement_system`, `turn_rate_movement_system`, `collision_repath_system`, `unit_rotation_system`, `air_unit_separation_system` as a tuple `.after(systems::core::rebuild_occupancy_map)`.
   - New channel consumer systems should replace or be registered alongside existing systems in Phase 3.
   - Ensure `.after()` ordering chains behaviors (Phase 2) → channel consumers (Phase 3) → grid_position_sync (Phase 4).

3. **`artifacts/developer/src/game/units/types/movement.rs`** — Movement model types (read-only reference)
   - `MoveTarget(pub Vec3)` (line 7): Component used by old pipeline. Will be deprecated/removed.
   - `Path { waypoints, current_waypoint }` (line 35): Component used by old pipeline. Will be deprecated/removed.
   - `Velocity(pub Vec3)` (line 11): Keep — channel consumers write to this.
   - `MovementSpeed(pub f32)` (line 15): Keep — used for non-parameterized speed.
   - `RotationSpeed(pub f32)` (line 19): Keep — used for non-TurnRate rotation.
   - Movement param structs (lines 128-183): `TurnRateMovementParams`, `FixedTurnRadiusMovementParams`, `SpeedTurnRadiusMovementParams`, `DragMovementParams`, `GliderMovementParams`. Each has `locomotion_orientation_constraint(loco, orient) -> TurnRateConstraint`.
   - `MovementParams` unified enum (line 177): Wraps all 5 types. Consider using this for a single generic consumer.
   - `Locomotion` / `Orientation` enums (lines 94-108): NOT ECS components — used as keys for constraint lookups. Map `LocomotionChannel` state → `Locomotion` enum, `OrientationChannel` state → `Orientation` enum.

4. **`artifacts/developer/src/game/units/types/state/behavior.rs`** — Channel definitions (read-only, already complete)
   - `LocomotionChannel` (line 58): `Moving(Vec<Vec3>)`, `Reversing(Vec<Vec3>)`, `Stopping`, `Stationary`.
   - `OrientationChannel` (line 73): `Turning(Vec3)`, `Maintaining`.
   - Note: `LocomotionChannel::Moving(path)` stores the full remaining path as `Vec<Vec3>`. The consumer must track progress through this path (current waypoint index).

### Key Patterns to Follow

**Existing turn_rate_movement_system pattern** (core.rs:899-1057):
- Get waypoint from path, compute angle, apply `params.turn_rate * delta`, compute speed with accel/decel, apply velocity in facing direction.
- Collision check: `occupancy.check_movement_collision(entity, proposed_pos.x, proposed_pos.z, half_w, half_h)` → if true, `velocity.0 = Vec3::ZERO` + insert `NeedsRepath`.
- Height lock: `transform.translation.y = if is_air_unit { 1.5 } else { 0.5 }`.
- `WAYPOINT_ARRIVAL_THRESHOLD = 0.5` constant for waypoint progression.

**Channel → Locomotion/Orientation mapping** for constraint lookups:
- `LocomotionChannel::Moving(_)` → `Locomotion::Moving`
- `LocomotionChannel::Reversing(_)` → `Locomotion::Reversing`
- `LocomotionChannel::Stopping` → `Locomotion::Stopping`
- `LocomotionChannel::Stationary` → `Locomotion::Stationary`
- `OrientationChannel::Turning(_)` → `Orientation::Turning`
- `OrientationChannel::Maintaining` → `Orientation::Maintaining`

**FixedTurnRadius constraint specifics** (movement.rs:199-220):
- `Stationary+Turning` and `Stopping+Turning` are **Invalid** — cannot turn while not moving.
- `Moving+Turning` and `Reversing+Turning` are `SpeedDependent` — max turn rate = `current_speed / minimum_turn_radius`.
- Use `max_turn_rate_at_speed(current_speed)` method.

**Path tracking gap**: `LocomotionChannel::Moving(Vec<Vec3>)` contains the full waypoint list but no current-index. The consumer system needs to either:
  (a) Track waypoint progress in `BaseBehaviorState` variants (which already have `planned_path` and `path_index` fields), or
  (b) Maintain a separate `LocomotionProgress` component, or
  (c) Pop consumed waypoints from the front of the channel path each tick (behaviors rewrite the full path each tick anyway).
  **Recommendation**: Option (c) is simplest since behaviors already rewrite channels every tick. The consumer reads `Moving(path)` and treats `path[0]` as the current target waypoint.

### Important: Dual Pipeline Coexistence

The **combat behavior systems** (`artifacts/developer/src/game/combat/systems/behaviors.rs`) still use the old `MoveTarget`/`Path` pattern heavily (attack_target_system, attack_move_system, patrol_system all insert `MoveTarget`/`Path` components). These are a **separate task** (action-channel-attack-integration). For this task:
- Do NOT remove `MoveTarget`/`Path` types from `movement.rs` — combat systems still use them.
- Keep the old `unit_movement_system` and `turn_rate_movement_system` alive for now, but make them only process entities that have `MoveTarget` but do NOT have `LocomotionChannel::Moving`/`Reversing` active. OR filter by `Without<LocomotionChannel>` if combat entities don't have channels.
- Alternatively, have the new channel consumer system take priority: if `LocomotionChannel` is not `Stationary`, the channel consumer drives movement; if `Stationary` and `MoveTarget` exists, fall through to old system.
- The `right_click_move_command` function (core.rs:179) also inserts `MoveTarget`/`Path` when dispatching Move/Patrol/AttackMove/Reverse commands (lines 442, 470, 496, 520, 544, 571). This is an existing dispatch path that will eventually need to write to channels instead, but is not in scope for this task.

### Unit Spawning

Units are spawned with channel components already in place:
- `core.rs:99-100`: `LocomotionChannel::default()` (`Stationary`) and `OrientationChannel::default()` (`Maintaining`) are inserted at spawn.
- All behavior-driven units (Agents, Guards, enemy test units) have both channels.
- Combat units spawned via `spawn_peacekeeper` (utils.rs) also get these channels.

### Design Spec Reference
- `artifacts/designer/design/control_system.md` lines 538-594: BaseActionChannels, LocomotionChannel, OrientationChannel, BaseAttackChannel definitions.
- Key spec constraints: BaseAiming/BaseFiring require Stationary locomotion. BaseReloading allows free Locomotion/Orientation. These attack-phase constraints are already checked via `attack_state.phase.base_action_constraints(is_turret_source)` in existing movement systems.

## Dependencies

- **action-channel-attack-integration** (sibling task): Handles combat behaviors migrating to channels. This locomotion/orientation task should be completed first since combat movement systems (`MoveTarget`/`Path`) must coexist with channel-driven movement until attack integration is done. Design the consumer systems to allow this coexistence.
- **Existing behavior systems** (already implemented in `behaviors.rs`): These write to `LocomotionChannel`/`OrientationChannel` each tick. The consumer systems being built here read those channels. No code changes needed in behaviors.rs for this task.
- **turret-autonomous-scanning-rework** and **turret-engagement-system** (sibling tasks): These are independent — turret systems use `TurretOrientationChannel`/`TurretAttackChannel`, not the base channels.
