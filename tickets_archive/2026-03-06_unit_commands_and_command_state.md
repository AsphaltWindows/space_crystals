# Ticket: Unit Commands and Command State

## Current State
No command system exists. Units have no way to receive or queue player-issued commands.

## Desired State
Implement 8 unit commands (Move, Attack, AttackGround, AttackMove, Patrol, HoldPosition, Stop, Reverse) with their parameter signatures as defined in the command table. Each command sets BaseCommandState with CommandType, TargetLocation (Coordinates | None), and TargetObject (ObjectInstance | None).

Implement command queue on UnitInstance (shift-click queuing). When a command is dequeued and executed, it sets BaseCommandState.

Implement TurretCommandState with LockedTarget (ObjectInstance | None). TurretCommandState is set by base behaviors, not directly by commands. When LockedTarget is None, turret falls back to autonomous scanning.

Each command has an Availability constraint:
- Move, Patrol, HoldPosition, Stop: All units
- Attack, AttackMove: All attacking units
- AttackGround: CanTargetGround units only
- Reverse: CanReverse bases only

## Justification
Defined in `features/unit_commands_and_behaviors.md` sections: Unit Commands, BaseCommandState, TurretCommandState. Commands originate from CommandIssuingTransitions in the control system (`features/control_system.md`).

## QA Steps
1. Create a UnitInstance with a command queue.
2. Issue a Move command with a TargetLocation. Verify BaseCommandState is set with CommandType=Move and the correct TargetLocation.
3. Issue multiple commands with shift held. Verify they queue and execute in FIFO order.
4. Issue an Attack command targeting a destructible ObjectInstance. Verify BaseCommandState sets CommandType=Attack and TargetObject.
5. Attempt to issue Reverse to a unit whose UnitBase does not have CanReverse. Verify it is rejected.
6. Attempt to issue AttackGround to a unit without CanTargetGround. Verify it is rejected.
7. Issue a Stop command. Verify BaseCommandState is set with CommandType=Stop and both targets are None.
8. Verify TurretCommandState.LockedTarget initializes to None.

## Expected Experience
Commands are accepted or rejected based on unit capability. BaseCommandState reflects the currently executing command. The command queue drains in order as each command completes. TurretCommandState is not directly modified by player commands.
