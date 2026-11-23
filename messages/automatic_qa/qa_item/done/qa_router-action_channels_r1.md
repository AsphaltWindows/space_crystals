# action_channels_r1

## Metadata
- **From**: qa_router
- **To**: automatic_qa

## Content

## Content

**Note from automatic_qa**: This item was routed for automated QA but cannot be verified automatically because ALL QA instructions require interactive gameplay — selecting units, clicking enemies, observing pursuit/attack behavior in real-time. No automated tests exist for action_channel scenarios. auto_capabilities.txt patterns are too broad and keep misrouting this item (bounced 25+ times). Returning for manual QA.

## Content

The action channels feature was partially implemented but targeted attack and target lock during Attack-Move are broken, preventing verification of the attack phase state machine.

**What passed:**
- Unit stops moving when engaging a target during Attack-Move
- Unit faces toward the target while shooting
- Move command successfully interrupts attack
- Damage is being dealt

**What failed:**
1. **Targeted attack broken**: Using Attack command + left-click on an enemy unit, or right-clicking an enemy unit, causes the attacking unit to simply stop in place. It does not pursue or attack the target.
2. **Attack-Move doesn't maintain target lock**: When a unit engages an enemy during Attack-Move, it fires once then resumes moving instead of staying locked on the target until it's dead or out of range.
3. **Turret units not available**: Steps 7-10 could not be tested as no turret units exist yet.

## QA Instructions

1. Select an infantry unit, press Attack, then left-click on an enemy unit. Verify the unit moves toward the enemy and begins attacking it.
2. Right-click on an enemy unit. Verify the unit moves toward and attacks the enemy.
3. Use Attack-Move to send a unit near enemies. When it engages a target, verify it stays locked on that target until dead or out of range.
4. During an active attack, verify the unit transitions through distinct phases: Aiming -> Firing -> Cooldown -> Reloading.
