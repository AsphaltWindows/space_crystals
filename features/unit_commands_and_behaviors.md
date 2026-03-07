# Feature: Unit Commands and Behaviors

## Overview
The command-to-behavior pipeline: unit commands set BaseCommandState, which drives base and turret behaviors through action channels.

## Design Sources
- `design/control_system.md` (Unit Commands, States, Behaviors, Action Channels)

## Specifications

### Unit Commands
Commands are issued via CommandIssuingTransitions from the Control System. Units maintain a command queue (shift-click queuing). Commands only mutate BaseCommandState; TurretCommandState is managed by behaviors.

| Command | Sets CommandType | TargetLocation | TargetObject | Availability |
|---------|-----------------|----------------|--------------|-------------|
| Move | Move | location or None | ObjectInstance or None | All units |
| Attack | Attack | None | Destructible ObjectInstance | All attacking units |
| AttackGround | AttackGround | location | None | CanTargetGround units |
| AttackMove | AttackMove | location | None | All attacking units |
| Patrol | Patrol | location | None | All units |
| HoldPosition | HoldPosition | None | None | All units |
| Stop | Stop | None | None | All units |
| Reverse | Reverse | location | None | CanReverse bases only |
| Enter | Enter | None | Tunnel (ObjectInstance) | Syndicate units, tier sufficient |
| Gather | Gather | None | Resource source (ObjectInstance) | Agent (resource-gathering units) |
| DropOffResources | DropOffResources | None | Own Tunnel (ObjectInstance) | Agent (when carrying resources) |
| BuildTunnel | BuildTunnel | location | None | Agent (via AwaitingPlacement) |

### BaseCommandState[UnitBase]
- CommandType, TargetLocation (Coordinates | None), TargetObject (ObjectInstance | None)
- Set when a command is dequeued and executed

### BaseBehaviorState[UnitBase]
- Internal data for behavior execution (planned paths, progress, cached info)
- Parameterized by UnitBase (different bases need different internal data)

### TurretCommandState
- LockedTarget: ObjectInstance | None
- Set by base behavior, not directly by commands
- When None, turret falls back to autonomous scanning

### TurretBehaviorState
- Internal data for turret behavior (scan state, last known target position)

### Base Action Channels (3 concurrent)

**LocomotionChannel**: Moving(path) | Reversing(path) | Stopping | Stationary
**OrientationChannel**: Turning(targetPosition) | Maintaining
**BaseAttackChannel** (infantry only): BaseAiming(target) | BaseFiring(target) | BaseCooldown | BaseReloading | None

### Turret Action Channels (2 concurrent, turret units only)

**TurretOrientationChannel**: TurretTurning(targetPosition) | TurretMaintaining
**TurretAttackChannel**: TurretAiming(target) | TurretFiring(target) | TurretCooldown | TurretReloading | TurretInactive

### Base Behaviors

**MovingToLocation**: Pathfind to TargetLocation, execute plan of (Locomotion, Orientation) pairs. Recompute on deviation. End: Stopping -> stopped -> complete. Glider: circles over target indefinitely.

**MovingToObject**: Like MovingToLocation but target moves. Recompute when target moves sufficiently. Complete when within proximity.

**ReversingToLocation**: Like MovingToLocation with Reversing locomotion. CanReverse bases only.

**AttackingObject**: Move toward target, engage when in range/arc. Infantry: stop and attack via BaseAttackChannel. Turret units: set TurretCommandState.LockedTarget. Resume movement if target moves out of range. Glider: strafing runs. Complete when target destroyed.

**AttackingLocation**: Like AttackingObject for ground locations. CanTargetGround only. Complete after attack effect applied. Glider: strafing runs.

**AttackMovingToLocation**: Move along path, scan for ValidTarget enemies within SightRange. Engage via AttackingObject sub-behavior. Leash back to path if perpendicular distance > AttackMoveLeashDistance (6 grid units). Glider: strafing runs along path.

**Patrolling**: Cycles AttackMovingToLocation between origin and destination. Never completes on its own.

**HoldingPosition**: Stationary always. Turret units: autonomous scanning continues. CanTurnInPlace infantry: rotate and engage via BaseAttackChannel. Non-turning infantry: engage only what's in current facing. Never completes.

**StoppingBehavior**: Locomotion=Stopping, Orientation=Maintaining. Clear TurretCommandState.LockedTarget. Complete when stopped.

**EnteringTunnel**: Unit moves to the target Tunnel's Side A (MovingToObject sub-behavior targeting Side A position), then enters the Tunnel Network. The unit is removed from the map and added to the Tunnel Network's unit pool. Only available to Syndicate units when the target Tunnel's tier meets the unit's base category transit requirement. No explicit behavior algorithm defined in design — intent is clear from Enter command description.

**GatheringResource**: Agent moves to the target resource source, performs mining/pickup (MiningDuration for crystals, PickUpDuration for supplies), then automatically moves to the nearest own Tunnel's appropriate side (Side B for crystals, Side C for supplies) to drop off (DropOffDuration). Behavior details inferred from Agent gathering stats — no formal behavior algorithm in design.

**DroppingOffResources**: Agent moves to the target Tunnel's appropriate side (Side B for crystals, Side C for supplies based on carried resource type) and performs drop-off (DropOffDuration). Auto-routing to correct side is handled by the behavior, not the player.

**BuildingTunnel**: Agent moves to the target build location, then begins construction. Agent embeds inside the partially-built Tunnel and becomes untargetable. Construction takes 480 frames. Only one Agent may construct a given Tunnel. See `syndicate_objects` Agent Building section for full construction flow.

### TurretAutonomousScanning
When turret has no locked target, autonomously selects best target from ValidTarget enemies within Range and TurnAngle arc.

**Priority**: (1) Threatening units first (can target this unit's domain), (2) least turret rotation, (3) closest distance.

### BaseAutoTargeting
Active during: **Idle** (with chase leash of 4 grid units from IdleOrigin) and **HoldPosition** (stationary engagement only).
NOT active during: Move, AttackTarget, AttackGround, Stop.
NOT applicable to: AttackMove, Patrol (have own scanning logic).

During Move: turret autonomous scanning still operates independently for turret units, but base does not acquire targets.

## Dependencies
- `control_system` (commands originate from interface transitions)
- `combat_system` (ValidTarget, attack phases, attack sources)
- `unit_system` (UnitBase properties, movement models, locomotion constraints)
