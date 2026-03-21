# action-channels

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Implement the base and turret action channel systems as defined in `artifacts/designer/design/control_system.md`.

**BaseActionChannels** — 3 concurrent channels per unit base:

1. **LocomotionChannel** (LocomotionEnum):
   - Moving: base moving along a path from BehaviorState
   - Reversing: base moving backward (CanReverse only)
   - Stopping: base slowing down to a stop
   - Stationary: base not moving

2. **OrientationChannel** (OrientationEnum):
   - Turning(TargetPosition): base rotating toward target position. Engine computes angle, applies base turn rate each tick. FixedTurnRadiusMovement: constrained by locomotion.
   - Maintaining: base holding current facing

3. **BaseAttackChannel** (BaseAttackEnum | None):
   Only active for non-turret units. Always None for turret units.
   - BaseAiming(Target): aiming at target. Orientation overridden to face target. Locomotion must be Stationary.
   - BaseFiring(Target): firing phase. Locomotion Stationary, Orientation locked.
   - BaseCooldown: cooldown after firing. Locomotion Stationary, Orientation locked.
   - BaseReloading: reloading between attacks. Locomotion and Orientation free.

**TurretActionChannels** — 2 concurrent channels (only on turret units):

1. **TurretOrientationChannel** (TurretOrientationEnum):
   - TurretTurning(TargetPosition): turret rotating toward target, constrained by TurnAngle
   - TurretMaintaining: turret holding current facing

2. **TurretAttackChannel** (TurretAttackEnum):
   - TurretAiming(Target): turret aiming, TurretOrientation overridden to face target
   - TurretFiring(Target): turret firing
   - TurretCooldown: turret cooldown after firing
   - TurretReloading: turret reloading
   - TurretInactive: turret not attacking

## QA Instructions

1. Order an infantry unit (non-turret) to attack — verify BaseAttackChannel transitions through Aiming -> Firing -> Cooldown -> Reloading.
2. During BaseAiming: verify unit is Stationary and Orientation is toward the target.
3. During BaseFiring: verify unit cannot move or turn.
4. During BaseReloading: verify unit can move and turn freely.
5. Interrupt during BaseAiming (issue Move command) — verify attack sequence cancels.
6. Attempt to interrupt during BaseFiring — verify it cannot be interrupted.
7. For a turret unit: verify TurretOrientationChannel rotates independently of base OrientationChannel.
8. Verify turret respects TurnAngle limit — cannot rotate beyond its arc.
9. Order a turret unit to Move — verify base moves (LocomotionChannel = Moving) while turret independently aims and fires (TurretAttackChannel cycles).
10. Verify that for turret units, BaseAttackChannel is always None.
