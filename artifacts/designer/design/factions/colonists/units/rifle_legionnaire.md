# Rifle Legionnaire

The Colonists' basic ranged combat infantry. Shares an identical physical profile with the Sabre Legionnaire and may convert into one at will (see Weapon Conversion below). Not directly produced — reached only by converting a Sabre Legionnaire.

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
- AttackType: FullyConnected (Ranged)
- TargetDomain: Ground
- TargetType: SingleTarget
- Damage: 10
- Range: 4 grid units
- MinRange: 0
- AimDuration: 3 frames
- FiringDuration: 2 frames
- CooldownDuration: 7 frames
- ReloadDuration: 4 frames

## ObjectInterfaceState: BasicCombatUnitInterfaceState, plus:
- **R: Convert to Sabre Legionnaire** — Immediate command (CommandIssuingTransition). See Weapon Conversion below for eligibility and duration.
