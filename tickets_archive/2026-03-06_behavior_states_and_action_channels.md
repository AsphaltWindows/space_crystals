# Ticket: Behavior States and Action Channels

## Current State
No behavior state or action channel infrastructure exists. There is no mechanism for translating command state into concurrent unit actions.

## Desired State
Implement BaseBehaviorState[UnitBase] — internal data for behavior execution (planned paths, progress, cached info), parameterized by UnitBase since different bases need different internal data.

Implement TurretBehaviorState — internal data for turret behavior (scan state, last known target position).

Implement 3 base action channels that run concurrently:
- **LocomotionChannel**: Moving(path) | Reversing(path) | Stopping | Stationary
- **OrientationChannel**: Turning(targetPosition) | Maintaining
- **BaseAttackChannel** (infantry only): BaseAiming(target) | BaseFiring(target) | BaseCooldown | BaseReloading | None

Implement 2 turret action channels (turret units only) that run concurrently:
- **TurretOrientationChannel**: TurretTurning(targetPosition) | TurretMaintaining
- **TurretAttackChannel**: TurretAiming(target) | TurretFiring(target) | TurretCooldown | TurretReloading | TurretInactive

Action channels are the output layer — behaviors write to channels each tick, and the channels drive the actual unit animation/movement/attack systems.

## Justification
Defined in `features/unit_commands_and_behaviors.md` sections: BaseBehaviorState, TurretBehaviorState, Base Action Channels, Turret Action Channels. These are the bridge between the command/behavior logic and the unit_system (`features/unit_system.md`) movement models and combat_system (`features/combat_system.md`) attack phases.

## QA Steps
1. Create a UnitInstance with a ground infantry UnitBase. Verify it has BaseBehaviorState, LocomotionChannel, OrientationChannel, and BaseAttackChannel.
2. Create a UnitInstance with a turret-bearing UnitBase. Verify it additionally has TurretBehaviorState, TurretOrientationChannel, and TurretAttackChannel.
3. Set LocomotionChannel to Moving(path) and OrientationChannel to Turning(position) simultaneously. Verify both channels can hold independent states concurrently.
4. Set BaseAttackChannel on a non-infantry unit. Verify it is not present or always None.
5. Set TurretAttackChannel to TurretInactive on a turret unit. Verify this is a valid state.
6. Verify BaseBehaviorState can store UnitBase-specific data (e.g., a Glider's circling state vs. infantry path state).

## Expected Experience
Each unit carries the appropriate set of action channels based on its UnitBase properties. Channels hold independent states and can be read by downstream systems (movement, animation, combat) each tick.
