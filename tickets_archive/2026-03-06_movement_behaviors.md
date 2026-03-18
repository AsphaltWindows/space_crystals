# Ticket: Movement Behaviors

## Current State
No behavior implementations exist. Units cannot pathfind, move, reverse, or stop in response to commands.

## Desired State
Implement 4 movement-related base behaviors:

**MovingToLocation**: Pathfind to TargetLocation, execute a plan of (Locomotion, Orientation) action channel pairs. Recompute path on deviation from plan. End condition: set Locomotion=Stopping, wait until stopped, then mark behavior complete. Glider exception: circles over target indefinitely instead of stopping.

**MovingToObject**: Like MovingToLocation but the target is a moving ObjectInstance. Recompute path when target moves sufficiently. Complete when unit is within proximity of target.

**ReversingToLocation**: Like MovingToLocation but uses Reversing locomotion instead of Moving. Only available to CanReverse UnitBases.

**StoppingBehavior**: Set Locomotion=Stopping, Orientation=Maintaining. Clear TurretCommandState.LockedTarget. Complete when unit has fully stopped.

Each behavior writes to BaseBehaviorState for its internal data (paths, progress) and outputs to the base action channels each tick.

## Justification
Defined in `features/unit_commands_and_behaviors.md` section: Base Behaviors (MovingToLocation, MovingToObject, ReversingToLocation, StoppingBehavior). These behaviors are triggered by the Move, Reverse, and Stop commands respectively.

## QA Steps
1. Issue Move to a location. Verify the unit pathfinds and moves toward the target, with LocomotionChannel=Moving and OrientationChannel=Turning as appropriate.
2. Block the path mid-movement. Verify the unit recomputes its path.
3. Verify the unit sets Locomotion=Stopping when near the target and marks behavior complete once stopped.
4. Issue Move to a location with a Glider unit. Verify it arrives and circles indefinitely without completing.
5. Issue Move targeting a moving ObjectInstance (e.g., right-click on ally). Verify MovingToObject tracks the target and recomputes as the target moves. Verify completion when within proximity.
6. Issue Reverse to a CanReverse unit. Verify Locomotion=Reversing is used instead of Moving.
7. Issue Reverse to a non-CanReverse unit. Verify it is rejected at the command level (not the behavior level).
8. Issue Stop. Verify StoppingBehavior sets Locomotion=Stopping, Orientation=Maintaining, clears TurretCommandState.LockedTarget, and completes when stopped.

## Expected Experience
Units pathfind and move smoothly to commanded locations or objects. Gliders circle rather than stop. Reversing units back up. Stop halts all action. Paths are recomputed dynamically when obstructed or when tracking a moving target.
