# command-to-state-mapping

## Metadata
- **From**: task_planner
- **To**: developer

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

## Technical Context

### Files to Modify

1. **`artifacts/developer/src/game/units/types/state/commands.rs`** — Add missing `CommandType` variants:
   - `CommandType` enum (line 76) is missing `HoldPosition` and `Stop` variants. Add them, plus update `name()` and `hotkey()` match arms. For HoldPosition use name "Hold Position" / hotkey "E", for Stop use name "Stop" / hotkey "X" (matching the grid slot assignments in control_system.md).
   - Also missing: `PickUpSupplies` and `AttachToTower` variants for chopper commands. These can be deferred or mapped to `Default` for now since they are chopper-specific and not in the design doc mapping. Document the decision.

2. **`artifacts/developer/src/game/units/systems/commands.rs`** — Add two new systems:
   - `command_state_sync_system`: Query `(&UnitCommand, &mut BaseCommandState)` on all units. Match each `UnitCommand` variant and set the corresponding `BaseCommandState` fields. Handle ALL variants including `Gather(entity)` → `CommandType::Gather`, `DropOffResources(entity)` → `CommandType::DropOff`, `Build{target,..}` → `CommandType::Build`, `BuildTunnel(pos)` → `CommandType::BuildTunnel`, `PickUpSupplies(entity)`/`AttachToTower(entity)` → `CommandType::Default` (or new variants).
   - `command_dequeue_system`: Query `(&UnitCommand, &mut CommandQueue)` — when `UnitCommand::Idle` and queue is non-empty, pop front and replace `UnitCommand`. Use `Commands` to insert the new `UnitCommand` component.

3. **`artifacts/developer/src/game/units/mod.rs`** — Register both new systems in `CommandsPlugin::build()` (line 57-64). System ordering:
   - `command_dequeue_system` should run BEFORE `command_state_sync_system` so dequeued commands get their state mapped in the same tick.
   - Both should run BEFORE behavior systems (currently in `UnitsPlugin` under `DiagCategory::Movement`). Since `CommandsPlugin` uses `DiagCategory::Commands`, ensure ordering between sets if needed, or add explicit `.before()` constraints.

### Existing Patterns to Follow

- **System signature pattern**: See `hold_position_system` (commands.rs line 65) and `stop_command_system` (commands.rs line 99) for the standard system signature pattern with `Commands`, `Query`, etc.
- **Component insertion pattern**: `commands.entity(entity).insert(UnitCommand::HoldPosition)` — use Bevy `Commands` for mutation.
- **Test pattern**: The existing tests in commands.rs (lines 209-656) use `CommandQueue` unit tests. For system tests, see the test module in `behaviors.rs` (line 1379+) which uses `App::new()` with `MinimalPlugins`.

### Key Types and Components

- `UnitCommand` (Component, enum) — current command on unit, defined at commands.rs:8
- `CommandQueue` (Component) — VecDeque-backed FIFO queue, commands.rs:149, methods: `push()`, `pop_front()`, `is_empty()`, `clear()`
- `BaseCommandState` (Component) — command_type + target_location + target_entity, commands.rs:189
- `CommandType` (enum, not Component) — discriminant for command type, commands.rs:76
- `BaseBehaviorState` (Component) — behavior state, behavior.rs:8. `BaseBehaviorState::None` = idle behavior
- `LocomotionChannel` (Component) — `Stationary` = not moving, behavior.rs:59

### Behavior Completion Detection

The dequeue system needs to detect when a command's behavior has completed. Currently:
- Behavior systems set `UnitCommand::Idle` when they finish (e.g., `building_behavior_system` at behaviors.rs:512, `gathering_resource_behavior_system` at behaviors.rs:652)
- Some behaviors DON'T set Idle directly — e.g., `moving_to_location_system` (behaviors.rs:128-131) explicitly notes: "setting UnitCommand::Idle must be done by a separate completion system"
- The dequeue system should check: `UnitCommand::Idle` = behavior done, dequeue next. This is the simplest and most correct approach since many behavior systems already set Idle on completion.
- For move completion specifically: check `UnitCommand::Move(_)` + `LocomotionChannel::Stationary` + `BaseBehaviorState::None` as a "move done" signal, then set `UnitCommand::Idle`. OR: add a simple `move_completion_system` that does this transition. The latter is cleaner and separates concerns.

### Bevy ECS Considerations

- Both new systems query `UnitCommand` (which is a Component). Use `&mut UnitCommand` in dequeue system since it replaces the command.
- `command_state_sync_system` needs `&UnitCommand` (read) and `&mut BaseCommandState` (write).
- System ordering within `DiagCategory::Commands`: `command_dequeue_system.before(command_state_sync_system)`.
- The dequeue system must not run during the same tick a behavior system sets Idle — it should wait one tick. This naturally happens if dequeue runs in Commands set (before Movement set behaviors).

### Unit Spawn Pattern

All units already spawn with `CommandQueue::new()` and `BaseCommandState::default()` (see `artifacts/developer/src/game/utils.rs` lines 471-472). No spawn changes needed.

## Dependencies

- **None for core implementation** — `UnitCommand`, `CommandQueue`, `BaseCommandState`, and `CommandType` already exist. This task adds the runtime plumbing that connects them.
- **Sibling task `shift-click-command-queuing`** depends on THIS task — shift-click pushes to `CommandQueue`, and this task's dequeue system is what reads from it. This task should be completed first.
- **`base-behaviors-verify` task** (from a different feature) may identify gaps in behavior completion signals that affect the dequeue system. No hard dependency, but the developer should be aware that not all behaviors cleanly set `UnitCommand::Idle` on completion yet.
