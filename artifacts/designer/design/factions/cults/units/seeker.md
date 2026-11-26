# Seeker

(Renamed from Jackal: user, 2026-11-27, a name in the same religious vein as the Prayer-mobile: a seeker wandering the desert for religious reasons.)

(user, 2026-11-26; named by the user) The Cults' **light vehicle**: a wheeled ATV/buggy built at the Chop Shop, with a **machine-gun turret** that fires **low damage at a high rate**, **cannot miss**, and **rotates 360°**.

## Entity Type - Unit
## Faction - TheCults
## UnitBase - WheeledVehicle
## Produced at - Chop Shop (consumes **2 Recruits**; Unit Control cost **2**; factions/cults/structures/README.md > Cults Vehicles)

## TurretAttributes:
- TurnAngle: **360°** (full rotation)

## AttackAttributes:
- AttackType: FullyConnected (machine gun; cannot miss)
- TargetDomain: **Universal** (ground and air)
- TargetType: SingleTarget
- Low damage, high rate of fire

**Placeholder stats** (designer, 2026-11-26, at the user's request: provisional numbers so the Director can implement; tune in the numbers pass):
- Silhouette 56x40 space units (length x width); MaxHP 120; PointArmor 1, FullArmor 1; SightRange 6 grid units; Groupable true.
- FixedTurnRadiusMovement: MinimumTurnRadius 64 su (1 grid unit); ForwardMaxSpeed 8 su/frame; ForwardAcceleration 1 su/frame²; ReverseMaxSpeed 3 su/frame; ReverseAcceleration 0.5 su/frame²; Deceleration 1 su/frame².
- Turret: TurnAngle 360°, TurnRate 12°/frame.
- Attack: Damage 3; Range 4 grid units; MinRange 0; Aim 1, Firing 1, Cooldown 1, Reload 1 frames (one shot every 4 frames: 12 damage per second before armor).
- Cost: Chop Shop, 2 Recruits, 75 Space Crystals, 320 frames.
- ObjectInterfaceState: BasicCombatUnitInterfaceState.
