# enter-command-behavior-pipeline

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-enter-command-tunnel.md

## Task

Wire UnitCommand::Enter to EnteringTunnelBehavior and fix the behavior system's tunnel network integration.

### What exists already
- `EnteringTunnelBehavior` component in `src/game/units/types/state/behavior.rs` (line ~135)
- `entering_tunnel_behavior_system` in `src/game/units/systems/behaviors.rs` (line ~420) — moves unit toward tunnel, despawns on arrival
- `UnitCommand::Enter(Entity)` variant in `src/game/units/types/state/commands.rs`
- `can_enter_tunnel()` validation in `src/game/units/utils.rs` (line ~157)
- `InTunnelNetwork` component used by production system (`src/game/world/faction.rs` line ~472)
- `tunnel_side_world_position()` utility re-exported from `src/game/units/utils.rs`

### What needs to be implemented

1. **Command dispatch system**: Create a system (or add to an existing one) that detects units with `UnitCommand::Enter(tunnel_entity)` and:
   - Validates the Enter command using `can_enter_tunnel()` (check faction, owner match, tier sufficiency)
   - If valid: inserts `EnteringTunnelBehavior::new(tunnel_entity)` on the unit, pathfinds to Side A position using `tunnel_side_world_position()`
   - If invalid (e.g., tier too low): rejects the command (remove UnitCommand, log warning)

2. **Fix entering_tunnel_behavior_system**: Currently the system calls `commands.entity(entity).despawn()` on arrival at Side A. This is wrong — the production system uses `InTunnelNetwork` + `Visibility::Hidden` to keep entities queryable for ejection. Fix to:
   - Remove `EnteringTunnelBehavior` marker
   - Insert `InTunnelNetwork { owner_player }` (get owner from the unit's Owner component)
   - Set `Visibility::Hidden`
   - Remove movement-related components (MoveTarget, Path, Velocity, etc.)
   - Set locomotion/orientation channels to stopped/maintaining
   - Do NOT despawn the entity

3. **Tests**:
   - Test that UnitCommand::Enter triggers EnteringTunnelBehavior insertion
   - Test that tier validation rejects invalid entries
   - Test that on arrival, unit gets InTunnelNetwork + Hidden (not despawned)
   - Test that the unit entity still exists after entering the network
