# command_context_and_issue_helper

## Metadata
- **From**: task_planner
- **To**: developer

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

All existing tests must continue to pass. This is a pure refactor -- no behavioral changes.

## Technical Context

### Primary file to modify

**`artifacts/developer/src/game/units/systems/core.rs`** -- this is where `right_click_move_command` lives (line 243, ~765 lines long through line 1008). The function has 12 parameters and contains the entire entity-click + ground-click dispatch logic.

### The duplicated pattern to extract

The following pattern appears 10+ times in the entity-click section (lines 339-750):

```rust
// 1. Check interruptibility
if let Some(attack_state) = attack_state_opt {
    if !attack_state.phase.is_interruptible() { continue; }
}
// 2. Get entity commands
let mut entity_cmds = commands_ecs.entity(entity);
// 3. Clear movement state (if not shift)
if !shift_held {
    clear_movement_state_full(&mut entity_cmds);
    entity_cmds.insert(AttackState::default());  // most handlers do this, Attack handler does NOT
}
// 4. Issue or queue
issue_or_queue_command(&mut entity_cmds, &mut command_queue, command, shift_held);
```

Specific call sites in entity-click section (all line numbers approximate):
1. **Attack** (line 343-376): Does NOT insert `AttackState::default()` on clear -- only calls `clear_movement_state_full`. Also adds pathfinding after. The helper's default behavior (inserting AttackState::default()) needs a variation here -- either omit AttackState from the helper and let callers add it, OR add a parameter/separate helper for the attack case.
2. **DropOff** (line 389-402): Standard pattern with AttackState::default().
3. **Enter (AwaitingTarget)** (line 419-430): Does NOT insert AttackState::default() -- only `clear_movement_state_full`.
4. **PickUpSupplies** (line 446-461): Standard + adds `PickingUpSuppliesBehavior` after.
5. **AttachToTower** (line 478-493): Standard + adds `AttachingToTowerBehavior` after.
6. **DropOffSupplies** (line 510-525): Standard + adds `DroppingOffSuppliesBehavior` after.
7. **Chopper right-click SDS** (line 544-559): Standard + adds `PickingUpSuppliesBehavior` after.
8. **Chopper right-click SupplyTower** (line 567-598): Standard + adds behavior based on command type (DropOff or AttachToTower).
9. **Agent Gather crystal** (line 612-624): Standard pattern.
10. **Agent Gather SDS** (line 633-645): Standard pattern.
11. **Agent Tunnel interaction** (line 654-678): Standard pattern.
12. **Recruit Enter Armory** (line 697-706): Does NOT insert AttackState::default() -- only `clear_movement_state_full`.
13. **Guard Enter Tunnel** (line 724-738): Does NOT insert AttackState::default() -- only `clear_movement_state_full`.

**Variation analysis**: About 8 of 13 call sites insert `AttackState::default()` on clear. 5 do not (Attack, Enter-AwaitingTarget, Recruit-Enter-Armory, Guard-Enter-Tunnel, and the Attack handler). Recommendation: Do NOT include `AttackState::default()` in the helper. Instead, have the helper handle steps 1-3 (interruptibility check + clear_movement_state_full) and let callers insert `AttackState::default()` separately when needed. This avoids the need for a parameter flag.

Alternative: include AttackState::default() in the helper as the majority pattern, and have the 5 variant sites either re-insert an in-progress AttackState or use a separate helper. The task description suggests the helper DOES insert AttackState::default(), so follow that and handle the Attack/Enter variants by not using the helper for the clear step (only for interruptibility + issue).

### Key types and imports needed

- **`AttackState`**: from `crate::game::combat::types` (already imported at line 5)
- **`AttackPhase`**: `AttackState.phase` field, `is_interruptible()` method (types.rs line 114)
- **`CommandQueue`**: from `crate::game::units::types::state::commands` (the game's CommandQueue, NOT bevy's)
- **`UnitCommand`**: from `crate::game::units::types` (already imported via wildcard at line 11)
- **`CommandType`**: from `crate::game::units::types::state::commands` (already imported via wildcard)
- **`CursorTarget`**: from `crate::ui::types` (already imported at line 10)
- **`clear_movement_state_full`**: from `crate::game::units::utils` (already imported at line 12)
- **`issue_or_queue_command`**: from `crate::game::units::utils` (already imported at line 12)
- **`LocalPlayer`**: from `crate::types` (already imported via wildcard at line 2)

### Ground-click section (lines 764-1008)

The ground-click handlers (Move, Patrol, AttackMove, AttackGround, Reverse) use a DIFFERENT pattern: they also compute paths and insert MoveTarget+Path components. The `try_issue_command` helper can still be used for the interruptibility check in these handlers, but the pathfinding + component insertion remains inline. Specifically:
- Lines 766-793 (Move/Default): interruptibility check + path computation + inline insert of MoveTarget/Path/cmd/AttackState
- Lines 799-825 (Patrol): same pattern
- Lines 832-856 (Attack->AttackMove): same pattern
- Lines 862-886 (AttackMove): same pattern
- Lines 892-916 (AttackGround): same pattern
- Lines 922-953 (Reverse): same pattern with can_reverse check

For ground-click, the helper can validate interruptibility and return early (the `continue` in the loop). The actual command issuance uses a different flow (push to queue if shift, or inline insert path+cmd if not shift).

### CommandContext construction point

Build CommandContext at line ~322 (after all early returns, after `shift_held` is computed). The struct needs:
- `shift_held`: already computed at line 322
- `local_player_number`: `local_player.0` (u32)
- `is_right_click`: already computed at line 257
- `is_left_click`: already computed at line 258
- `in_command_mode`: already computed at line 259
- `command_type`: already computed at line 305
- `cursor_target`: `&*cursor_target` (deref Res to get &CursorTarget)

Note: `local_player` is `Res<LocalPlayer>` where `LocalPlayer` wraps a `u32` (field `.0`). Ownership check at line 318 uses `owner.player_number() != Some(local_player.0)`. The `Owner::player_number()` returns `Option<u8>`, so there may be a u32/u8 type mismatch -- check actual types before deciding the field type.

### Existing test patterns (line 2976+)

Tests in core.rs use `World::new()` + `run_system_once()`. For the `try_issue_command` helper (a plain function, not a system), tests can call it directly:
- Spawn an entity with `AttackState`, `CommandQueue`, and `UnitCommand::Idle`
- Call `try_issue_command` with `Commands` from an `App` or use `World::run_system_once` with a closure-based system that calls the helper
- Assert on return value + check entity components after

Since `try_issue_command` takes `&mut Commands`, the test needs a Bevy `App` or `World` to get a `Commands` instance. The cleanest pattern: write a one-off system closure that calls `try_issue_command` and stores the result in a resource:

```rust
#[derive(Resource)]
struct TestResult(bool);

app.world_mut().run_system_once(|mut commands: Commands, mut query: Query<...>, mut result: ResMut<TestResult>| {
    // call try_issue_command, store in result
}).unwrap();
```

Alternatively, use `World::commands()` approach if available in Bevy 0.17.

### Existing helper functions in utils.rs

- `clear_movement_state_full` (utils.rs:141): removes MoveTarget, Path, HoldingPosition, IdleOrigin; inserts Velocity(ZERO)
- `issue_or_queue_command` (utils.rs:153): if shift, pushes to CommandQueue; else clears queue and inserts UnitCommand

### Registration / system ordering

No new systems are being added -- this is a pure refactor of existing code. `right_click_move_command` is registered in `UnitsPlugin` (artifacts/developer/src/game/units/mod.rs). No ordering changes needed.

## Dependencies

None. This is a standalone refactor task that modifies only `right_click_move_command` in core.rs. All existing imports and helper functions already exist. No other planned_tasks need to land first.
