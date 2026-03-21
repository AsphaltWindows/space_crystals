# base-behaviors

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Implement all base behaviors as defined in `artifacts/designer/design/control_system.md`.

**MovingToLocation:**
Move to a static target location. Computes path, translates to (Locomotion, Orientation) plan respecting LocomotionOrientationConstraints. Each tick: recompute if position deviates from expected. Final step: Stopping + Maintaining. GliderMovement exception: circles over target instead of stopping. Failure: if no progress, stop and complete.

**MovingToObject:**
Move to a mobile/static object. Same as MovingToLocation but target position updates when object moves, triggering replanning. Complete when within proximity of target.

**AttackingObject:**
Attack a destructible object. MovingToObject toward target; when in Range, beyond MinRange, and within arc: stop and engage. Infantry: BaseAttackChannel. Turret units: set TurretCommandState.LockedTarget. If target moves out of range: resume MovingToObject. Target destroyed: complete. GliderMovement: strafing runs (fly toward, fire while passing, loop around).

**AttackingLocation:**
Attack ground location (CanTargetGround only). Same as AttackingObject but static target, uses MovingToLocation. Complete after attack effect applied. Glider: strafing runs on location.

**ReversingToLocation:**
Reverse to location (CanReverse only). Identical to MovingToLocation but uses Reversing locomotion. Not applicable to Gliders.

**AttackMovingToLocation:**
Move to location while engaging enemies. Scans for ValidTargets within SightRange while moving. If enemy detected: switch to AttackingObject sub-behavior. If distance from path exceeds AttackMoveLeashDistance (6 grid units): disengage, resume path. Target destroyed: resume path. Glider: strafing runs while following path.

**Patrolling:**
Move back and forth between origin and destination while engaging enemies. Uses AttackMovingToLocation for each leg. Records PatrolOrigin at command time. When a leg completes: swap and repeat. Never completes on its own.

**HoldingPosition:**
Remain stationary. Turret units: turret continues autonomous scanning. Non-turret with CanTurnInPlace: scan for enemies, turn and engage via BaseAttackChannel. Non-turret without CanTurnInPlace: only engage targets in current facing. Never completes.

**StoppingBehavior:**
Cease all activity. Locomotion=Stopping, Orientation=Maintaining. Clear TurretCommandState.LockedTarget. When stopped: complete, unit has no active behavior.

## QA Instructions

1. Order Move to a location — verify unit pathfinds and stops at destination.
2. Order Move to a moving enemy — verify unit tracks and follows the target.
3. Order Attack on enemy — verify unit approaches, stops at weapon range, and fires. If enemy runs, unit chases.
4. Order AttackMove — verify unit moves toward destination but stops to fight enemies within SightRange. After killing or chasing too far (6+ grid units from path), resumes toward destination.
5. Order Patrol — verify unit walks back and forth between start point and target, fighting enemies along the way. Verify it never stops on its own.
6. Order HoldPosition on a turret unit — verify unit stays put, turret scans and fires at enemies in range.
7. Order HoldPosition on infantry — verify unit turns to face enemies and fires but never moves.
8. Order Stop — verify unit halts immediately and goes idle.
9. Order Reverse (CanReverse unit) — verify unit drives backward to target location.
10. Test Glider behaviors — verify it never stops, performs strafing runs on attack targets, circles when idle at destination.
