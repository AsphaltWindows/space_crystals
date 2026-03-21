# action-channel-attack-integration

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-action-channels.md

## Task

Integrate BaseAttackChannel and TurretAttackChannel/TurretOrientationChannel with the combat systems. Currently the combat module uses its own `AttackState`/`AttackPhase` enum (in `game/combat/types.rs`) and does NOT write to the action channel components. The turret systems similarly don't use `TurretAttackChannel` or `TurretOrientationChannel`.

**What to implement:**

1. **BaseAttackChannel integration**: The attack phase system (`attack_phase_system` in `game/combat/systems/core.rs`) should write to `BaseAttackChannel` as it transitions through phases:
   - `AttackPhase::Aiming` → `BaseAttackChannel::Aiming(target_entity)`
   - `AttackPhase::Firing` → `BaseAttackChannel::Firing(target_entity)`
   - `AttackPhase::Cooldown` → `BaseAttackChannel::Cooldown`
   - `AttackPhase::Reloading` → `BaseAttackChannel::Reloading`
   - `AttackPhase::None` → `BaseAttackChannel::None`
   - For turret units: `BaseAttackChannel` must always remain `None`

2. **Cross-channel constraint enforcement**: When `BaseAttackChannel` is active (non-turret units):
   - `Aiming`: Override `OrientationChannel` to `Turning(target_pos)`, enforce `LocomotionChannel::Stationary`
   - `Firing`/`Cooldown`: Enforce `LocomotionChannel::Stationary`, lock `OrientationChannel::Maintaining`
   - `Reloading`: No constraints — locomotion and orientation free
   This replaces the current `base_action_constraints()` pattern in the movement systems.

3. **TurretAttackChannel integration**: The turret attack/scanning systems should write to `TurretAttackChannel`:
   - Turret aiming → `TurretAttackChannel::Aiming(target_entity)`
   - Turret firing → `TurretAttackChannel::Firing(target_entity)`
   - Turret cooldown → `TurretAttackChannel::Cooldown`
   - Turret reloading → `TurretAttackChannel::Reloading`
   - No target → `TurretAttackChannel::Inactive`

4. **TurretOrientationChannel integration**: The turret rotation system (`turret_rotation_system` in `game/combat/turret.rs`) should read from `TurretOrientationChannel`:
   - `TurretTurning(target_pos)`: rotate turret toward target, constrained by `TurnAngle`
   - `TurretMaintaining`: hold current turret facing
   - When `TurretAttackChannel::Aiming(target)`, override `TurretOrientationChannel` to face the target

5. **Interruptibility**: When a non-turret unit receives a new command during `BaseAttackChannel::Aiming` or `Reloading`, the attack sequence cancels (channel resets to None). During `Firing` or `Cooldown`, commands are queued/rejected.

**Key files:**
- Channel definitions: `game/units/types/state/behavior.rs`
- AttackState/AttackPhase: `game/combat/types.rs` (~line 95+)
- Attack phase system: `game/combat/systems/core.rs`
- Turret rotation: `game/combat/turret.rs`
- Turret scanning: `game/combat/turret.rs` (`turret_autonomous_scanning_system`)
- Combat plugin registration: `game/combat/mod.rs`

**Design reference:** `artifacts/designer/design/control_system.md` — BaseAttackChannel, TurretActionChannels sections.
