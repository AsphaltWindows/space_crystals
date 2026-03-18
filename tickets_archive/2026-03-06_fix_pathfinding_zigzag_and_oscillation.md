# Ticket: Fix Pathfinding Zigzag Paths and Unit Oscillation

## Current State
The A* pathfinding implementation only considers 4 cardinal neighbors (N/S/E/W), producing zigzag/staircase paths on open terrain. The heuristic uses Manhattan distance and movement cost is hardcoded to 1.0. Additionally, units sometimes oscillate/fidget between two positions, never reaching their destination — likely caused by dual waypoint advancement (behavior layer and core movement layer both independently manage path index with the same 0.3 threshold) and/or the arrival threshold being too tight relative to unit speed and deceleration.

### Specific code locations (per task_planner analysis):
- `src/game/units/utils.rs:32` — `get_neighbors()` returns only 4 cardinal neighbors
- `src/game/units/utils.rs:28` — `heuristic()` uses Manhattan distance
- `src/game/units/pathfinding.rs:67` — movement cost hardcoded to 1.0
- `src/game/units/systems/core.rs:502` — `unit_movement_system` (non-TurnRate) uses WAYPOINT_REACHED_THRESHOLD = 0.3
- `src/game/units/systems/core.rs:614` — `turn_rate_movement_system` uses same 0.3 threshold
- `src/game/units/systems/behaviors.rs:32` — `moving_to_location_system` also advances waypoints independently

## Desired State

### 1. Diagonal pathfinding (8-directional A*)
- `get_neighbors()` returns 8 neighbors (4 cardinal + 4 diagonal)
- `heuristic()` uses octile distance: `max(|dx|, |dz|) + (sqrt(2) - 1) * min(|dx|, |dz|)`
- Diagonal movement cost uses `sqrt(2) ~= 1.414` instead of 1.0
- Diagonal moves check that both adjacent cardinal tiles are traversible (corner-cutting prevention)
- Existing `smooth_path()` continues to operate on the resulting paths

### 2. Oscillation fix
- Only one layer (either behavior or core movement) manages waypoint index advancement — eliminate the dual-advancement conflict
- Arrival threshold is appropriate for unit speed and deceleration (increase if needed, or scale proportionally to unit speed)
- Units reliably reach their destination and stop cleanly via the Stopping -> stopped -> complete transition

## Justification
Per `features/unit_system.md`, all 5 movement models operate in continuous space (SpaceUnits, degrees, acceleration/deceleration curves) — nothing restricts pathfinding to cardinal directions. Per `features/unit_commands_and_behaviors.md`, the MovingToLocation behavior specifies "pathfind to TargetLocation, execute plan of (Locomotion, Orientation) pairs" with "End: Stopping -> stopped -> complete" — paths should be smooth and units must reliably arrive and stop. Both issues are implementation bugs confirmed by full forum consensus in `forum/pathfinding_zigzag_and_oscillation.md`.

## QA Steps
1. Launch the game and spawn a Peacekeeper unit.
2. Right-click a distant open tile diagonally from the unit's position (e.g., upper-right or lower-left, across clear terrain with no obstacles).
3. Observe the path the unit takes — it should move in a smooth diagonal line, not a zigzag staircase pattern.
4. Right-click several more diagonal destinations in succession to confirm diagonal movement is consistent.
5. Right-click a tile that requires a mix of diagonal and cardinal movement (e.g., 5 tiles right and 3 tiles up). Confirm the path is smooth and direct, not purely cardinal steps.
6. Right-click a destination across the map. Let the unit travel the full distance. Confirm the unit arrives at the destination and stops cleanly without fidgeting or oscillating.
7. Rapidly issue multiple move commands to different locations. Confirm the unit transitions between paths without getting stuck or oscillating.
8. Move a unit to a location near obstacles/map edges. Confirm it arrives and stops cleanly.
9. Test with multiple units simultaneously moving to the same or nearby destinations. Confirm none exhibit oscillation.

## Expected Experience
- Units move in smooth, direct paths that include diagonal movement when appropriate — no zigzag staircase patterns on open terrain.
- Units decelerate and come to a complete stop at their destination without any visible fidgeting, jittering, or back-and-forth oscillation.
- Movement looks natural and fluid across all terrain types and distances.
