# Sabre Legionnaire

The Colonists' basic melee combat infantry, produced at the Bio Portal. Shares an identical physical profile with the Rifle Legionnaire and may convert into one at will (see Weapon Conversion below).

## Entity Type - Unit
## Faction - Colonists
## UnitBase - HeavyInfantry

## Silhouette - 36x36 space units (square)
## MaxHP - 70
## MaxShields - 10
## PointArmor - 0
## FullArmor - 0
## SightRange - 5 grid units
## Destructible - true
## Groupable - true
## BeaconCapacityCost - 2

## UnitBaseAttributes[HeavyInfantry]:
- TurnRate: 180 degrees/frame
- Acceleration: infinite
- Deceleration: infinite
- MaxSpeed: 5 space units/frame

## TurretAttributes - None

## AttackAttributes:
- AttackType: FullyConnected (Melee)
- TargetDomain: Ground
- TargetType: SingleTarget
- Damage: 15
- Range: melee (adjacent contact)
- AimDuration: 3 frames
- FiringDuration: 3 frames
- CooldownDuration: 1 frame
- ReloadDuration: 5 frames

## ObjectInterfaceState: BasicCombatUnitInterfaceState, plus:
- **R: Convert to Rifle Legionnaire** — Immediate command (CommandIssuingTransition). See Weapon Conversion below for eligibility and duration.
