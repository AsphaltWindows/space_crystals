# Controls: Unit Commands

## Unit Command
An order issued to a unit, produced by a CommandIssuingTransition from the Control System. A unit maintains a command queue; the current command is dequeued and sets the BaseCommandState, which drives the unit's behavior. When the current command's behavior completes, the next command is dequeued. Commands are queued by holding **Shift** while issuing them (right-click, command hotkey + target click, etc.): the new command is appended to the end of the queue instead of replacing it. Without Shift, issuing a command replaces the whole queue (user, 2026-11-23). Commands only mutate BaseCommandState — TurretCommandState is managed by the base behavior.

### Value - UnitCommandEnum

## Move
Order the unit to move to a target location or object.

### Sets BaseCommandState:
- CommandType = Move
- TargetLocation = location | None
- TargetObject = ObjectInstance | None (one of TargetLocation or TargetObject must be set)

## Attack
Order the unit to attack a specific destructible object instance (unit or structure).

### Sets BaseCommandState:
- CommandType = Attack
- TargetLocation = None
- TargetObject = Destructible ObjectInstance

## AttackGround
Order the unit to attack a specific ground location. Only available to units whose AttackType has CanTargetGround = true.

### Sets BaseCommandState:
- CommandType = AttackGround
- TargetLocation = location
- TargetObject = None

## AttackMove
Order the unit to move to a target location, engaging enemies encountered along the way.

### Sets BaseCommandState:
- CommandType = AttackMove
- TargetLocation = location
- TargetObject = None

## Patrol
Order the unit to move back and forth between its current location and a target location, engaging enemies encountered along the way.

### Sets BaseCommandState:
- CommandType = Patrol
- TargetLocation = location
- TargetObject = None

## HoldPosition
Order the unit to remain stationary. The unit will not move for any reason.

### Sets BaseCommandState:
- CommandType = HoldPosition
- TargetLocation = None
- TargetObject = None

## Stop
Order the unit to cease all current activity.

### Sets BaseCommandState:
- CommandType = Stop
- TargetLocation = None
- TargetObject = None

## Reverse
Order the unit to reverse to a target location. Only available to bases with CanReverse = true (WheeledVehicle, TrackedVehicle, DrillUnit).

### Sets BaseCommandState:
- CommandType = Reverse
- TargetLocation = location
- TargetObject = None

## Enter
Order the unit to enter an own structure that can hold units. The unit walks to the structure's entrance side and goes inside. Valid targets:

- **Syndicate — Tunnel**: any Syndicate unit, when the target Tunnel's tier is sufficient for the unit's base category. The unit walks to Side A and enters the Tunnel Network.
- **Cults — Altar**: any Cults infantry (Recruit, Soldier, Gunner, Acolyte), up to the Altar's InternalUnitCapacity.
- **Cults — Armory, Cathedral**: Recruits only (to be trained), up to the building's internal capacity.
- **Cults — Temple, Gunworks, Machine Shop**: Recruits only (to staff research).
- **Cults — Shrine**: Recruits only (to join the ritual).
- **Cults — Garrison**: Recruits, Soldiers, Gunners, and Acolytes (up to 4; Recruits and Acolytes fire as Soldiers; Soldiers and Gunners get +1 range).
- **GDO — Bunker**: all GDO infantry (up to 4; fire their own weapons, no range bonus).

Each faction's unit and building docs define the right-click resolution and capacity details (factions/syndicate/, factions/cults/).

### Sets BaseCommandState:
- CommandType = Enter
- TargetLocation = None
- TargetObject = Tunnel (ObjectInstance)

## BaseCommandState[UnitBase]
The parameters set by the currently executing command that define the unit's objective. Set when a command is dequeued and executed. Read by the base behavior to determine what the unit should be doing. Parameterized by UnitBase as different bases may require different command parameters.

### CommandType - UnitCommandEnum
### TargetLocation - Coordinates | None
### TargetObject - ObjectInstance | None

## TurretCommandState
The parameters that define the turret's current objective. Set by the base behavior (not directly by commands). When the base behavior identifies a target, it updates the TurretCommandState. When the base behavior does not specify a target, the turret retains its previous TurretCommandState. If the turret has no assigned target, it falls back to autonomous target scanning.

### LockedTarget - ObjectInstance | None
