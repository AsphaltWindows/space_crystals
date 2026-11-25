# Soldier

The Soldier is the Cults' basic combat infantry, trained from a Recruit at the Armory. A light infantry unit with a fully connected attack effective against both ground and air targets.

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
- TargetDomain: Universal
- TargetType: SingleTarget
- Damage: 4
- Range: 4 grid units
- MinRange: 0
- AimDuration: 2 frames
- FiringDuration: 2 frames
- CooldownDuration: 1 frame
- ReloadDuration: 3 frames

## ObjectInterfaceState: BasicCombatUnitInterfaceState
