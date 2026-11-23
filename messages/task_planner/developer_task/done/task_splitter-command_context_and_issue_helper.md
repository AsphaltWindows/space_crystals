# command_context_and_issue_helper

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-right_click_command_dispatch.md

## Task

Extract a `CommandContext` struct and a `try_issue_command` helper function from the monolithic `right_click_move_command` system in `src/game/units/systems/core.rs`. This is the foundation work for a larger dispatch refactor.

### CommandContext struct

Create a `CommandContext` struct (in `src/game/units/systems/core.rs` or a new submodule) that bundles the shared read-only parameters used throughout `right_click_move_command`:

```rust
struct CommandContext<'a> {
    shift_held: bool,
    local_player_number: u32,
    is_right_click: bool,
    is_left_click: bool,
    in_command_mode: bool,
    command_type: CommandType,
    cursor_target: &'a CursorTarget,
}
```

Construct this struct at the top of `right_click_move_command` (after early returns) and pass it to the entity-click and ground-click sections. This replaces passing `shift_held`, `local_player.0`, `command_type`, `is_right_click`, etc. as separate locals.

### try_issue_command helper

Extract the repeated check-interruptible / clear-state / issue-command pattern (appears 10+ times in the function) into a helper:

```rust
fn try_issue_command(
    commands_ecs: &mut Commands,
    entity: Entity,
    attack_state_opt: Option<&AttackState>,
    command_queue: &mut CommandQueue,
    command: UnitCommand,
    shift_held: bool,
) -> bool {
    if let Some(attack_state) = attack_state_opt {
        if !attack_state.phase.is_interruptible() { return false; }
    }
    let mut entity_cmds = commands_ecs.entity(entity);
    if !shift_held {
        clear_movement_state_full(&mut entity_cmds);
        entity_cmds.insert(AttackState::default());
    }
    issue_or_queue_command(&mut entity_cmds, command_queue, command, shift_held);
    true
}
```

Replace ALL instances of the duplicated pattern in `right_click_move_command` with calls to `try_issue_command`. Some call sites have slight variations (e.g., the Attack entity handler does NOT insert AttackState::default() on clear, some handlers insert additional behavior components after issuing). For those, the helper should still handle the common part, and the caller adds the extra insert after.

**Important notes:**
- The helper returns `bool` (true = command issued, false = not interruptible) so callers can skip further per-entity work.
- Some handlers (chopper PickUpSupplies, DropOffSupplies, AttachToTower) insert additional behavior components (PickingUpSuppliesBehavior, DroppingOffSuppliesBehavior, AttachingToTowerBehavior) after issuing. The helper should return the EntityCommands or the caller should get them separately for the extra insert.
- The Move/Patrol/Attack ground handlers have a different pattern (they also pathfind and insert MoveTarget+Path). The helper covers the interruptibility check + clear + issue part; pathfinding remains in the caller.

### Testing

Write unit tests for the `try_issue_command` helper:
- `test_try_issue_command_interruptible`: Unit with interruptible attack state -> command issued, returns true
- `test_try_issue_command_not_interruptible`: Unit with non-interruptible attack state -> command NOT issued, returns false
- `test_try_issue_command_shift_queue`: shift_held=true -> command queued, movement state NOT cleared
- `test_try_issue_command_no_shift`: shift_held=false -> movement state cleared, attack state reset

All existing tests must continue to pass. This is a pure refactor — no behavioral changes.
