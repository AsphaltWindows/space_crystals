# shift_click_queue_fix

## Metadata
- **From**: task_splitter
- **To**: task_planner

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
