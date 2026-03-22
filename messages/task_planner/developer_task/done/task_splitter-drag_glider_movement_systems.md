# drag_glider_movement_systems

## Metadata
- **From**: task_splitter
- **To**: task_planner

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
- Higher speeds → wider turns; lower speeds → tighter circles
- Acceleration/deceleration from params
- Air domain — no ground collision, uses `SeparationRadius` for soft separation (handled by existing `air_unit_separation_system`)
- Path following: unlike other systems, the glider cannot stop at waypoints — it flies through them and curves toward the next one

### Registration

Add both systems to `UnitsPlugin` in Phase 3 (movement systems). Ensure `grid_position_sync_system` runs after them.
