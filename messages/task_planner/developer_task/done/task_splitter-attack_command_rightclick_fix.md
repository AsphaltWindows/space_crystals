# attack_command_rightclick_fix

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-unit_command_system_r1.md

## Task

Fix right-click attack command so units pursue and attack enemy targets instead of stopping in place.

**Bug**: When a unit is selected and the player right-clicks an enemy unit or building, the unit stops moving instead of pursuing and attacking. The UnitCommand::AttackTarget is being issued (core.rs ~line 351) but the unit does not move toward the target.

**Investigation areas**:
- Check system ordering: `attacking_object_behavior_system` (combat/systems/behaviors.rs line 22) must run after the UnitCommand component is inserted. The system reads `UnitCommand::AttackTarget(target)` and sets MoveTarget+Path for approach when out of range (line 96-118). Verify this system is actually matching entities — check that all required query components (AttackCapability, AttackState, UnitCommand, UnitBaseEnum, Owner, GridPosition) are present on spawned units.
- Check if `clear_movement_state_full` (called at core.rs line 349 before insert) is removing something that prevents the behavior system from working on the same or next tick.
- Check if there is a conflicting system (e.g., `behavior_completion_system`, `stopping_behavior_system`) that resets AttackTarget to Idle before the attack behavior system runs.
- Check if the `command_dequeue_system` (commands.rs line 228) is prematurely dequeuing when the command was inserted directly (not queued).
- Test with a Peacekeeper right-clicking an enemy unit — verify the unit moves toward and attacks it.
- Test with a unit right-clicking an enemy building — verify same behavior.
