# command-to-state-mapping

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-unit-command-system.md

## Task

Implement the command-to-BaseCommandState mapping system and the command dequeue pipeline.

### Context

`BaseCommandState` (in `src/game/units/types/state/commands.rs`) exists with fields `command_type: CommandType`, `target_location: Option<Vec3>`, `target_entity: Option<Entity>` but is never populated from `UnitCommand`. The `CommandQueue` component also exists but nothing dequeues from it.

### Requirements

1. **UnitCommand → BaseCommandState mapping system**: Create a system (e.g., `command_state_sync_system`) that runs each tick and updates `BaseCommandState` from the current `UnitCommand` component on each unit:
   - `Move(pos)` → CommandType::Move, target_location=Some(pos), target_entity=None
   - `Move` to object → CommandType::Move, target_location=None, target_entity=Some(entity) (currently Move only takes Vec3; if needed, this maps to Move with location)
   - `AttackTarget(entity)` → CommandType::Attack, target_location=None, target_entity=Some(entity)
   - `AttackLocation(pos)` → CommandType::AttackGround, target_location=Some(pos), target_entity=None
   - `AttackMove(pos)` → CommandType::AttackMove, target_location=Some(pos), target_entity=None
   - `Patrol{..}` → CommandType::Patrol, target_location=Some(end), target_entity=None
   - `HoldPosition` → CommandType::HoldPosition, target_location=None, target_entity=None
   - `Stop` → CommandType::Stop, target_location=None, target_entity=None
   - `Reverse(pos)` → CommandType::Reverse, target_location=Some(pos), target_entity=None
   - `Enter(entity)` → CommandType::Enter, target_location=None, target_entity=Some(entity)
   - `Idle` → CommandType::Default, target_location=None, target_entity=None

2. **Command dequeue system**: Create a system that detects when the current command's behavior has completed (unit is Idle with a non-empty CommandQueue) and dequeues the next command:
   - Pop front from `CommandQueue`
   - Set the popped command as the unit's `UnitCommand` component
   - The mapping system (above) will then update `BaseCommandState` accordingly
   - When queue is empty and behavior completes, set `UnitCommand::Idle`

3. Add tests verifying:
   - Each UnitCommand variant maps to the correct BaseCommandState fields
   - Dequeue pops commands in FIFO order
   - Empty queue after completion results in Idle state

### Design reference
See `artifacts/designer/design/control_system.md` under 'Unit Command' and 'BaseCommandState' sections.
