# enter-right-click-integration

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-enter-command-tunnel.md

## Task

Add Enter command right-click resolution for BasicCombatUnit and tier validation, plus AwaitingTarget[Enter] entity click handling.

### What exists already
- Agent right-click own Tunnel in `right_click_move_command` (`src/game/units/systems/core.rs` lines ~390-412): issues Enter if not carrying, DropOff if carrying — but does NOT validate tunnel tier
- AwaitingTarget[Enter] ground click handler (core.rs line ~582): resets to Default (correct — Enter needs entity target)
- `can_enter_tunnel()` utility in `src/game/units/utils.rs`
- Entity-click section in `right_click_move_command` (core.rs line ~256) handles Attack, Chopper targets, Agent targets — but NOT BasicCombatUnit tunnel targets

### What needs to be implemented

1. **BasicCombatUnit right-click own Tunnel → Enter** (in the entity-click section of `right_click_move_command`, core.rs):
   - After the Agent-specific block (~line 414) and before the fall-through to ground Move
   - When: `is_right_click && command_type == CommandType::Default`, target is an own Tunnel
   - For each selected Syndicate unit (not Agent — Agent is already handled above): validate `can_enter_tunnel()` with the unit's base type and tunnel tier
   - If valid: clear movement state, insert `UnitCommand::Enter(target_entity)`
   - The query needs access to `TunnelState` (for tier) and `Owner` on the target, and `UnitBaseEnum` + faction info on selected units
   - Note: the selected_units query tuple may need extending to include `UnitBaseEnum` if not already present

2. **Add tier validation to Agent right-click Enter** (core.rs lines ~390-412):
   - Currently the Agent right-click own Tunnel block issues `UnitCommand::Enter` without checking tier
   - Add `can_enter_tunnel()` check — if tier insufficient, fall through (don't issue Enter, treat as regular right-click → Move)
   - Agent is HeavyInfantry, so Tier 1+ always allows them — but the validation should still be called for correctness and future-proofing

3. **AwaitingTarget[Enter] left-click entity resolution** (in the entity-click section of `right_click_move_command`, core.rs):
   - When `command_type == CommandType::Enter` and `is_left_click` and target is an own Tunnel with sufficient tier
   - Issue `UnitCommand::Enter(target_entity)` to selected Syndicate units
   - Reset interface state to Default
   - If target is not a valid tunnel, reset interface state without issuing command

4. **Tests**:
   - Test BasicCombatUnit (Guard) right-click own tunnel issues Enter
   - Test tier validation rejects vehicle Enter on Tier 1 tunnel
   - Test Agent carrying resources right-clicks tunnel → DropOff (not Enter)
   - Test AwaitingTarget[Enter] left-click valid tunnel issues Enter command

## Technical Context

### Primary file to modify
- **`artifacts/developer/src/game/units/systems/core.rs`** — the `right_click_move_command` system (starts line 179)

### Key function: `right_click_move_command` structure
- **Line 179**: Function signature with 11 query/resource params
- **Line 185**: `selected_units` query already includes `&UnitBaseEnum` (index 2) and `&Owner` (index 3) and `&ObjectInstance` (index 6) — no query changes needed
- **Line 186**: `target_info` query already includes `Option<&TunnelState>` (index 4) and `&Owner` (index 2) — tunnel tier accessible via `tunnel_state.tier`
- **Lines 256-416**: Entity-click section (where all 3 new blocks go)
- **Lines 418-600**: Ground-click section (Enter ground-click at line 582 already resets to Default — correct)

### New import needed
Add `can_enter_tunnel` to the import line 12:
```rust
use crate::game::units::utils::{world_to_grid, create_attack_capability, smooth_path, clear_movement_state_full, can_enter_tunnel};
```

### Implementation point 1: AwaitingTarget[Enter] entity-click handler
Insert a new block after the DropOff handler (line 310) and before the chopper handler (line 313). Pattern follows the existing DropOff handler (lines 287-310):
```rust
// Left-click entity in Enter mode: target must be own Tunnel with valid tier
if command_type == CommandType::Enter {
    if let Some(target_entity) = cursor_target.entity {
        if let Ok((_sds_opt, _st_opt, target_owner, _crystal_opt, tunnel_opt)) = target_info.get(target_entity) {
            if let Some(tunnel_state) = tunnel_opt {
                if target_owner.player_number() == Some(local_player.0) {
                    for (entity, _, unit_base, owner, attack_state_opt, _, obj, _) in &selected_units {
                        let is_syndicate = matches!(obj.object_type, ObjectEnum::SyndicateAgent | ObjectEnum::SyndicateGuard);
                        if can_enter_tunnel(is_syndicate, owner.player_number(), target_owner.player_number(), unit_base, &tunnel_state.tier).is_ok() {
                            if let Some(attack_state) = attack_state_opt {
                                if !attack_state.phase.is_interruptible() { continue; }
                            }
                            let mut entity_cmds = commands_ecs.entity(entity);
                            clear_movement_state_full(&mut entity_cmds);
                            entity_cmds.insert(UnitCommand::Enter(target_entity));
                        }
                    }
                    *interface_state = ObjectInterfaceState::Default;
                    return;
                }
            }
        }
    }
    // Invalid target — reset without issuing command
    *interface_state = ObjectInterfaceState::Default;
    return;
}
```
Note: this block fires on BOTH left-click (command mode) AND right-click (because Enter also works as a right-click target command). The `command_type == CommandType::Enter` check gates it.

**IMPORTANT**: This block must be placed BEFORE the chopper/agent right-click sections since Enter mode uses left-click, not right-click. Actually, it should be placed right after the DropOff handler and before the chopper block for consistency with the DropOff pattern.

### Implementation point 2: Add tier validation to Agent right-click Enter (lines 390-412)
In the existing Agent tunnel block (line 391-412), wrap the Enter branch with a `can_enter_tunnel()` check:
```rust
// Existing code at line 402-405:
if carry_opt.map(|cs| cs.is_carrying()).unwrap_or(false) {
    entity_cmds.insert(UnitCommand::DropOffResources(target_entity));
} else {
    // NEW: validate tier before Enter
    if let Ok((_,_,_,_,tunnel_opt)) = target_info.get(target_entity) {
        if let Some(ts) = tunnel_opt {
            if can_enter_tunnel(true, owner.player_number(), target_owner.player_number(), unit_base, &ts.tier).is_ok() {
                entity_cmds.insert(UnitCommand::Enter(target_entity));
            }
            // else: tier insufficient, don't issue Enter (fall through to Move)
        }
    }
}
```
**Note**: The Agent closure destructure must also capture `unit_base` — check that the loop variable unpacks index 2. Current loop is: `for (entity, _, _, _, attack_state_opt, _, obj, carry_opt)` — it skips indices 1,2,3. Change to capture `unit_base` at index 2 (currently `_`). The `target_owner` variable is already in scope from the outer `if let Ok(...)` (line 353).

BUT there's a subtlety: `target_info.get(target_entity)` is already done in the outer scope — the `tunnel_opt` from line 353's destructure already has the TunnelState. So you can use the `tunnel_opt` already in scope and just call:
```rust
if let Some(ts) = tunnel_opt {
    if can_enter_tunnel(true, owner.player_number(), target_owner.player_number(), unit_base, &ts.tier).is_ok() {
        entity_cmds.insert(UnitCommand::Enter(target_entity));
    }
}
```
This requires the loop to capture `owner` (index 3, currently `_`) and `unit_base` (index 2, currently `_`).

### Implementation point 3: BasicCombatUnit (Guard) right-click Enter
Insert a new block after the Agent block ends (line 414), before the fall-through comment (line 415):
```rust
// Right-click on non-enemy Tunnel: BasicCombatUnit (Guard etc.) Enter
if is_right_click && command_type == CommandType::Default {
    if let Some(target_entity) = cursor_target.entity {
        if let Ok((_sds_opt, _st_opt, target_owner, _crystal_opt, tunnel_opt)) = target_info.get(target_entity) {
            if let Some(tunnel_state) = tunnel_opt {
                if target_owner.player_number() == Some(local_player.0) {
                    let mut any_entered = false;
                    for (entity, _, unit_base, owner, attack_state_opt, _, obj, _) in &selected_units {
                        // Skip Agents (already handled above)
                        if obj.object_type == ObjectEnum::SyndicateAgent { continue; }
                        let is_syndicate = matches!(obj.object_type, ObjectEnum::SyndicateGuard);
                        if !is_syndicate { continue; }
                        if can_enter_tunnel(is_syndicate, owner.player_number(), target_owner.player_number(), unit_base, &tunnel_state.tier).is_ok() {
                            if let Some(attack_state) = attack_state_opt {
                                if !attack_state.phase.is_interruptible() { continue; }
                            }
                            let mut entity_cmds = commands_ecs.entity(entity);
                            clear_movement_state_full(&mut entity_cmds);
                            entity_cmds.insert(UnitCommand::Enter(target_entity));
                            any_entered = true;
                        }
                    }
                    if any_entered {
                        info!("Guard: Enter tunnel");
                        *interface_state = ObjectInterfaceState::Default;
                        return;
                    }
                }
            }
        }
    }
}
```
**Design choice**: If no unit could enter (tier too low), fall through to ground Move instead of consuming the click. This matches the task description's "fall through" semantics.

### Key types and their locations
- **`can_enter_tunnel(is_syndicate, unit_owner, tunnel_owner, unit_base, tunnel_tier) -> Result<(), &str>`** — `artifacts/developer/src/game/units/utils.rs:157`
- **`TunnelState { tier: TunnelTier, current_operation: Option<TunnelOperation> }`** — `artifacts/developer/src/game/types/structures.rs:572`
- **`TunnelTier::can_transit(&self, base: &UnitBaseEnum) -> bool`** — `structures.rs:554`
- **`ObjectEnum::SyndicateAgent`, `ObjectEnum::SyndicateGuard`** — `artifacts/developer/src/shared/types.rs:312-313`
- **`UnitBaseEnum`** — in `crate::types` (already imported via `use crate::types::*`)
- **`clear_movement_state_full(&mut EntityCommands)`** — already imported in core.rs line 12
- **`ObjectInterfaceState::Default`** / `ObjectInterfaceState::AwaitingTarget(CommandType::Enter)`** — `artifacts/developer/src/ui/types.rs`
- **`AgentMenuState::AgentDefault`** — `artifacts/developer/src/ui/types.rs`
- **`Owner::player_number() -> Option<u8>`** — `artifacts/developer/src/shared/types.rs:51`

### Test patterns (follow existing tests at line ~2346)
- Use `World::new()` + `run_system_once(right_click_move_command)`
- Helper: `spawn_selected_agent(world)` spawns Agent (LightInfantry, player 0)
- Helper: `spawn_own_tunnel(world)` spawns Tunnel (player 0, Tier 1)
- Helper: `spawn_enemy_tunnel(world)` spawns Tunnel (player 1, Tier 1)
- For Guard tests: spawn with `UnitBaseEnum::HeavyInfantry`, `ObjectEnum::SyndicateGuard`, `Owner::player(0)`
- For Vehicle tests: spawn with `UnitBaseEnum::WheeledVehicle` and appropriate ObjectEnum
- Set `ButtonInput::<MouseButton>` with `press(MouseButton::Right)` or `press(MouseButton::Left)`
- Set `CursorTarget { kind, location, entity }`
- Set `ObjectInterfaceState` resource
- Resources needed: `LocalPlayer(0)`, `GridMap`, `OccupancyMap::default()`, `CursorOverUi(false)`
- Assert: check `UnitCommand` component on entity, check `ObjectInterfaceState` resource

### Guard entity spawn pattern (new helper suggestion)
```rust
fn spawn_selected_guard(world: &mut World) -> Entity {
    world.spawn((
        Unit,
        Selected,
        Transform::from_xyz(0.0, 0.5, 0.0),
        UnitBaseEnum::HeavyInfantry,
        Owner::player(0),
        ObjectInstance::destructible(ObjectEnum::SyndicateGuard, 100.0),
        UnitCommand::Idle,
    )).id()
}
```

## Dependencies

- **`can_enter_tunnel` utility** (`artifacts/developer/src/game/units/utils.rs:157`) — Already exists with full test coverage. This task calls it but does not need to modify it.
- **TunnelState/TunnelTier types** (`artifacts/developer/src/game/types/structures.rs`) — Already exist. The `target_info` query in `right_click_move_command` already includes `Option<&TunnelState>`.
- No other planned_tasks are dependencies — this task is self-contained within `core.rs`.
