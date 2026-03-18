# Ticket: Autonomous Targeting (Turret Scanning and Base Auto-Targeting)

## Current State
No autonomous target acquisition exists. Turrets without locked targets are idle. Units in idle or hold-position states do not self-engage nearby enemies.

## Desired State
Implement two autonomous targeting systems:

**TurretAutonomousScanning**: When TurretCommandState.LockedTarget is None, the turret autonomously selects the best target from ValidTarget enemies within weapon Range and TurnAngle arc. Priority order: (1) Threatening units first (those that can target this unit's domain), (2) least turret rotation required, (3) closest distance. Once a target is selected, the turret engages via TurretAttackChannel.

**BaseAutoTargeting**: Applies to non-turret attacking units (infantry). Active during:
- **Idle** state: unit acquires targets and chases within a 4 grid unit leash from IdleOrigin (the position where the unit became idle).
- **HoldPosition**: stationary engagement only (no chasing).

NOT active during: Move, AttackTarget (already has a target), AttackGround (already has a target), Stop.
NOT applicable to: AttackMove, Patrol (these behaviors have their own scanning logic built in).

During Move: turret autonomous scanning still operates independently for turret units, but base does not acquire targets.

## Justification
Defined in `features/unit_commands_and_behaviors.md` sections: TurretAutonomousScanning, BaseAutoTargeting. Depends on `features/combat_system.md` for ValidTarget filtering and attack range/arc calculations.

## QA Steps
1. Place a turret unit with no locked target near an enemy. Verify the turret autonomously selects and engages the enemy.
2. Place two enemies in turret range: one threatening (can attack this unit's domain) and one non-threatening. Verify the turret prioritizes the threatening one.
3. Place two equally threatening enemies: one requiring less turret rotation. Verify the turret picks the one needing less rotation.
4. Place two equally threatening enemies at equal rotation: one closer. Verify the turret picks the closer one.
5. Lock a turret target via behavior (e.g., Attack command). Verify autonomous scanning stops while LockedTarget is set.
6. Place an idle infantry unit near an enemy. Verify BaseAutoTargeting engages the enemy.
7. Lure the enemy 5+ grid units from the infantry's IdleOrigin. Verify the infantry leashes back (returns to IdleOrigin, does not chase beyond 4gu).
8. Issue HoldPosition to infantry near an enemy. Verify it engages without moving.
9. Issue Move to an infantry unit passing near enemies. Verify BaseAutoTargeting does NOT activate (unit keeps moving).
10. Issue Move to a turret unit passing near enemies. Verify TurretAutonomousScanning DOES activate (turret fires while moving) but base does not stop.

## Expected Experience
Turrets intelligently select targets when uncommanded, preferring threats and minimizing rotation. Idle units defend themselves within a leash radius. Hold-position units engage in place. Moving units ignore enemies at the base level but turrets still fire opportunistically.
