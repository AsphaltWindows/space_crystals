# combat-attack-system

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Implement the combat and attack system as defined in `artifacts/designer/design/combat.md`.

**AttackAttributes** (per unit):
- AttackType: FullyConnected | HeadDisjointed | TailDisjointed | DoublyDisjointed
- TargetDomain: Ground | Air | Universal
- TargetType: SingleTarget | AoE (with AoERadius)
- Damage, Range, MinRange, ProjectileSpeed (Head/Doubly only)
- AimDuration, FiringDuration, CooldownDuration, ReloadDuration

**AttackPhase Sequence** (Aiming -> Firing -> Cooldown -> Reloading):
- **Aiming**: Unit/turret rotates to face target. Interruptible. Target must remain valid. UnitBase source: Turning only. Turret source: base free to Move+Turn.
- **Firing**: Attack effect happens. NOT interruptible. UnitBase source: locked. Turret source: base free.
- **Cooldown**: Post-fire lockout. NOT interruptible. Same constraints as Firing.
- **Reloading**: Delay between attacks. Interruptible. All movement free.

**AttackType Variants:**
1. **FullyConnected**: Animation+effect unified during Firing. Cannot miss. Cannot target ground. Subtypes: Ranged (normal range, elevation applies) and Melee (adjacent contact, elevation does NOT apply to range).
2. **HeadDisjointed**: Spawns tracking projectile during Firing. Projectile tracks target, cannot miss. Cannot target ground.
3. **TailDisjointed**: Target location locked at end of Aiming. Effect applied to that location during Firing. CAN miss (targets dodge). CAN target ground.
4. **DoublyDisjointed**: Spawns projectile to locked location. CAN miss. CAN target ground. Projectile travels to where target WAS.

**DamageCalculation:**
- **SingleTargetDamage**: Damage taken = Attack Damage - PointArmor. PointArmor checked at projectile hit location on silhouette. Domain must match.
- **AoEDamage**: Uniform across AoE circle. Unit's share = Damage x (overlap area / AoE area). Effective armor = FullArmor x (overlap area / unit area). Damage taken = share - armor. Domain must match.
- **DirectionalArmor**: For units with DirectionalArmor=true, armor is modified by angle of attack. Facing the attacker = damage reduction. Hit from rear = damage increase. Direction from attacker position (single) or AoE center (AoE) to target.

**ValidTarget** (referenced by scanning/auto-attack):
1. Enemy is Destructible
2. Enemy is visible to attacker's owner
3. Enemy domain compatible with attacker's TargetDomain (Ground hits Ground+surfaced Underground, Air hits Air, Universal hits all)

## QA Instructions

1. Test a FullyConnected Ranged attack (e.g., Peacekeeper) — verify damage = Attack Damage - PointArmor, never misses.
2. Test a FullyConnected Melee attack (e.g., Agent) — verify it only works at adjacent range, elevation does not modify melee range.
3. Test attack phases: verify Aiming is interruptible (issue new command during aim), Firing/Cooldown are NOT interruptible.
4. During Reloading phase, verify the unit can move and turn freely.
5. Test a TailDisjointed attack — verify the target location locks at end of Aiming, and a fast unit can dodge by moving away.
6. Test a HeadDisjointed attack — verify projectile tracks and hits even if target moves.
7. Test a DoublyDisjointed attack — verify projectile goes to locked location, target can dodge.
8. Test AoE damage — verify units partially inside the AoE take proportional damage based on overlap.
9. Test DirectionalArmor — attack a vehicle from the front vs rear, verify front takes less damage.
10. Verify Ground attacks cannot target Air units, Air attacks cannot target Ground units, Universal attacks hit both.
