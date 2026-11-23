# action-channels-targeted-attack-fix

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-action_channels_r1.md

## Task

Fix targeted attack (Attack+click enemy / right-click enemy) so the unit pursues and attacks the target instead of stopping in place.

### Root Cause

The `attacking_object_behavior_system` (combat/systems/behaviors.rs, ~lines 22-121) sets `AttackPhase::Aiming` when it finds a target, then uses legacy `MoveTarget/Path` to approach out-of-range targets. However, `attack_channel_sync_system` (combat/systems/core.rs, ~lines 575-631) runs after and forces `LocomotionChannel::Stationary` whenever `AttackPhase` is Aiming/Firing/Cooldown — even when the unit is out of range and needs to approach.

### Required Changes

1. **In `attacking_object_behavior_system`**: When the target is OUT of range, write `LocomotionChannel::Moving(waypoints)` to drive approach movement via channels instead of (or in addition to) legacy MoveTarget/Path. When IN range, write `LocomotionChannel::Stationary`.

2. **In `attack_channel_sync_system`**: Only force `LocomotionChannel::Stationary` during Aiming/Firing/Cooldown when the unit is actually in attack range of the target. If out of range, do NOT override the locomotion channel — let the behavior system's approach movement persist. This may require passing range information through a component or checking distance in the sync system.

3. Ensure `OrientationChannel::Turning(target_pos)` is written when in range (already done by sync system) and during approach.

### Verification

- Select an infantry unit, press Attack, left-click an enemy → unit moves toward enemy and attacks.
- Right-click an enemy → same behavior.
- Unit transitions through Aiming → Firing → Cooldown → Reloading phases when in range.

### Key Files
- `artifacts/developer/src/game/combat/systems/behaviors.rs` — attacking_object_behavior_system
- `artifacts/developer/src/game/combat/systems/core.rs` — attack_channel_sync_system
- `artifacts/developer/src/game/combat/mod.rs` — system ordering
