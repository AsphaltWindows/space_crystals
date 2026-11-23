# extract_command_handlers_dispatch

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-right_click_command_dispatch.md

## Task

Refactor the body of `right_click_move_command` in `src/game/units/systems/core.rs` into a dispatch pattern by extracting all command-type branches into standalone handler functions. After this task, the main function should be under 100 lines of dispatch logic.

**Prerequisite**: The `CommandContext` struct and `try_issue_command` helper from the previous task must be in place.

### Entity-click handlers to extract

Extract these from the entity-click section (currently lines ~330-753) into standalone `fn` functions. Each handler receives the `CommandContext`, `Commands`, relevant queries, and the target entity.

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

11. **`handle_move_ground`** — Move/Default ground click with pathfinding.
12. **`handle_patrol_ground`** — Patrol ground click with pathfinding.
13. **`handle_attack_ground`** — Attack + ground click -> AttackMove with pathfinding.
14. **`handle_attack_move_ground`** — AttackMove ground click with pathfinding.
15. **`handle_attack_ground_location`** — AttackGround click -> AttackLocation with pathfinding.
16. **`handle_reverse_ground`** — Reverse ground click with pathfinding + can_reverse check.

The remaining ground-click branches (Enter, Build, Gather, DropOff, PickUpSupplies, HoldPosition, Stop, etc.) are simple mode resets (1-2 lines each). These can stay inline in the match dispatcher or be grouped into a `handle_ground_mode_reset` function.

### Constraints
- No extracted handler function should exceed 60 lines
- Each handler returns `bool` (true = handled, caller should return) or is called directly in match arms
- Use `try_issue_command` helper everywhere the duplicated pattern appeared
- The ground-click pathfinding handlers share a common pattern -- consider a local closure or helper for the pathfind+insert part
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

## Technical Context

### Primary file to modify

- **`artifacts/developer/src/game/units/systems/core.rs`** (5612 lines) — contains the monolithic `right_click_move_command` function (lines 243-1008, ~765 lines). This is where extracted handler functions should live as private `fn` items in the same module.

### Current function structure (lines 243-1008)

The function currently takes 12 parameters:
```rust
pub fn right_click_move_command(
    mut commands_ecs: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut interface_state: ResMut<ObjectInterfaceState>,
    cursor_target: Res<CursorTarget>,
    cursor_over_ui: Res<CursorOverUi>,
    mut selected_units: Query<(Entity, &Transform, &UnitBaseEnum, &Owner,
        Option<&AttackState>, Option<&SupplyChopperState>, &ObjectInstance,
        Option<&AgentCarryState>, &mut CommandQueue),
        (With<Unit>, With<Selected>)>,
    target_info: Query<(Option<&SupplyDeliveryStation>, Option<&SupplyTowerState>,
        &Owner, Option<&SpaceCrystalPatch>, Option<&TunnelState>,
        Option<&ArmoryState>), With<ObjectInstance>>,
    tiles: Query<(&GridPosition, &TilePreset), With<Tile>>,
    grid: Res<GridMap>,
    local_player: Res<LocalPlayer>,
    occupancy: Res<OccupancyMap>,
)
```

Layout:
- Lines 257-321: Early returns (no click, cursor over UI, placement mode, SetRallyPoint, ScheduleDeliveries, ownership guard)
- Lines 322-327: Pre-computed booleans (`shift_held`, `has_selected_choppers`, `has_selected_agents`, `has_selected_recruits`)
- Lines 330-753: Entity-click section (`if let Some(target_entity) = cursor_target.entity`)
  - Lines 339-383: Attack entity handler
  - Lines 386-412: DropOff target handler
  - Lines 415-440: Enter target handler
  - Lines 443-472: PickUpSupplies target handler
  - Lines 475-504: AttachToTower target handler
  - Lines 507-536: DropOffSupplies target handler
  - Lines 539-606: Chopper right-click handler
  - Lines 608-687: Agent right-click handler
  - Lines 690-715: Recruit right-click (EnterArmory) handler
  - Lines 718-750: Guard right-click (Enter tunnel) handler
- Lines 756-1008: Ground-click section (match on `command_type`)
  - Lines 765-796: Move/Default
  - Lines 798-828: Patrol
  - Lines 830-859: Attack (-> AttackMove)
  - Lines 861-889: AttackMove
  - Lines 891-919: AttackGround (-> AttackLocation)
  - Lines 921-953: Reverse
  - Lines 955-1005: Simple mode resets (Enter, Build, Gather, DropOff, etc.)

### Prerequisite: CommandContext and try_issue_command

This task depends on the sibling task `command_context_and_issue_helper` (currently active in the pipeline). That task introduces:

1. **`CommandContext` struct** — bundles `shift_held`, `local_player_number`, `is_right_click`, `is_left_click`, `in_command_mode`, `command_type`, `cursor_target` reference. Constructed at the top of `right_click_move_command` after early returns.

2. **`try_issue_command` function** — handles the repeated check-interruptible / clear-state / issue-command pattern. Signature:
```rust
fn try_issue_command(
    commands_ecs: &mut Commands,
    entity: Entity,
    attack_state_opt: Option<&AttackState>,
    command_queue: &mut CommandQueue,
    command: UnitCommand,
    shift_held: bool,
) -> bool
```
Returns true if command was issued (attack phase interruptible or no attack state), false if blocked.

These MUST be in place before this task can start. If they are not present when you begin, coordinate with the other developer task or implement them yourself following the spec in the sibling task.

### Duplicated patterns to eliminate via try_issue_command

The interruptible-check + clear-state + issue pattern appears 13+ times in the function. Each instance follows this shape:
```rust
if let Some(attack_state) = attack_state_opt {
    if !attack_state.phase.is_interruptible() { continue; }
}
let mut entity_cmds = commands_ecs.entity(entity);
if !shift_held {
    clear_movement_state_full(&mut entity_cmds);
    entity_cmds.insert(AttackState::default());
}
issue_or_queue_command(&mut entity_cmds, &mut command_queue, cmd, shift_held);
```

**Variation**: Attack entity handler (line 339) does NOT insert `AttackState::default()` on clear. The `try_issue_command` helper handles the common part; the attack handler adds its own behavior after.

### Ground-click pathfinding common pattern

The 6 ground-click movement handlers (Move, Patrol, Attack, AttackMove, AttackGround, Reverse) all share this pattern:
```rust
if shift_held {
    command_queue.push(cmd);
} else {
    command_queue.clear();
    let start_grid = world_to_grid(transform.translation);
    if let Some(path) = find_path_for_domain(start_grid, target_grid, &tiles,
        unit_base, grid.width as i32, grid.height as i32, &occupancy,
        (start_grid.x, start_grid.z)) {
        let smoothed_waypoints = smooth_path(path);
        commands_ecs.entity(entity)
            .remove::<HoldingPosition>()
            .remove::<IdleOrigin>()
            .insert((MoveTarget(target_pos), Path { ... }, cmd, AttackState::default()));
    }
}
```

**Variations**: Attack and AttackMove do NOT insert `AttackState::default()` in the bundle. Reverse adds a `can_reverse` check. Consider a helper closure/function that takes the command and a bool for whether to insert AttackState::default().

### Behavior marker components (chopper handlers)

Three chopper handlers insert additional behavior markers AFTER issuing the command (only when `!shift_held`):
- `PickingUpSuppliesBehavior::new(target_entity)` — for PickUpSupplies
- `AttachingToTowerBehavior::new(target_entity)` — for AttachToTower
- `DroppingOffSuppliesBehavior::new(target_entity)` — for DropOffSupplies

These are defined in `artifacts/developer/src/game/units/types/state/behavior.rs` (lines 334, 353, 367). They are `#[derive(Component)]` structs with a `new(Entity)` constructor.

### Key helper functions used

- **`clear_movement_state_full`** (`artifacts/developer/src/game/units/utils.rs:141`): Removes MoveTarget, Path, HoldingPosition, IdleOrigin; inserts Velocity(Vec3::ZERO)
- **`issue_or_queue_command`** (`artifacts/developer/src/game/units/utils.rs:153`): Shift=push to queue; !shift=clear queue + insert UnitCommand
- **`can_enter_tunnel`** (`artifacts/developer/src/game/units/utils.rs:167`): Validates Syndicate faction, ownership, tier compatibility
- **`world_to_grid`** (`artifacts/developer/src/game/units/utils.rs`): Vec3 -> GridPosition conversion
- **`smooth_path`** (`artifacts/developer/src/game/units/utils.rs`): Smooths A* path waypoints
- **`find_path_for_domain`** (`artifacts/developer/src/game/units/pathfinding.rs:10`): A* pathfinding with domain-aware walkability

### Key types involved

- **`CommandType`** enum (`artifacts/developer/src/game/units/types/state/commands.rs:88`): Default, Move, Attack, AttackGround, AttackMove, Patrol, Reverse, Enter, Gather, DropOff, PickUpSupplies, AttachToTower, DropOffSupplies, etc.
- **`UnitCommand`** enum: Idle, Move(Vec3), AttackTarget(Entity), Gather(Entity), Enter(Entity), DropOffResources(Entity), PickUpSupplies(Entity), AttachToTower(Entity), DropOffSupplies(Entity), EnterArmory(Entity), Patrol{...}, AttackMove(Vec3), AttackLocation(Vec3), Reverse(Vec3)
- **`ObjectInterfaceState`** (`artifacts/developer/src/ui/types.rs`): Default, AwaitingTarget(CommandType), AgentMenu(AgentMenuState), etc.
- **`CursorTarget`** resource: `kind: CursorTargetEnum`, `location: Option<Vec3>`, `entity: Option<Entity>`
- **`ArmoryState`** (`artifacts/developer/src/game/types/structures.rs`): `stored_recruits: Vec<Entity>`
- **`ARMORY_INTERNAL_RECRUIT_CAPACITY`** (`artifacts/developer/src/game/types/cults_structure_stats.rs`)

### Interface state return values per handler

Each handler resets the interface state upon handling. These return targets MUST be preserved:
- Attack entity -> `Default`
- DropOff target -> `AgentMenu(AgentDefault)` (both success and failure)
- Enter target -> `Default` (both success and failure)
- PickUpSupplies/AttachToTower/DropOffSupplies -> `Default` (both success and failure)
- Chopper right-click -> `Default`
- Agent right-click (crystal/SDS/tunnel) -> `Default`
- Recruit right-click -> `Default`
- Guard right-click -> `Default`
- All ground-click handlers -> `Default` (except Gather/DropOff which return to `AgentMenu(AgentDefault)`)

### Existing tests (must pass unchanged)

Tests in `core.rs` starting at line 2976 that exercise `right_click_move_command`:
- `dropoff_left_click_own_tunnel_issues_command` (line 3918) — uses `run_system_once(right_click_move_command)`
- `dropoff_left_click_non_tunnel_resets_to_agent_default` (line 3952)
- `dropoff_left_click_enemy_tunnel_resets_to_agent_default` (line 3992)
- `enter_left_click_own_tunnel_issues_command` (line ~4139+)
- `enter_left_click_enemy_tunnel_does_not_issue_command` (line ~4200+)
- `enter_left_click_non_tunnel_resets` (line ~4240+)
- `enter_left_click_tier_mismatch_does_not_issue` (line ~4270+)
- `recruit_right_click_armory_issues_enter_armory` (line ~4310+)
- `guard_right_click_own_tunnel_issues_enter` (line ~4350+)
- Ground-click pathfinding tests near line 5452+

All use `world.run_system_once(right_click_move_command)` — refactoring the internals of the system function will not break these tests as long as the system signature and behavior remain identical.

### System registration

`right_click_move_command` is registered in `UnitsPlugin` (`artifacts/developer/src/game/units/mod.rs:87`) under `DiagCategory::Movement` in the Update schedule. `command_indicator_sync_system` runs `.after(right_click_move_command)` (line 91). The system signature must remain unchanged -- only the internals are refactored.

### Recommended placement of handler functions

Keep all extracted handlers as **private functions in `core.rs`** (not `commands.rs`). Rationale:
1. They are tightly coupled to the `right_click_move_command` system's query types and parameters
2. Moving them to `commands.rs` would require re-exporting many types and query tuples
3. `commands.rs` (975 lines) already has its own domain (hotkey systems, command state sync, command dispatch)
4. Private functions in `core.rs` avoids any import/visibility changes

Place them just before the `right_click_move_command` function definition or immediately after it (before `set_rally_point_click_system` at line 1010).

## Dependencies

- **`command_context_and_issue_helper` planned task** (sibling task, currently active in pipeline): Must be completed first. This task introduces the `CommandContext` struct and `try_issue_command` helper that all extracted handlers will use. Without these, the refactored dispatch code has no shared context struct to pass to handlers and no helper to replace the duplicated interruptible-check pattern.
