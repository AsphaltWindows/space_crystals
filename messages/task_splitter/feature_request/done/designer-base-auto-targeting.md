# base-auto-targeting

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Implement BaseAutoTargeting as defined in `artifacts/designer/design/control_system.md`.

**BaseAutoTargeting:**
When a unit is idle (no active command) or executing HoldPosition, the base autonomously scans for and engages ValidTarget enemies.

**Target Selection Priority** (same as TurretAutonomousScanning):
1. Threatening targets first (those that can attack this unit's domain)
2. Least rotation (unit facing for infantry, turret angle for turret units)
3. Closest distance

**Active during Idle:**
- When command queue is empty, unit scans for ValidTargets within SightRange
- On acquiring a target: switch to AttackingObject sub-behavior targeting the enemy
- Record unit's position as IdleOrigin
- If unit strays more than **IdleLeashDistance (4 grid units)** from IdleOrigin: disengage and return to IdleOrigin via MovingToLocation
- If target destroyed or leaves SightRange: return to idle at current position (new IdleOrigin)

**Active during HoldPosition:**
- Same scanning, but Locomotion locked to Stationary (unit never chases)
- Turret units: turret engages within range and arc
- Non-turret with CanTurnInPlace: Orientation = Turning(enemy position), engage via BaseAttackChannel
- Non-turret without CanTurnInPlace: only engage targets already in current facing and range
- Target leaves range: disengage immediately

**NOT active during:**
- Move (pure movement, turret autonomous scanning still works independently)
- AttackTarget (already has explicit target)
- AttackGround (deliberate location attack, don't override)
- Stop (halting all activity)

**NOT applicable to:**
- AttackMove, Patrol (these have their own scanning logic)

## QA Instructions

1. Leave a unit idle near an enemy — verify it auto-acquires and attacks the enemy.
2. Let the idle unit chase an enemy — verify it disengages and returns when it exceeds 4 grid units from where it started.
3. Destroy the auto-target — verify unit returns to idle at its current position.
4. Order HoldPosition near enemies — verify unit fires at enemies but never moves.
5. Order HoldPosition on infantry without CanTurnInPlace — verify it only engages enemies in its current facing direction.
6. Order Move past enemies — verify the base does NOT auto-engage (turret still fires independently on turret units).
7. Order Stop — verify unit does not auto-engage anything.
8. Verify auto-targeting prefers threatening enemies (those that can attack back).
