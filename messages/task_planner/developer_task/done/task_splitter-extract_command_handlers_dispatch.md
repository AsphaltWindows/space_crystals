# extract_command_handlers_dispatch

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-right_click_command_dispatch.md

## Task

Refactor the body of `right_click_move_command` in `src/game/units/systems/core.rs` into a dispatch pattern by extracting all command-type branches into standalone handler functions. After this task, the main function should be under 100 lines of dispatch logic.

**Prerequisite**: The `CommandContext` struct and `try_issue_command` helper from the previous task must be in place.

### Entity-click handlers to extract

Extract these from the entity-click section (currently lines ~330-753) into standalone `fn` functions. Each handler receives the `CommandContext`, `Commands`, relevant queries, and the target entity. Place them in `src/game/units/systems/commands.rs` (the file already exists — add to it) or keep them in `core.rs` as private functions, whichever is cleaner.

1. **`handle_attack_entity`** — Attack command on entity (explicit Attack mode or right-click on enemy). Includes approach path computation.
2. **`handle_dropoff_target`** — Left-click entity in DropOff mode (agent -> own tunnel).
3. **`handle_enter_target`** — Left-click entity in Enter mode (unit -> own tunnel with tier validation).
4. **`handle_pickup_supplies_target`** — Left-click entity in PickUpSupplies mode (chopper -> SDS).
5. **`handle_attach_to_tower_target`** — Left-click entity in AttachToTower mode (chopper -> own SupplyTower).
6. **`handle_dropoff_supplies_target`** — Left-click entity in DropOffSupplies mode (chopper -> own SupplyTower).
7. **`handle_chopper_right_click`** — Right-click default mode with selected choppers (SDS -> PickUp, own SupplyTower -> DropOff/Attach based on carried_supplies).
8. **`handle_agent_right_click`** — Right-click default mode with selected agents (Crystal/SDS -> Gather, own Tunnel -> DropOff/Enter).
9. **`handle_recruit_right_click`** — Right-click default mode with selected recruits (own Armory -> EnterArmory).
10. **`handle_guard_right_click`** — Right-click default mode with Guard units (own Tunnel -> Enter).

### Ground-click handlers to extract

Extract these from the ground-click match (currently lines ~764-1005):

11. **`handle_move_ground`** — Move/Default ground click with pathfinding.
12. **`handle_patrol_ground`** — Patrol ground click with pathfinding.
13. **`handle_attack_ground`** — Attack + ground click -> AttackMove with pathfinding.
14. **`handle_attack_move_ground`** — AttackMove ground click with pathfinding.
15. **`handle_attack_ground_location`** — AttackGround click -> AttackLocation with pathfinding.
16. **`handle_reverse_ground`** — Reverse ground click with pathfinding + can_reverse check.

The remaining ground-click branches (Enter, Build, Gather, DropOff, PickUpSupplies, HoldPosition, Stop, etc.) are simple mode resets (1-2 lines each). These can stay inline in the match dispatcher or be grouped into a `handle_ground_mode_reset` function.

### Dispatch structure

The refactored `right_click_move_command` should look like:

```rust
pub fn right_click_move_command(/* same params */) {
    // Early returns (cursor checks, placement mode, etc.) — keep as-is (~25 lines)
    let ctx = CommandContext { ... };
    
    // Entity-click dispatch
    if let Some(target_entity) = cursor_target.entity {
        // Explicit command mode entity handlers
        if handle_attack_entity(&ctx, ...) { return; }
        if handle_dropoff_target(&ctx, ...) { return; }
        if handle_enter_target(&ctx, ...) { return; }
        if handle_pickup_supplies_target(&ctx, ...) { return; }
        if handle_attach_to_tower_target(&ctx, ...) { return; }
        if handle_dropoff_supplies_target(&ctx, ...) { return; }
        
        // Right-click contextual handlers
        if ctx.is_right_click && ctx.command_type == CommandType::Default {
            if handle_chopper_right_click(&ctx, ...) { return; }
            if handle_agent_right_click(&ctx, ...) { return; }
            if handle_recruit_right_click(&ctx, ...) { return; }
            if handle_guard_right_click(&ctx, ...) { return; }
        }
    }
    
    // Ground-click dispatch
    match ctx.command_type {
        CommandType::Move | CommandType::Default => handle_move_ground(&ctx, ...),
        CommandType::Patrol => handle_patrol_ground(&ctx, ...),
        // ... etc
    }
}
```

### Constraints
- No extracted handler function should exceed 60 lines
- Each handler returns `bool` (true = handled, caller should return) or is called directly in match arms
- Use `try_issue_command` helper everywhere the duplicated pattern appeared
- The ground-click pathfinding handlers share a common pattern — consider a local closure or helper for the pathfind+insert part
- **Behavioral equivalence**: Same inputs must produce same outputs. No command paths lost.
- All existing tests must pass unchanged

### New tests to write

Write unit tests for the extracted handlers:
- `test_handle_attack_entity_command`: Attack command type + cursor on enemy -> AttackTarget issued
- `test_handle_move_ground_command`: Right-click ground -> Move issued with pathfinding
- `test_handle_chopper_pickup_command`: PickUpSupplies command on SDS -> PickUpSupplies issued
- `test_handle_chopper_dropoff_supplies`: Chopper with carried supplies, right-click own SupplyTower -> DropOffSupplies issued
- `test_handle_chopper_attach_to_tower`: Chopper empty, right-click own SupplyTower -> AttachToTower issued
- `test_handle_agent_gather_crystal`: Agent right-click crystal patch -> Gather issued
- `test_handle_agent_enter_tunnel`: Agent not carrying, right-click own tunnel -> Enter issued
- `test_handle_agent_dropoff_tunnel`: Agent carrying, right-click own tunnel -> DropOffResources issued
- `test_handle_recruit_enter_armory`: Recruit right-click own armory with capacity -> EnterArmory issued
- `test_handle_guard_enter_tunnel`: Guard right-click own tunnel -> Enter issued
