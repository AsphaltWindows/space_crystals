# Gunner

The Gunner is the Cults' heavier-hitting combat infantry, trained from a Recruit at the Armory (requires owning a Gunworks). Shares the Militant's physical profile but trades air targeting and attack speed for a longer-range, harder-hitting ground attack that **can miss** (user, 2026-11-26).

## Entity Type - Unit
## Faction - TheCults
## UnitBase - LightInfantry

## Silhouette - 24x24 space units (square)
## MaxHP - 40
## PointArmor - 0
## FullArmor - 0
## SightRange - 5 grid units
## Destructible - true
## Groupable - true

## UnitBaseAttributes[LightInfantry]:
- TurnRate: 180 degrees/frame
- Acceleration: infinite
- Deceleration: infinite
- MaxSpeed: 4 space units/frame
- RuggedTerrainDefenseBonus: 50%

## TurretAttributes - None

## AttackAttributes:
- AttackType: **TailDisjointed** (user, 2026-11-26; was FullyConnected): the target's position is locked at the end of Aiming and the shot lands there at the end of Firing, so a target that moves during the Firing phase is missed, and whatever unit stands at the spot then is hit instead (any owner). Visually still an instant gunshot.
- CanTargetGround: true (derived): the Gunner can **Attack Ground**, like every attack that can miss.
- TargetDomain: Ground
- TargetType: SingleTarget
- Damage: 16
- Range: 5 grid units
- MinRange: 2
- AimDuration: 4 frames
- FiringDuration: 4 frames (user, 2026-11-26; Q0126: was 2, too short for anything to leave the spot; at 4 a walking infantry unit can step off it)
- CooldownDuration: 6 frames (was 8, shortened by the same 2 frames so the rate of fire is unchanged; designer placeholder)
- ReloadDuration: 8 frames

## ObjectInterfaceState: BasicCombatUnitInterfaceState
