# action_channels_r1

## Metadata
- **From**: manual_qa
- **To**: task_splitter

## Content

The action channels feature was partially implemented but targeted attack and target lock during Attack-Move are broken, preventing verification of the attack phase state machine.

**What passed:**
- Unit stops moving when engaging a target during Attack-Move
- Unit faces toward the target while shooting
- Move command successfully interrupts attack
- Damage is being dealt

**What failed:**
1. **Targeted attack broken**: Using Attack command + left-click on an enemy unit, or right-clicking an enemy unit, causes the attacking unit to simply stop in place. It does not pursue or attack the target.
2. **Attack-Move doesn't maintain target lock**: When a unit engages an enemy during Attack-Move, it fires once then resumes moving instead of staying locked on the target until it's dead or out of range. Expected: unit should keep attacking the acquired target until destroyed or out of range, then resume the Attack-Move.
3. **Turret units not available**: Steps 7-10 (turret-specific action channels) could not be tested as no turret units exist yet. This is not a rework item — just noting for future QA.

## QA Instructions

1. Select an infantry unit, press Attack, then left-click on an enemy unit. Verify the unit moves toward the enemy and begins attacking it (Aiming -> Firing -> Cooldown -> Reloading cycle).
2. Right-click on an enemy unit. Verify the unit moves toward and attacks the enemy.
3. Use Attack-Move (Attack + click ground) to send a unit near enemies. When it engages a target, verify it stays locked on that target and keeps attacking until the target is dead or moves out of range, before resuming the Attack-Move path.
4. During an active attack on a locked target, verify the unit transitions through distinct phases: Aiming (stationary, facing target) -> Firing (stationary, locked orientation) -> Cooldown (stationary) -> Reloading (can move/turn).
