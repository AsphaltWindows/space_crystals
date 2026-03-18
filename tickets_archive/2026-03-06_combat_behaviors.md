# Ticket: Combat Behaviors

## Current State
No combat behavior implementations exist. Units cannot engage targets, patrol, or hold position in response to commands.

## Desired State
Implement 5 combat-related base behaviors:

**AttackingObject**: Move toward target, engage when in range and arc. Infantry: stop and attack via BaseAttackChannel. Turret units: set TurretCommandState.LockedTarget and let turret channels handle firing. Resume movement if target moves out of range. Glider: performs strafing runs instead of stopping. Complete when target is destroyed.

**AttackingLocation**: Like AttackingObject but targets a ground location. CanTargetGround units only. Complete after the attack effect is applied. Glider: strafing runs.

**AttackMovingToLocation**: Move along path, scan for ValidTarget enemies within SightRange. When enemy found, engage via AttackingObject sub-behavior. Leash back to path if perpendicular distance from path exceeds AttackMoveLeashDistance (6 grid units). Glider: strafing runs along path.

**Patrolling**: Cycles AttackMovingToLocation between the origin position and the commanded destination. Never completes on its own — continuously patrols back and forth.

**HoldingPosition**: Unit remains stationary always. Turret units: autonomous scanning continues normally. CanTurnInPlace infantry: rotate toward enemies and engage via BaseAttackChannel. Non-turning infantry: engage only targets in current facing arc. Never completes.

## Justification
Defined in `features/unit_commands_and_behaviors.md` section: Base Behaviors (AttackingObject, AttackingLocation, AttackMovingToLocation, Patrolling, HoldingPosition). These behaviors depend on `features/combat_system.md` for ValidTarget filtering and attack phases, and `features/unit_system.md` for UnitBase properties (CanTargetGround, CanTurnInPlace, Glider movement).

## QA Steps
1. Issue Attack on an enemy unit in range. Verify infantry stops and engages via BaseAttackChannel. Verify turret unit sets TurretCommandState.LockedTarget.
2. Move the enemy out of range. Verify the attacking unit resumes movement to chase.
3. Destroy the target. Verify AttackingObject completes.
4. Issue Attack with a Glider. Verify it performs strafing runs rather than stopping.
5. Issue AttackGround on a location with a CanTargetGround unit. Verify it moves to range, fires at the location, and completes after the attack effect.
6. Attempt AttackGround with a non-CanTargetGround unit. Verify rejection at command level.
7. Issue AttackMove to a location. Verify the unit moves along the path and engages enemies that enter SightRange.
8. Lure the unit 7+ grid units perpendicular from its path during AttackMove. Verify it leashes back (disengages and returns to path).
9. Issue Patrol between two points. Verify the unit continuously cycles between them, engaging enemies along the way. Verify it never completes.
10. Issue HoldPosition with a turret unit. Verify it stays stationary and turret autonomously scans/engages.
11. Issue HoldPosition with CanTurnInPlace infantry. Verify it rotates toward nearby enemies and engages.
12. Issue HoldPosition with non-turning infantry. Verify it only engages targets in its current facing arc.

## Expected Experience
Combat behaviors produce visible engagement: units chase targets, fire at ground, patrol routes while fighting, and hold positions with appropriate engagement rules. Gliders strafe instead of hovering. The 6gu leash on AttackMove prevents units from being lured away indefinitely.
