# agent_command_behavior_bridge

## Metadata
- **From**: task_splitter
- **To**: task_planner

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

1. **Gather from Crystal** — command_dispatch.rs line ~356: After `issue_or_queue_command` for Gather(crystal), add `entity_cmds.insert(GatheringResourceBehavior::new(target_entity));` (when !shift_held)

2. **Gather from SDS** — command_dispatch.rs line ~377: Same fix for SDS gather path

3. **DropOff via AwaitingTarget** — command_dispatch.rs line ~138: After `issue_or_queue_command` for DropOffResources, add `entity_cmds.insert(DroppingOffResourcesBehavior::new(target_entity));` (when !shift_held)

4. **DropOff via right-click Tunnel** — command_dispatch.rs line ~394-410: In the tunnel right-click block, when `cmd = Some(UnitCommand::DropOffResources(target_entity))`, insert `DroppingOffResourcesBehavior::new(target_entity)` (when !shift_held)

5. **BuildTunnel via placement** — placement.rs line ~512: After `commands.entity(source_entity).insert(UnitCommand::BuildTunnel(world_pos));`, add `.insert(BuildingTunnelBehavior::new(world_pos));`

### Pattern to Follow

Match the SupplyChopper pattern: insert behavior ONLY when !shift_held (immediate command, not queued). For queued commands, a separate dequeue system should handle behavior insertion (future work).

For BuildTunnel in placement.rs, there's no shift_held context — always insert the behavior since placement is always an immediate command.

### Import Requirements

- `GatheringResourceBehavior` — from `game::units::types::state::behavior`
- `DroppingOffResourcesBehavior` — from `game::units::types::state::behavior`  
- `BuildingTunnelBehavior` — from `game::units::types::state::behavior`

### Verification

After fix:
- Right-click crystal patch with Agent → Agent walks to patch and begins mining (GatherPhase::MovingToResource)
- Right-click SDS with Agent → Agent walks to SDS and picks up supplies
- Right-click own Tunnel while carrying → Agent walks to drop-off side
- Press B (DropOff), click own Tunnel → Agent walks to drop-off side
- Place Tunnel via A → Agent walks to build site and begins construction
