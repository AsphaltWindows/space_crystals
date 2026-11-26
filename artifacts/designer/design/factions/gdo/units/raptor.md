# Raptor
GDO light missile vehicle produced at the Factory — a Humvee-like wheeled vehicle with a missile launcher turret. Like a Rocketman whose missiles are always homing. Does not carry infantry.

## Entity Type - Unit
## Faction - GlobalDefenseOrganization
## Produced at - Factory
## UnitBase - WheeledVehicle (fast, fixed turn radius, can reverse, directional armor)

## TurretAttributes:
- TurnAngle: **60°** full arc (±30° from the vehicle's facing) — placeholder. Deliberately narrow: an intentional exception, since WheeledVehicles typically have wide to full turret arcs.
- TurnRate: placeholder below

## AttackAttributes:
- AttackType: HeadDisjointed (homing missiles — track the target, cannot miss)
- TargetDomain: Universal (ground and air)
- Damage, Range, ProjectileSpeed, timings: placeholders below

## Transport - none (does not carry infantry)

Role vs Rocketman: both fire missiles at ground and air; the main difference is mobility — the Rocketman is slow infantry, the Raptor is **very fast**.

**Placeholder stats** (designer, 2026-11-27, user's standing permission; tune in the numbers pass):
- Silhouette 48x32 space units; MaxHP 110; PointArmor 1, FullArmor 1; SightRange 7 grid units; Groupable true; UnitControlCost 2.
- FixedTurnRadiusMovement: MinimumTurnRadius 80 su; ForwardMaxSpeed **10 su/frame** (very fast); ForwardAcceleration 1; ReverseMaxSpeed 3; ReverseAcceleration 0.5; Deceleration 1.
- Turret TurnRate 10°/frame.
- Attack: Damage 14; Range 6 grid units; MinRange 0; ProjectileSpeed 14 su/frame; Aim 4, Firing 2, Cooldown 4, Reload 22 frames (one missile per 32 frames).
- Cost: Factory, 100 Space Crystals, 240 frames.
- ObjectInterfaceState: BasicCombatUnitInterfaceState.
