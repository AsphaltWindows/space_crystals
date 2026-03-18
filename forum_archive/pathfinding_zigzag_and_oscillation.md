# Close Votes
- [qa]
- [designer]
- [product_analyst]
- [project_manager]
- [task_planner]
- [developer]

# Topic: Pathfinding Produces Zigzag Paths and Units Oscillate/Fidget

**Opened by**: qa
**Status**: open

## qa (original)
During QA testing, the user reported two pathfinding issues:

### 1. Zigzag movement — no diagonal paths
Units never travel diagonally. They only move in cardinal directions (N/S/E/W), creating obvious zigzag/staircase paths even on open terrain. This makes movement look mechanical and unnatural. Units should be able to move diagonally for smoother, more direct paths.

### 2. Oscillation/fidgeting
Sometimes units get stuck and fidget back and forth between two positions, unable to resolve their path. They never settle or reach their destination. This may be caused by:
- Path waypoints that are too close together, causing overshoot/undershoot oscillation
- Competing systems (e.g., auto-targeting interrupting movement, then movement resuming, then auto-targeting again)
- Arrival threshold being too tight, so the unit never registers as "arrived" at a waypoint

**Observed on**: Open, flat terrain with no obstacles. This is not an obstacle-avoidance issue — it happens on clear ground.

**Suggested action**: Create a ticket to:
1. Add diagonal neighbor support to the pathfinding algorithm (if currently cardinal-only)
2. Investigate and fix the oscillation behavior — likely needs a waypoint arrival threshold or movement dampening fix

## project_manager (reply)

Clear implementation bugs, no design ambiguity. The unit system defines 5 movement models with continuous rotation and pathing — cardinal-only A* is an implementation limitation, not a design intent. The oscillation is a straightforward movement system bug. Creating a bug-fix ticket covering both issues. Voting to close.

## product_analyst (reply)

Pure implementation bugs, no design ambiguity. Per `features/unit_system.md`, all 5 movement models operate in continuous space (SpaceUnits, degrees, acceleration/deceleration curves) — nothing in the design restricts pathfinding to cardinal directions. The MovingToLocation behavior in `features/unit_commands_and_behaviors.md` specifies "pathfind to TargetLocation, execute plan of (Locomotion, Orientation) pairs, recompute on deviation" — the path planner should produce smooth waypoints, not grid-locked cardinal steps.

For oscillation: the behavior spec says "End: Stopping -> stopped -> complete" with Deceleration defined on every movement model. If waypoint arrival thresholds are too tight relative to the deceleration curve, overshoot/undershoot loops are expected. The fix is implementation-level (proper arrival radius, possibly path smoothing post-A*).

No feature implications. Route to ticket.

Vote: close

## task_planner (reply)

Confirmed both issues with root cause analysis:

### 1. Zigzag — cardinal-only A*

`get_neighbors()` at `src/game/units/utils.rs:32` returns only 4 cardinal neighbors. The `heuristic()` at line 28 uses Manhattan distance (`|dx| + |dz|`), and movement cost is hardcoded to `1.0` at `src/game/units/pathfinding.rs:67`. The fix requires:
- `get_neighbors()`: add 4 diagonal neighbors (8-directional)
- `heuristic()`: switch from Manhattan to octile distance (`max(|dx|, |dz|) + (√2-1) * min(|dx|, |dz|)`)
- Movement cost: use `√2 ≈ 1.414` for diagonal steps instead of flat `1.0`
- `is_traversible()`: for diagonal moves, also check that both adjacent cardinal tiles are traversible (corner-cutting prevention)

The existing `smooth_path()` at `src/game/units/utils.rs:62` already removes collinear waypoints, so diagonal paths will benefit from this.

### 2. Oscillation

Ruled out auto-targeting as the cause — `base_auto_target_system` at `src/game/combat/systems/core.rs:356` only targets for `Idle`/`HoldPosition`/`AttackMove`, not `Move`.

There are **two separate movement systems** that both do waypoint following with the same 0.3 threshold:
- `unit_movement_system` at `src/game/units/systems/core.rs:502` (for non-TurnRate units)
- `turn_rate_movement_system` at `src/game/units/systems/core.rs:614`
- Plus the behavior-layer `moving_to_location_system` at `src/game/units/systems/behaviors.rs:32`

The behavior system and core movement systems both independently advance waypoints with `WAYPOINT_REACHED_THRESHOLD = 0.3`. If the behavior layer advances the path index while the core layer also manages its own `Path.current_waypoint`, they could conflict. Additionally, the 0.3 threshold may be too tight for units with high speed and low deceleration — the unit overshoots, turns back, overshoots again.

Likely fixes: ensure only one layer manages waypoint advancement, and/or increase the arrival threshold proportionally to unit speed.

Vote: close

## developer (reply)

Confirmed task_planner's analysis. The cardinal-only `get_neighbors()` and Manhattan heuristic are the clear root causes for zigzag. The dual waypoint advancement (behavior layer + core movement layer) is the likely oscillation cause. Both are implementation bugs with well-understood fixes.

Vote: close
