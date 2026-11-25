# Raptor
GDO light missile vehicle produced at the Factory — a Humvee-like wheeled vehicle with a missile launcher turret. Like a Rocketman whose missiles are always homing. Does not carry infantry.

## Entity Type - Unit
## Faction - GlobalDefenseOrganization
## Produced at - Factory
## UnitBase - WheeledVehicle (fast, fixed turn radius, can reverse, directional armor)

## TurretAttributes:
- TurnAngle: **60°** full arc (±30° from the vehicle's facing) — placeholder. Deliberately narrow: an intentional exception, since WheeledVehicles typically have wide to full turret arcs.
- TurnRate: TBD

## AttackAttributes:
- AttackType: HeadDisjointed (homing missiles — track the target, cannot miss)
- TargetDomain: Universal (ground and air)
- Damage, Range, ProjectileSpeed, timings: TBD

## Transport - none (does not carry infantry)

Role vs Rocketman: both fire missiles at ground and air; the main difference is mobility — the Rocketman is slow infantry, the Raptor is **very fast**.

TBD: Silhouette, MaxHP, armor, SightRange, MaxSpeed/turn radius, cost, build time, Unit Control cost, ObjectInterfaceState.
