# Guard

The Guard is the Syndicate's basic combat infantry. A heavy infantry unit armed with a **laser rifle** that hits **ground and air** and **cannot miss** (instant hit), firing at a measured pace rather than rapidly (user, 2026-11-24). Tougher than the GDO Peacekeeper but with shorter range.

## Produced at - Hideout
## TechPrerequisite - player owns at least one **Optics Lab** (user, 2026-11-24; core/tech_and_research.md > Tech Prerequisites > Unit Prerequisites). So the Optics Lab is the Syndicate's gate to an army; before it, only Agents (pistol) can fight.

## Entity Type - Unit
## Faction - TheSyndicate
## UnitBase - HeavyInfantry

## Silhouette - 36x36
## MaxHP - 80
## PointArmor - 1
## FullArmor - 1
## SightRange - 5
## TunnelSpaceCost - 1
## Groupable - true

## UnitBaseAttributes[HeavyInfantry]:
- MaxSpeed: 5 space units/frame
- Acceleration: infinite
- Deceleration: infinite
- TurnRate: 180 degrees/frame

## TurretAttributes - None

## AttackAttributes:
- AttackType: FullyConnected (laser; instant hit, cannot miss)
- TargetDomain: Universal (ground and air)
- TargetType: SingleTarget
- Damage: 18 (placeholder)
- Range: 3 grid units
- MinRange: 0
- AimDuration: 4 frames (placeholder)
- FiringDuration: 1 frame (placeholder)
- CooldownDuration: 3 frames (placeholder)
- ReloadDuration: 16 frames (placeholder)

One shot every 24 frames (1.5 s) ≈ 12 damage per second, the same output as the earlier rapid-fire version (6 damage every 0.5 s); fewer, bigger hits make armor matter less against it.

## ObjectInterfaceState: BasicCombatUnitInterfaceState
