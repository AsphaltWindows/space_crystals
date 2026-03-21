# shift-click-command-queuing

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-unit-command-system.md

## Task

Add shift-click command queuing to all command-issuing code paths so that holding Shift appends commands to the `CommandQueue` instead of replacing the current command.

### Context

The design spec says 'Commands can be queued by the player (e.g., shift-click)'. Currently, all command issuing paths (right-click in `right_click_move_command`, hold_position_system, stop_command_system, and `execute_command_action` in command_panel.rs) directly replace the unit's `UnitCommand` component. The `CommandQueue` component exists but nothing pushes to it.

### Requirements

1. **Detect Shift key** in all command-issuing code paths
2. **Queuing behavior**: When Shift is NOT held: clear `CommandQueue`, replace `UnitCommand` (current behavior). When Shift IS held: push the new command onto `CommandQueue` instead of replacing `UnitCommand`. Do NOT clear movement state or current command.
3. Add tests verifying queue behavior.

## Technical Context

### Files to Modify

1. **`artifacts/developer/src/game/units/systems/core.rs`** — `right_click_move_command` function (line 179)
   - Add `keyboard: Res<ButtonInput<KeyCode>>` to the system parameters (currently NOT present — the system only has `buttons: Res<ButtonInput<MouseButton>>`)
   - Every branch that issues a command needs the shift check. The command-issuing points are:
     - Line 276: `UnitCommand::AttackTarget(target_entity)` (entity-click attack)
     - Line 299: `UnitCommand::DropOffResources(target_entity)` (AwaitingTarget DropOff)
     - Line 325: `UnitCommand::PickUpSupplies(target_entity)` (Chopper SDS)
     - Line 342: `UnitCommand::AttachToTower(target_entity)` (Chopper tower)
     - Line 365: `UnitCommand::Gather(target_entity)` (Agent crystal)
     - Line 383: `UnitCommand::Gather(target_entity)` (Agent SDS supplies)
     - Line 403-405: `UnitCommand::DropOffResources` or `UnitCommand::Enter` (Agent Tunnel interaction)
     - Line 444: `UnitCommand::Move(target_pos)` (ground move)
     - Line 472: `UnitCommand::Patrol{...}` (patrol)
     - Line 498: `UnitCommand::AttackMove(target_pos)` (attack+ground → attack-move)
     - Line 522: `UnitCommand::AttackMove(target_pos)` (explicit attack-move)
     - Line 546: `UnitCommand::AttackLocation(target_pos)` (attack-ground)
     - Line 573: `UnitCommand::Reverse(target_pos)` (reverse)
   - **Pattern for each**: Check `keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight)`. If shift held, push to `CommandQueue` instead of inserting `UnitCommand`, and do NOT call `clear_movement_state_full`. If shift not held, keep current behavior AND add `command_queue.clear()`.
   - You'll need to add `&mut CommandQueue` (the game's custom type, NOT `bevy::ecs::world::CommandQueue`) to the `selected_units` query. Be careful: `bevy::ecs::world::CommandQueue` is used in test helpers for Bevy's deferred command application — these are completely different types.

2. **`artifacts/developer/src/game/units/systems/commands.rs`** — Two systems:
   - `hold_position_system` (line 65): Already has `keyboard: Res<ButtonInput<KeyCode>>`. Add `&mut CommandQueue` to the `selected_units` query. When shift held, push `UnitCommand::HoldPosition` to queue instead of inserting it and the `HoldingPosition` marker. When not shift, also clear the queue.
   - `stop_command_system` (line 99): Already has `keyboard: Res<ButtonInput<KeyCode>>`. Same pattern. When shift held, push `UnitCommand::Stop` to queue. When not shift, also clear the queue.

3. **`artifacts/developer/src/ui/command_panel.rs`** — Two paths call `execute_command_action`:
   - `handle_command_button_clicks` (line 799): Mouse click path. Does NOT have keyboard access. You need to add `keyboard: Res<ButtonInput<KeyCode>>` to this system and pass it through.
   - `command_panel_hotkeys` (line 828): Already has `keyboard: Res<ButtonInput<KeyCode>>`. Already computes `shift_held` (line 851). Pass it through to `execute_command_action`.
   - `execute_command_action` (line 1107): Add a `shift_held: bool` parameter and a `command_queues: &mut Query<&mut CommandQueue>` parameter (or fold into existing unit query). The two relevant branches are:
     - `CommandButtonAction::UnitHoldPosition` (line 1389): same shift logic
     - `CommandButtonAction::UnitStop` (line 1407): same shift logic
   - Other `CommandButtonAction` variants (Move, Attack, Patrol, Reverse, etc.) just set `ObjectInterfaceState::AwaitingTarget` — these don't issue commands directly, so no shift-queue needed there.

### Key Types

- **`CommandQueue`** — `artifacts/developer/src/game/units/types/state/commands.rs` line 150. Has `push(cmd)`, `pop_front()`, `clear()`, `is_empty()`, `len()`. Wraps `VecDeque<UnitCommand>`.
- **`UnitCommand`** — Same file. The enum of all commands (Move, Attack, HoldPosition, Stop, etc.).
- **`ButtonInput<KeyCode>`** — Bevy's keyboard state resource. Use `keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight)` for held-state detection.

### Helper Pattern

Consider extracting a helper function to reduce duplication:

```rust
/// Issues a command to a unit, either immediately or queued based on shift state.
/// When shift_held: pushes to CommandQueue, does not touch current command.
/// When !shift_held: clears CommandQueue, inserts UnitCommand (caller handles movement state).
fn issue_or_queue_command(
    entity_cmds: &mut EntityCommands,
    command_queue: &mut CommandQueue,
    command: UnitCommand,
    shift_held: bool,
) {
    if shift_held {
        command_queue.push(command);
    } else {
        command_queue.clear();
        entity_cmds.insert(command);
    }
}
```

Place this in `artifacts/developer/src/game/units/utils.rs` alongside `clear_movement_state` / `clear_movement_state_full`.

### Non-shift Path Change

When shift is NOT held (existing behavior), the code currently does NOT clear `CommandQueue`. You MUST add `command_queue.clear()` to every non-shift path so that a direct command wipes any previously queued commands.

### Testing

Add tests in `artifacts/developer/src/game/units/systems/core.rs` (test module starts at line 1375) and/or `commands.rs`:

1. **Non-shift replaces and clears queue**: Spawn unit with a pre-filled `CommandQueue`. Issue command without shift. Verify `UnitCommand` is replaced and queue is empty.
2. **Shift appends to queue**: Spawn unit with `UnitCommand::Idle`. Issue command with shift. Verify `UnitCommand` is still `Idle` and queue contains the new command.
3. **Multiple shift-clicks accumulate**: Issue 3 commands with shift. Verify queue has 3 entries in order.

Follow existing test patterns: use `World::new()` + `Commands` + manual `CommandQueue` application (see test helpers in `core.rs` line ~2063 and `utils.rs` line ~499).

### System Registration

No new systems needed — this modifies existing systems. Verify the modified system signatures still match their registrations in:
- `artifacts/developer/src/game/units/mod.rs` line ~60 (hold_position_system, stop_command_system)
- `artifacts/developer/src/ui/mod.rs` (handle_command_button_clicks, command_panel_hotkeys)

## Dependencies

- **CommandQueue component** (already exists at `commands.rs:150`): Must be spawned on all units. Verify it is — check `artifacts/developer/src/game/utils.rs` lines 471, 567, 664, 923 where `CommandQueue::new()` is spawned on units. This dependency is already satisfied.
- **Dequeue system** (from sibling task): A separate system must dequeue from `CommandQueue` when the current command completes. This task only handles the enqueue side. The dequeue system is part of the broader unit-command-system feature but is a separate task. This task can be implemented independently — queued commands will simply accumulate until the dequeue system is implemented.
