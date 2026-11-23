# shift_click_queue_fix

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-unit_command_system_r1.md

## Task

Fix shift-click command queuing so that shift-right-clicking multiple ground locations queues all commands and the unit visits them sequentially.

**Bug**: When shift-right-clicking multiple locations, the unit only moves to the first location and ignores the rest. Expected: each shift-click appends to the CommandQueue and the unit processes them in order.

**Investigation areas**:
- The `issue_or_queue_command` function (units/utils.rs line 153) correctly pushes to CommandQueue when shift_held and inserts directly when not. Verify the ground-click Move handler in `right_click_move_command` (core.rs) actually passes the shift_held flag and uses issue_or_queue_command (not a direct insert).
- Check the ground-click handler section (core.rs around line 600+) — look for the Move command path when command_type is Default and cursor is ground. Ensure it calls `issue_or_queue_command` with shift_held rather than always inserting UnitCommand::Move directly.
- Check `command_dequeue_system` (commands.rs line 228): it dequeues when current command is UnitCommand::Idle. Verify that when a Move command completes (unit arrives at destination), the UnitCommand gets reset to Idle so the next queued command can be picked up.
- Check `behavior_completion_system` (units/systems/behaviors.rs line 378) — when a MovingToLocation behavior completes (unit reaches target), does it set UnitCommand back to Idle?
- Test: select a unit, shift-right-click 3 ground locations, verify unit visits all 3 sequentially.

## Technical Context

### Root Cause Analysis

The shift-click queuing mechanism works correctly at the queue level — commands are pushed and dequeued properly. The bug is that **dequeued commands lack the side-effect setup that direct commands receive**.

When a non-shift Move is issued in `right_click_move_command` (core.rs lines 748-768), the code performs inline setup:
1. Calls `find_path_for_domain()` to compute a path from current position to target
2. Calls `smooth_path()` on the result
3. Inserts `MoveTarget(target_pos)`, `Path { waypoints, current_waypoint: 0 }`, the `UnitCommand::Move(target)`, and `AttackState::default()` all as a bundle

When shift is held (lines 749-750), only `command_queue.push(cmd)` is called — no pathfinding, no MoveTarget, no Path, no BaseBehaviorState initialization.

When `command_dequeue_system` (commands.rs line 228) later pops the command and sets `UnitCommand::Move(target)`, the behavior system `moving_to_location_system` (behaviors.rs line 52) tries to read the path from `BaseBehaviorState`, but it is still `BaseBehaviorState::None` (set when the previous command completed via `behavior_completion_system` at line 406). The system skips entities with `BaseBehaviorState::None` (line 92), so the unit never moves.

This same gap affects ALL ground-click queued commands: Move, Patrol, AttackMove, AttackGround, and Reverse — each follows the same pattern of inline setup on direct insert but queue-only on shift.

### Files That Need to Change

1. **`artifacts/developer/src/game/units/systems/commands.rs`** — Add a new `move_command_dispatch_system` (or similar name) that detects when a `UnitCommand::Move(target)` (and Patrol, AttackMove, AttackGround, Reverse) is set on an entity that has `BaseBehaviorState::None` (meaning no behavior is initialized yet) and performs the necessary setup:
   - Compute path via `find_path_for_domain()` using the unit's current `Transform`, `UnitBaseEnum`, tile data, grid, and occupancy
   - Smooth the path via `smooth_path()`
   - Initialize `BaseBehaviorState` with the appropriate variant based on the unit's movement params (TurnRateMovementParams -> TurnRate, FixedTurnRadiusMovementParams -> FixedTurnRadius, etc.)
   - Remove `HoldingPosition` and `IdleOrigin` markers
   - Optionally insert `MoveTarget`/`Path` for backward compat with old movement pipeline (only needed if unit uses old pipeline)

   Alternatively, this could be placed in `artifacts/developer/src/game/units/systems/behaviors.rs` alongside other dispatch systems like `enter_command_dispatch_system` (line 466).

2. **`artifacts/developer/src/game/units/mod.rs`** — Register the new dispatch system in Phase 2 (behavior systems), BEFORE `moving_to_location_system` and after `command_dequeue_system`. The system must run after dequeue so it sees the newly-set UnitCommand. System ordering: `command_dequeue_system` (in CommandsPlugin/DiagCategory::Commands) -> `move_command_dispatch_system` (in UnitsPlugin/DiagCategory::Movement, Phase 2) -> `moving_to_location_system`.

3. **`artifacts/developer/src/game/units/systems/core.rs`** — The inline setup code in `right_click_move_command` (lines 748-768 for Move, 774-800 for Patrol, 806-831 for AttackMove, etc.) can be simplified to always use `issue_or_queue_command` and let the dispatch system handle setup, OR the inline path can remain as an optimization (immediate pathfinding vs next-tick pathfinding). If keeping both paths, ensure they produce identical results.

### Key Types and Functions

- `CommandQueue` (commands.rs line 150): `VecDeque<UnitCommand>` wrapper with push/pop_front/clear
- `command_dequeue_system` (commands.rs line 228): pops front of queue when `UnitCommand::Idle`
- `behavior_completion_system` (behaviors.rs line 378): sets `UnitCommand::Idle` + `BaseBehaviorState::None` when locomotion is Stationary and velocity near zero — this correctly enables dequeue
- `BaseBehaviorState` variants: `TurnRate { planned_path, path_index }`, `FixedTurnRadius { planned_path, path_index }`, `SpeedTurnRadius { planned_path, path_index }`, `Drag { planned_path, path_index }`, `Glider { planned_path, path_index, circling }`, `None`
- Movement param components (determine which BaseBehaviorState variant to use): `TurnRateMovementParams`, `FixedTurnRadiusMovementParams`, `SpeedTurnRadiusMovementParams`, `DragMovementParams`, `GliderMovementParams`
- `find_path_for_domain(start_grid, target_grid, &tiles, &unit_base, width, height, &occupancy, exclusion)` — pathfinding function in `game/units/pathfinding`
- `smooth_path(path: Vec<Vec3>) -> Vec<Vec3>` — in `game/units/utils.rs`
- `world_to_grid(pos: Vec3) -> GridPosition` — in `game/units/utils.rs`

### Pattern to Follow

The `enter_command_dispatch_system` (behaviors.rs line 466) is the closest existing pattern:
- It queries for entities with a specific UnitCommand variant
- It validates preconditions
- It inserts behavior marker components
- It is registered in Phase 2 of UnitsPlugin with ordering constraints

The new dispatch system should follow this pattern but handle multiple command variants (Move, Patrol, AttackMove, AttackGround, Reverse) and perform pathfinding + BaseBehaviorState initialization.

### Bevy ECS Considerations

- The dispatch system needs read access to `Res<GridMap>`, `Res<OccupancyMap>`, tile queries, and the unit's `Transform`/`UnitBaseEnum`
- It needs write access to `BaseBehaviorState`, and optionally `Commands` for inserting/removing MoveTarget/Path/HoldingPosition/IdleOrigin
- System ordering is critical: must run AFTER `command_dequeue_system` so dequeued commands are visible, and BEFORE behavior systems that read `BaseBehaviorState`
- Since `command_dequeue_system` is in `DiagCategory::Commands` and behavior systems are in `DiagCategory::Movement`, there is no automatic ordering between them. Use explicit `.after()` or cross-set ordering.
- The dispatch system should also handle direct-insert commands that bypass the queue (when `issue_or_queue_command` inserts directly with shift not held) — currently the direct path in `right_click_move_command` handles setup inline, but if the dispatch system is added, the inline setup could be removed for consistency. However, the inline path currently also inserts MoveTarget/Path for the old movement pipeline, so be careful about backward compatibility.

### Test Strategy

- Unit test: spawn a unit with `UnitCommand::Idle`, push 3 `UnitCommand::Move(pos)` to CommandQueue, run dequeue + dispatch systems, verify BaseBehaviorState is initialized with a path
- Unit test: run behavior_completion_system to set Idle after first move completes, then dequeue + dispatch again, verify second move path is set up
- Integration test: shift-right-click 3 locations, verify all 3 are visited sequentially

## Dependencies

- **behavior_completion_system** (already exists, behaviors.rs line 378): Must correctly set `UnitCommand::Idle` when Move completes. Investigation confirms it does — it checks for `LocomotionChannel::Stationary` + near-zero velocity and sets `UnitCommand::Idle` + `BaseBehaviorState::None`.
- **command_dequeue_system** (already exists, commands.rs line 228): Must correctly pop the next command when current is Idle. Investigation confirms it does.
- **Pathfinding module** (`game/units/pathfinding`): The dispatch system needs access to `find_path_for_domain`. Already public.
- No dependency on other planned_tasks — this is a standalone bug fix in the command dispatch pipeline.
