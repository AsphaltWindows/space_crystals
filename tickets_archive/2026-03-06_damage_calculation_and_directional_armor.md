# Ticket: Damage Calculation and Directional Armor

## Current State
No damage calculation logic exists. There is no way to resolve attack damage against targets, account for armor, or handle AoE damage distribution.

## Desired State
Implement damage calculation for both SingleTarget and AoE attacks, including directional armor.

**SingleTarget Damage**:
- Damage taken = Attack Damage - PointArmor
- PointArmor is checked at the projectile hit location on the unit's silhouette
- Only affects targets whose domain matches the attack's TargetDomain

**AoE Damage**:
- Damage is uniform across the AoE circle
- Only affects domain-compatible units within the AoE radius
- Per-unit calculation:
  - Unit damage share = Attack Damage x (unit overlap area / AoE area)
  - Effective armor = FullArmor x (unit overlap area / unit total area)
  - Damage taken = damage share - effective armor
- Units partially in the AoE receive proportionally less damage but also apply proportionally less armor

**Directional Armor** (only for units with DirectionalArmor = true):
- Modifies armor based on attack angle relative to target's facing
- Direction vector for SingleTarget: attacker position -> target position
- Direction vector for AoE: AoE center -> unit center
- Facing the attack source: damage reduction bonus (armor increased)
- Hit from rear: damage increase penalty (armor decreased)

**Domain compatibility in damage** (same rules as TargetDomain):
- Ground attacks: damage Ground and surfaced Underground units
- Air attacks: damage Air units
- Universal attacks: damage any unit

## Justification
Required by `features/combat_system.md`. Damage calculation is the final resolution of combat actions. The split between SingleTarget (point-based armor) and AoE (area-based armor) creates distinct tactical roles. Directional armor rewards positioning and flanking. AoE partial-overlap math ensures small units take less splash but also shield less.

## QA Steps
1. SingleTarget: unit with 10 Damage attacks unit with 3 PointArmor -> verify 7 damage taken.
2. SingleTarget: unit with 5 Damage attacks unit with 8 PointArmor -> verify 0 damage taken (not negative).
3. AoE: unit fully inside AoE circle -> verify damage share = Attack Damage x (unit area / AoE area), effective armor = FullArmor x (unit area / unit area) = FullArmor, damage = share - FullArmor.
4. AoE: unit half inside AoE circle -> verify damage share uses 50% overlap, effective armor uses 50% overlap fraction.
5. AoE: verify only domain-compatible units take damage (Ground AoE does not damage Air units).
6. Directional armor: unit facing attacker -> verify PointArmor/FullArmor is increased (damage reduction bonus).
7. Directional armor: unit facing away from attacker -> verify PointArmor/FullArmor is decreased (damage increase penalty).
8. Directional armor: verify it only applies to units with DirectionalArmor = true; units without it use base armor regardless of angle.
9. AoE directional armor: verify direction vector is from AoE center to unit center, not from attacker to unit.
10. Verify domain filtering: Ground attack does not damage Air unit in AoE; Universal attack damages all units in AoE.

## Expected Experience
SingleTarget attacks deal damage reduced by point armor at the hit location. AoE attacks distribute damage proportionally based on unit overlap with the blast area, with armor similarly scaled. Units with directional armor take less damage when facing their attacker and more when flanked. Domain filtering ensures attacks only affect compatible targets. Edge cases (armor exceeding damage, zero overlap) resolve cleanly without negative damage.
