# turret-behavior-system

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Implement the turret behavior system as defined in `artifacts/designer/design/control_system.md` under TurretCommandState, TurretBehaviorState, TurretBehavior, and TurretAutonomousScanning.

**TurretCommandState:**
Defines the turret's current objective. Set by the base behavior (NOT directly by commands). When base behavior identifies a target, it updates TurretCommandState.LockedTarget. When base behavior doesn't specify a target, turret retains previous TurretCommandState. If no assigned target, turret falls back to autonomous scanning.
- LockedTarget: ObjectInstance | None

**TurretBehaviorState:**
Internal working data (scan state, last known target position, etc.).

**TurretBehavior:**
When turret has a locked target: engage that target (rotate toward it, fire when aligned). When turret has no locked target: autonomously scan for targets.

**TurretAutonomousScanning:**
When LockedTarget=None, turret selects the best available target from ValidTarget enemies within weapon Range and TurnAngle arc.

**Target Selection Priority:**
1. **Threatening units first**: Prefer ValidTargets whose AttackAttributes.TargetDomain includes this unit's domain. These are units capable of attacking THIS unit.
2. **Least rotation**: Among equal-priority targets, prefer the one requiring least turret rotation from current facing.
3. **Closest distance**: Among equal-rotation targets, prefer the closest.

**Algorithm:**
1. Gather all ValidTarget enemies within Range and within TurnAngle arc
2. Partition into threatening (can target this unit's domain) and non-threatening
3. From highest-priority non-empty partition, select target with least turret rotation, break ties by closest distance
4. Set TurretCommandState.LockedTarget to selected target
5. If no valid targets: TurretAttackChannel = TurretInactive

## QA Instructions

1. Select a turret unit and place it near enemies — verify turret autonomously rotates toward and fires at enemies without commands.
2. Verify turret prefers threatening targets (e.g., if a ground turret unit sees both a ground attacker and an air-only attacker, it prioritizes the ground attacker that can fire back).
3. Among equally threatening targets, verify turret picks the one requiring least rotation.
4. Among equal-rotation targets, verify turret picks the closest.
5. Issue an Attack command — verify base behavior sets TurretCommandState.LockedTarget, overriding autonomous scanning.
6. Issue a Move command — verify turret continues scanning autonomously while base moves (TurretCommandState.LockedTarget not set by base during Move).
7. Kill the locked target — verify turret falls back to autonomous scanning for next target.
8. Move turret unit away from all enemies — verify TurretAttackChannel goes to TurretInactive.
9. Verify turret only considers targets within its TurnAngle arc (not behind it if arc < 360).
