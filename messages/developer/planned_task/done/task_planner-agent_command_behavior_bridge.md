# agent_command_behavior_bridge

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-agent_interface_r1.md

## Task

Insert behavior components alongside UnitCommand for Agent-specific commands. Currently, right-click and placement handlers issue UnitCommand::Gather, UnitCommand::DropOffResources, and UnitCommand::BuildTunnel, but do NOT insert the corresponding behavior components (GatheringResourceBehavior, DroppingOffResourcesBehavior, BuildingTunnelBehavior). The behavior systems only query for entities WITH these components, so commands are silently ignored at runtime.

### Root Cause

Compare the SupplyChopper pattern (which works) to the Agent pattern (which doesn't):

**SupplyChopper (works)** — command_dispatch.rs inserts BOTH command AND behavior:
```rust
issue_or_queue_command(..., UnitCommand::PickUpSupplies(target_entity), shift_held);
entity_cmds.insert(PickingUpSuppliesBehavior::new(target_entity));
```

**Agent Gather (broken)** — command_dispatch.rs only inserts command, no behavior:
```rust
issue_or_queue_command(..., UnitCommand::Gather(target_entity), shift_held);
// Missing: entity_cmds.insert(GatheringResourceBehavior::new(target_entity));
```

### Locations to Fix

1. **Gather from Crystal** — command_dispatch.rs line ~356: After `issue_or_queue_command` for Gather(crystal), add behavior insert
2. **Gather from SDS** — command_dispatch.rs line ~377: Same fix for SDS gather path
3. **DropOff via AwaitingTarget** — command_dispatch.rs line ~138: After `issue_or_queue_command` for DropOffResources, add behavior insert
4. **DropOff via right-click Tunnel** — command_dispatch.rs line ~394-410: In the tunnel right-click block, add behavior insert
5. **BuildTunnel via placement** — placement.rs line ~512: After inserting UnitCommand::BuildTunnel, add behavior insert

### Pattern to Follow

Match the SupplyChopper pattern: insert behavior ONLY when !shift_held (immediate command, not queued). For queued commands, a separate dequeue system should handle behavior insertion (future work). For BuildTunnel in placement.rs, there's no shift_held context — always insert the behavior since placement is always an immediate command.

### Verification

After fix: Right-click crystal patch with Agent → Agent walks to patch and begins mining. Right-click SDS with Agent → Agent gathers supplies. Right-click own Tunnel while carrying → Agent drops off. Press B (DropOff), click own Tunnel → Agent drops off. Place Tunnel via A → Agent walks to build site and begins construction.

## Technical Context

### Files to Modify

1. **`artifacts/developer/src/game/units/systems/core/command_dispatch.rs`** — Main file with 5 fix locations.
   - **Imports**: No new imports needed. `GatheringResourceBehavior`, `DroppingOffResourcesBehavior`, and `BuildingTunnelBehavior` are all accessible via the existing `use crate::game::units::types::*;` (line 9), which re-exports `state::*` → `behavior::*`.
   
2. **`artifacts/developer/src/game/world/faction/placement.rs`** — BuildTunnel placement fix.
   - **Imports**: `BuildingTunnelBehavior` is already explicitly imported (line 10).

### Exact Fix Locations in command_dispatch.rs

**Fix 1 — Gather from Crystal (line ~356):**
Current code:
```rust
issue_or_queue_command(&mut entity_cmds, &mut command_queue, UnitCommand::Gather(target_entity), shift_held);
// line 357: closing brace
```
Add after the issue_or_queue_command call, before the closing brace:
```rust
if !shift_held {
    entity_cmds.insert(GatheringResourceBehavior::new(target_entity));
}
```

**Fix 2 — Gather from SDS (line ~377):**
Same pattern as Fix 1 — add `GatheringResourceBehavior::new(target_entity)` insert after `issue_or_queue_command`.

**Fix 3 — DropOff via AwaitingTarget (line ~138):**
Current code:
```rust
issue_or_queue_command(&mut entity_cmds, &mut command_queue, UnitCommand::DropOffResources(target_entity), shift_held);
// line 139-142: closing braces + info! + state reset
```
Add after the issue_or_queue_command call:
```rust
if !shift_held {
    entity_cmds.insert(DroppingOffResourcesBehavior::new(target_entity));
}
```

**Fix 4 — DropOff/Enter via right-click Tunnel (lines ~404-410):**
Current code (line 404-410):
```rust
if let Some(cmd) = cmd {
    let mut entity_cmds = commands_ecs.entity(entity);
    if !shift_held {
        clear_movement_state_full(&mut entity_cmds);
        entity_cmds.insert(AttackState::default());
    }
    issue_or_queue_command(&mut entity_cmds, &mut command_queue, cmd, shift_held);
}
```
After the `issue_or_queue_command` call, add behavior insert based on which command was chosen:
```rust
if !shift_held {
    match &cmd {
        UnitCommand::DropOffResources(target) => {
            entity_cmds.insert(DroppingOffResourcesBehavior::new(*target));
        }
        UnitCommand::Enter(target) => {
            entity_cmds.insert(EnteringTunnelBehavior::new(*target));
        }
        _ => {}
    }
}
```
**IMPORTANT**: The `cmd` variable is moved into `issue_or_queue_command`, so you need to either clone it before or restructure — capture the target entity and command variant before the call. The simplest approach: capture the cmd variant before issuing:
```rust
if let Some(cmd) = cmd {
    let mut entity_cmds = commands_ecs.entity(entity);
    if !shift_held {
        clear_movement_state_full(&mut entity_cmds);
        entity_cmds.insert(AttackState::default());
    }
    // Insert behavior before issuing command (since issue_or_queue_command consumes cmd)
    if !shift_held {
        match &cmd {
            UnitCommand::DropOffResources(t) => {
                entity_cmds.insert(DroppingOffResourcesBehavior::new(*t));
            }
            UnitCommand::Enter(t) => {
                entity_cmds.insert(EnteringTunnelBehavior::new(*t));
            }
            _ => {}
        }
    }
    issue_or_queue_command(&mut entity_cmds, &mut command_queue, cmd, shift_held);
}
```
Note: Check `issue_or_queue_command` signature — if it takes `cmd` by value, place the match before the call. If by reference, after is fine.

**Note on EnteringTunnelBehavior**: Verify this type exists in `behavior.rs`. If it doesn't exist, skip the Enter arm — the task description only lists Gather, DropOff, and BuildTunnel. But include it if it exists since the code path also handles Enter.

### Exact Fix Location in placement.rs

**Fix 5 — BuildTunnel placement (line ~512):**
Current code:
```rust
commands.entity(source_entity)
    .insert(UnitCommand::BuildTunnel(world_pos));
```
Change to:
```rust
commands.entity(source_entity)
    .insert(UnitCommand::BuildTunnel(world_pos))
    .insert(BuildingTunnelBehavior::new(world_pos));
```
`BuildingTunnelBehavior` is already imported on line 10.

### Reference Pattern (SupplyChopper — working correctly)

See lines 195-198 (PickUpSupplies), 259-262 (DropOffSupplies), 320-325 (DropOffSupplies/AttachToTower right-click). All follow: `issue_or_queue_command(...)` then `if !shift_held { entity_cmds.insert(SomeBehavior::new(target)); }`.

### Existing Tests to Update

Tests in command_dispatch.rs verify UnitCommand insertion but do NOT check behavior components. Consider adding behavior assertions to:
- `dropoff_left_click_own_tunnel_issues_command` (line 1020) — verify `DroppingOffResourcesBehavior` is present
- Gather tests (if they exist) — verify `GatheringResourceBehavior`
- The right-click tunnel interaction tests (line ~1230) — verify `DroppingOffResourcesBehavior`

### Key Types

- `GatheringResourceBehavior` — Component, has `::new(target_resource: Entity)`, fields: `target_resource`, `phase` (GatherPhase)
- `DroppingOffResourcesBehavior` — Component, has `::new(target_tunnel: Entity)`, field: `target_tunnel`
- `BuildingTunnelBehavior` — Component, has `::new(target_location: Vec3)`, field: `target_location`
- `EnteringTunnelBehavior` — check if exists in `behavior.rs`; if so has `::new(target_tunnel: Entity)`
- `issue_or_queue_command` — from `crate::game::units::utils`, takes `(&mut EntityCommands, &mut CommandQueue, UnitCommand, bool)`

## Dependencies

None — this task is standalone. It fixes missing behavior insertions in existing command dispatch code. The behavior types and command types all exist already.
