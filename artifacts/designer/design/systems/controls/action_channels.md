# Controls: Action Channels

See also: `systems/controls/behaviors.md`, `systems/controls/command_behaviors.md`, `systems/controls/action_channels.md` (split from `behaviors.md`, 2026-11-27).

## BaseActionChannels
The unit base operates on three concurrent action channels. Each channel has one active value at a time. Valid combinations of channel states are constrained by UnitBase and current attack phase.

## LocomotionChannel
Controls the unit base's movement.

### Value - LocomotionEnum

## Moving
The unit base is moving along a path from BehaviorState.

### Path - from BaseBehaviorState

## Reversing
The unit base is moving backward along a path. Only available to bases with CanReverse = true (WheeledVehicle, TrackedVehicle, DrillUnit).

### Path - from BaseBehaviorState

## Stopping
The unit base is slowing down to a stop.

## Stationary
The unit base is not moving.

## OrientationChannel
Controls the unit base's facing direction.

### Value - OrientationEnum

## Turning
The unit base is rotating toward a target position. The engine computes the required angle and applies the base's turn rate each tick. For FixedTurnRadiusMovement bases, orientation is constrained by locomotion.

### TargetPosition - Coordinates

## Maintaining
The unit base is holding its current facing direction.

## BaseAttackChannel
Controls the unit base's attack state. Only active for non-turret units. For turret units this channel is always None.

### Value - BaseAttackEnum | None

## BaseAiming
The unit base is aiming at an attack target. Orientation channel is overridden to face the target. Locomotion must be Stationary. If the unit is given a different command, the attack sequence is interrupted.

### Target - AttackTarget

## BaseFiring
The unit base is executing the firing phase. Locomotion must be Stationary. Orientation is locked.

### Target - AttackTarget

## BaseCooldown
The unit base is in the cooldown phase after firing. Locomotion must be Stationary. Orientation is locked.

## BaseReloading
The unit base is in the reloading phase between attacks. Locomotion and Orientation are free.

## TurretActionChannels
The turret operates on two concurrent action channels, running independently of the base action channels. Only present on turret-bearing units.

## TurretOrientationChannel
Controls the turret's facing direction, independent of the unit base's orientation.

### Value - TurretOrientationEnum

## TurretTurning
The turret is rotating toward a target position. The engine computes the required angle and applies the turret's turn rate each tick, constrained by TurnAngle.

### TargetPosition - Coordinates

## TurretMaintaining
The turret is holding its current facing direction.

## TurretAttackChannel
Controls the turret's attack state.

### Value - TurretAttackEnum

## TurretAiming
The turret is aiming at an attack target. TurretOrientation is overridden to face the target.

### Target - AttackTarget

## TurretFiring
The turret is executing the firing phase of its attack sequence.

### Target - AttackTarget

## TurretCooldown
The turret is in the cooldown phase after firing.

## TurretReloading
The turret is in the reloading phase between attacks.

## TurretInactive
The turret is not performing any attack action.
