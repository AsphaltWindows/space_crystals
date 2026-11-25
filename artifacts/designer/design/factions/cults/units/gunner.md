# Gunner

The Gunner is the Cults' heavier-hitting combat infantry, trained from a Recruit at the Armory (requires owning a Gunworks). Shares the Soldier's physical profile but trades air targeting and attack speed for a longer-range, harder-hitting ground attack.

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
- AttackType: FullyConnected
- TargetDomain: Ground
- TargetType: SingleTarget
- Damage: 16
- Range: 5 grid units
- MinRange: 2
- AimDuration: 4 frames
- FiringDuration: 2 frames
- CooldownDuration: 8 frames
- ReloadDuration: 8 frames

## ObjectInterfaceState: BasicCombatUnitInterfaceState
