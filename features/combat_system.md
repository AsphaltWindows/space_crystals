# Feature: Combat System

## Overview
Attack attributes, attack phases, attack types, target domains, and damage calculation including directional armor and AoE mechanics.

## Design Sources
- `design/combat.md`

## Specifications

### AttackAttributes (per unit that can attack)
- AttackType: FullyConnected | HeadDisjointed | TailDisjointed | DoublyDisjointed
- TargetDomain: Ground | Air | Universal
- TargetType: SingleTarget | AoE (with AoERadius)
- Damage, Range, MinRange
- ProjectileSpeed (HeadDisjointed/DoublyDisjointed only)
- AimDuration, FiringDuration, CooldownDuration, ReloadDuration

### Attack Source
Determined by UnitBase.HasTurret:
- **UnitBaseSource**: Entire unit turns to face target. Cannot move during Aiming/Firing/Cooldown.
- **TurretSource**: Turret rotates independently. Unit base free to move/turn during all phases.

### Attack Phases (4, in order)

| Phase | Interruptible | UnitBase Source Actions | Turret Source Actions | Turret Source Base Actions |
|-------|---------------|------------------------|-----------------------|---------------------------|
| Aiming | Yes | Turning | Turning | Moving, Turning |
| Firing | No | None | None | Moving, Turning |
| Cooldown | No | None | None | Moving, Turning |
| Reloading | Yes | Turning, Moving | Turning | Moving, Turning |

### Attack Types

| Type | CanMiss | CanTargetGround | Valid Targets | Behavior |
|------|---------|-----------------|---------------|----------|
| FullyConnected | No | No | UnitTarget only | Animation + effect are unified in Firing phase |
| HeadDisjointed | No | No | UnitTarget only | Fires tracking projectile, unit free after spawn |
| TailDisjointed | Yes | Yes | Unit or Location | Locks target location at end of Aiming, effect at location |
| DoublyDisjointed | Yes | Yes | Unit or Location | Fires projectile to locked location, unit free after spawn |

#### FullyConnected Subtypes
FullyConnected has two subtypes that determine range behavior:

| Subtype | Range | ElevationModifier on Range |
|---------|-------|---------------------------|
| **Ranged** | Standard numeric range | Yes (benefits from elevation) |
| **Melee** | Fixed short range (adjacent contact) | No (elevation does not affect range) |

- **Ranged**: Standard FullyConnected behavior with a meaningful Range value. ElevationModifier applies to range.
- **Melee**: Close-quarters variant. Fixed short range defined as adjacent contact (attacker silhouette touching target). ElevationModifier does NOT apply to range. All other FullyConnected properties (CanMiss=false, UnitTarget only, unified animation+effect) still apply.

### Target Domain Compatibility
- Ground attacks: hit Ground and surfaced Underground units
- Air attacks: hit Air units
- Universal attacks: hit any unit

### ValidTarget (reusable filter)
An enemy unit is a valid target if:
1. Destructible ObjectInstance
2. Visible to attacker's owner
3. Domain compatible with attacker's TargetDomain

Range and arc checks applied separately by context.

### Damage Calculation

**Single Target**: Damage taken = Attack Damage - PointArmor
- PointArmor checked at projectile hit location on unit's silhouette
- Directional armor modifies PointArmor based on angle from attacker to target

**AoE**: Uniform damage across AoE circle. AoE only affects domain-compatible units.
- Unit damage share = Attack Damage x (unit overlap area / AoE area)
- Effective armor = FullArmor x (unit overlap area / unit total area)
- Damage taken = damage share - effective armor
- Directional armor modifies FullArmor based on angle from AoE center to unit center

### Directional Armor
- Only for units with DirectionalArmor = true
- Facing attack source: damage reduction bonus
- Hit from rear: damage increase penalty
- Direction vector: attacker-to-target (single target) or AoE-center-to-target (AoE)

## Dependencies
- `unit_system` (UnitBase determines attack source, movement constraints during attack)
- `vision_system` (ValidTarget requires visibility)
- `simulation_core` (GridUnit for ranges)

## Open Questions
- Head Disjointed projectile speed: purely cosmetic or gameplay impact beyond timing?
- Melee range: "adjacent contact" needs a concrete implementation rule (silhouette overlap? fixed small numeric range? grid adjacency?)
- ElevationModifier on attack range: the Ranged/Melee distinction establishes that elevation affects attack range for Ranged attacks. Exact modifier formula not yet specified in combat context (see `vision_system` for vision ElevationModifier).
