# action-channel-locomotion-orientation

## Metadata
- **From**: task_splitter
- **To**: task_planner

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

**Key files:**
- Channel definitions: `game/units/types/state/behavior.rs` (already complete)
- Current movement systems: `game/units/systems/core.rs` (~line 759+)
- Behavior systems that write channels: `game/units/systems/behaviors.rs`
- Movement params with constraints: `game/units/types/movement.rs`
- Plugin registration: `game/units/mod.rs`

**Design reference:** `artifacts/designer/design/control_system.md` — BaseActionChannels, LocomotionChannel, OrientationChannel sections.
