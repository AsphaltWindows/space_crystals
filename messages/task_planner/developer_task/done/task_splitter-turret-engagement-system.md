# turret-engagement-system

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-turret-behavior-system.md

## Task

Implement a new `turret_engagement_system` that reads `TurretCommandState.locked_target` and drives turret rotation and firing via the turret action channels.

**System behavior:**
1. For each turret unit with `TurretCommandState`:
   - If `locked_target = None`: set `TurretAttackChannel::Inactive` and `TurretOrientationChannel::Maintaining`
   - If `locked_target = Some(target)`:
     a. Validate target still exists (if despawned, clear locked_target, set Inactive)
     b. Compute turret rotation needed toward target position
     c. Set `TurretOrientationChannel::Turning(target_position)`
     d. If turret is aligned with target (within tolerance): set `TurretAttackChannel::Aiming(target)` or `Firing(target)` based on attack phase
     e. If target moves out of weapon Range or TurnAngle arc: clear `locked_target` (allow re-scan)

2. Use the existing `compute_relative_turret_angle` helper and `Turret::can_reach_angle` for arc checks

3. The actual turret rotation (applying angular velocity) and attack execution (damage dealing) should integrate with or reuse existing turret rotation logic and the `attack_phase_system` — this system's job is to SET the channel values that those systems consume

**Key files:**
- `game/combat/systems/core.rs` — add new system, existing turret helpers (compute_relative_turret_angle)
- `game/combat/mod.rs` — register the new system
- `game/units/types/state/behavior.rs` — TurretOrientationChannel, TurretAttackChannel enums
- `game/units/types/state/commands.rs` — TurretCommandState

**Components queried:** Transform, Turret, AttackCapability, TurretCommandState, TurretBehaviorState, TurretOrientationChannel, TurretAttackChannel, Owner, GridPosition
