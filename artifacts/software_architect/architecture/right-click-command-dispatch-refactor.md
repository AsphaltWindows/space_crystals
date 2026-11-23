# Right-Click Command Dispatch Refactor

## Date: 2026-03-24
## Status: Architecture request sent
## Priority: 1 (highest impact)
## Parent: codebase-refactoring-priorities.md

## Problem Statement

`right_click_move_command` in `src/game/units/systems/core.rs` (lines 243-830+) is a single monolithic Bevy system function that handles ALL right-click and command-mode-left-click interactions for every unit type in the game. It has grown to ~600 lines of logic and exhibits several structural problems that make it the single largest source of developer pain in the codebase.

### Evidence

- **Developer**: Reports this function as a top pain point; every combat/command task requires modifying it.
- **Task Planner**: Confirms nearly every combat or command task touches this function, creating cross-cutting dependencies that slow planning.
- **Task Splitter**: Confirms difficulty splitting tasks due to unclear module boundaries in this function.
- **Forum topic**: `2026-03-23T12-10-00Z-operator-codebase-refactoring-review` (closed, all parties agreed on priorities).

### Specific Issues

1. **12-parameter system signature** pushing Bevy's SystemParam limits:
   ```rust
   pub fn right_click_move_command(
       mut commands_ecs: Commands,
       buttons: Res<ButtonInput<MouseButton>>,
       keyboard: Res<ButtonInput<KeyCode>>,
       mut interface_state: ResMut<ObjectInterfaceState>,
       cursor_target: Res<CursorTarget>,
       cursor_over_ui: Res<CursorOverUi>,
       mut selected_units: Query<(Entity, &Transform, &UnitBaseEnum, &Owner, Option<&AttackState>, Option<&SupplyChopperState>, &ObjectInstance, Option<&AgentCarryState>, &mut CommandQueue), (With<Unit>, With<Selected>)>,
       target_info: Query<(Option<&SupplyDeliveryStation>, Option<&SupplyTowerState>, &Owner, Option<&SpaceCrystalPatch>, Option<&TunnelState>, Option<&ArmoryState>), With<ObjectInstance>>,
       tiles: Query<(&GridPosition, &TilePreset), With<Tile>>,
       grid: Res<GridMap>,
       local_player: Res<LocalPlayer>,
       occupancy: Res<OccupancyMap>,
   )
   ```

2. **target_info query proliferation**: The 6-Option tuple `(Option<&SupplyDeliveryStation>, Option<&SupplyTowerState>, &Owner, Option<&SpaceCrystalPatch>, Option<&TunnelState>, Option<&ArmoryState>)` grows linearly with each new interactable structure type, and each Option must be destructured in 6+ places.

3. **Duplicated command issuance pattern**: The following pattern appears 10+ times:
   ```rust
   if let Some(attack_state) = attack_state_opt {
       if !attack_state.phase.is_interruptible() { continue; }
   }
   let mut entity_cmds = commands_ecs.entity(entity);
   if !shift_held {
       clear_movement_state_full(&mut entity_cmds);
       entity_cmds.insert(AttackState::default());
   }
   issue_or_queue_command(&mut entity_cmds, &mut command_queue, UnitCommand::X(target), shift_held);
   ```

4. **Linear growth**: Adding a new unit type or command type adds another if-block + inner loop + destructure site.

## Proposed Refactoring

### Phase 1: Extract Command Type Handlers

Extract the body of each command type branch into a standalone function. The main function becomes a dispatcher:

```rust
pub fn right_click_move_command(/* same params */) {
    // ... early returns (cursor checks, placement mode, etc.) stay here ...

    let command_type = interface_state.awaiting_command_type().unwrap_or(CommandType::Default);

    // Entity-click commands (when cursor is on an entity)
    if let Some(target_entity) = cursor_target.entity {
        // Explicit command mode handlers
        if handle_explicit_command_on_entity(command_type, ...) { return; }

        // Right-click contextual handlers (unit-type specific)
        if is_right_click && command_type == CommandType::Default {
            if handle_chopper_right_click(...) { return; }
            if handle_agent_right_click(...) { return; }
            if handle_recruit_right_click(...) { return; }
            if handle_guard_right_click(...) { return; }
        }
    }

    // Ground-click commands
    match command_type {
        CommandType::Move | CommandType::Default => handle_move_ground_command(...),
        CommandType::Patrol => handle_patrol_ground_command(...),
        CommandType::Attack => handle_attack_ground_command(...),
        CommandType::HoldPosition => handle_hold_position_command(...),
        CommandType::Stop => handle_stop_command(...),
        _ => {}
    }
}
```

Each extracted handler is a plain `fn` (not a Bevy system) that receives its needed parameters as arguments. This avoids changing the system registration and keeps the refactor contained.

### Phase 2: CommandContext Struct

Bundle the shared read-only context into a struct to reduce parameter passing:

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

### Phase 3: Deduplicate Command Issuance

Extract the repeated check-interruptible/clear-state/issue-command pattern into a helper:

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

### Phase 4: TargetKind Enum (Optional, Lower Priority)

Replace the 6-Option target_info query with a computed enum. This is a larger change and may be deferred to a follow-up if Phase 1-3 alone reduce the function to manageable size:

```rust
enum TargetKind {
    SupplyStation,
    SupplyTower(SupplyTowerState),
    CrystalPatch,
    Tunnel(TunnelState),
    Armory(ArmoryState),
    Unit,
    Other,
}
```

## Scope

### Files Modified
- `src/game/units/systems/core.rs` — primary target, extract handlers from `right_click_move_command`
- `src/game/units/systems/commands.rs` — new file (or extend existing) for extracted command handler functions

### Files NOT Modified
- No changes to other systems, components, or game logic
- No changes to UI, combat, world, or simulation modules
- The external behavior of the system is unchanged — same inputs produce same outputs

## Constraints

- **Behavioral equivalence**: The refactored code must produce identical behavior. No gameplay changes.
- **No new systems**: The extracted functions are plain Rust functions called from within the existing system, not new Bevy systems. This avoids system ordering changes.
- **Incremental**: Phases 1-3 are the core deliverable. Phase 4 (TargetKind enum) is optional and can be a follow-up.
- **Test compatibility**: All existing unit tests and integration tests must continue to pass without modification (except updating import paths if functions move to a new module).

## Success Metrics

After refactoring:
- `right_click_move_command` body is under 100 lines (dispatch only)
- Each extracted handler is under 60 lines
- The duplicated check-interruptible/clear-state/issue-command pattern appears exactly once (in the helper)
- No function has more than 8 parameters
- Adding a new command type requires adding one handler function and one dispatch line
