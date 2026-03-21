# base-behaviors-verify

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-base-behaviors.md

## Task

Verify that all 9 base behaviors defined in control_system.md are fully implemented and registered. The following systems already exist and need verification against the design spec:

**In units/systems/behaviors.rs:**
1. `moving_to_location_system` — MovingToLocation (UnitCommand::Move, path following, Glider circling exception, Stopping on completion)
2. `moving_to_object_system` — MovingToObject (target tracking, re-pathing when target moves, proximity completion)
3. `reversing_to_location_system` — ReversingToLocation (Reversing locomotion, CanReverse only)
4. `stopping_behavior_system` — StoppingBehavior (Stopping+Maintaining, clears TurretCommandState.LockedTarget)

**In combat/systems/behaviors.rs:**
5. `attacking_object_behavior_system` — AttackingObject (approach via pathfinding, engage at range with elevation adjustment, infantry stops/turret continues/glider strafes, target destroyed=complete)
6. `attacking_location_behavior_system` — AttackingLocation (approach location, fire at ground, complete after one attack cycle)
7. `attack_move_behavior_system` — AttackMovingToLocation (move toward destination, scan SightRange for enemies, engage with AttackMoveLeashDistance=6gu, disengage and resume on leash exceeded or target destroyed)
8. `hold_position_behavior_system` — HoldingPosition (never moves, non-turret infantry scans for enemies in range, facing arc check for non-CanTurnInPlace)
9. `patrol_scanning_system` — Patrolling (enemy scanning during patrol, PatrolEngaged state save/restore, resume patrol on target destroyed)

**Verification checklist:**
- All 9 systems are registered in their respective plugins (units mod.rs and combat mod.rs)
- BaseBehaviorState has variants for all 5 movement models (TurnRate, FixedTurnRadius, SpeedTurnRadius, Drag, Glider) plus None
- Action channels (LocomotionChannel, OrientationChannel, BaseAttackChannel) are defined and used correctly
- Constants match spec: AttackMoveLeashDistance=6gu, IdleLeashDistance=4gu
- HoldingPosition component marker exists and is managed correctly by command systems
- PatrolEngaged component preserves patrol state during engagement
- Tests pass: `cargo test` in artifacts/developer/

If any behavior is incomplete or deviates from the spec in control_system.md, implement the fix. If all behaviors match the spec, confirm with a passing test run.
