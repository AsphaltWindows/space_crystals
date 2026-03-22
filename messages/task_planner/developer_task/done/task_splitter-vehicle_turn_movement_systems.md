# vehicle_turn_movement_systems

## Metadata
- **From**: task_splitter
- **To**: task_planner

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
