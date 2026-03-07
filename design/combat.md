# Combat

## AttackAttributes
Combat statistics for a unit that can attack. Only present on units that can attack. The attack source (turret or unit base) is determined by the unit's UnitBase HasTurret property. When attacking from a turret, the turret rotates to aim independently of the unit base. When attacking from the unit base, the entire unit must turn to face the target.

### AttackType - AttackTypeEnum (FullyConnected | HeadDisjointed | TailDisjointed | DoublyDisjointed)
### TargetDomain - TargetDomainEnum (Ground | Air | Universal)
### TargetType - TargetTypeEnum (SingleTarget | AoE)
### AoERadius - number (AoE only)
### Damage - number
### Range - number
### MinRange - number
### ProjectileSpeed - number (HeadDisjointed / DoublyDisjointed only)
### AimDuration - number
### FiringDuration - number
### CooldownDuration - number
### ReloadDuration - number

## AttackPhase
An attack sequence consists of four phases executed in order. Each phase has a duration and determines what actions the attacking unit or turret may perform.

### Value - AttackPhaseEnum

## Aiming
The unit or turret rotates to face the target. If the unit is given a different command during this phase, the attack sequence is interrupted. The target must remain valid throughout this phase or the attack sequence is cancelled. The unit will attempt to maintain target validity (e.g., turning to keep target in arc).

### Interruptible - true
### UnitBaseSourceActions - Turning
### TurretSourceActions - Turning
### TurretSourceBaseActions - Moving, Turning

## Firing
The attack effect takes place or becomes imminent. Once the Firing phase begins it cannot be interrupted.

### Interruptible - false
### UnitBaseSourceActions - None
### TurretSourceActions - None
### TurretSourceBaseActions - Moving, Turning

## Cooldown
A short phase during which the unit is still unresponsive, occurring after the attack effect takes place or becomes imminent.

### Interruptible - false
### UnitBaseSourceActions - None
### TurretSourceActions - None
### TurretSourceBaseActions - Moving, Turning

## Reloading
The main delay between attacks. The unit may engage in other behavior unrelated to the attack sequence.

### Interruptible - true
### UnitBaseSourceActions - Turning, Moving
### TurretSourceActions - Turning
### TurretSourceBaseActions - Moving, Turning

## AttackTarget
The target of an attack action. Constrained by AttackType: FullyConnected and HeadDisjointed require UnitTarget, TailDisjointed and DoublyDisjointed accept both.

### Value - UnitTarget | LocationTarget

## UnitTarget
A specific object instance being targeted. The attack tracks this target.

### Target - ObjectInstance

## LocationTarget
A specific map location being targeted. The attack is applied to this location.

### Target - Coordinates

## ValidTarget
An enemy unit is a valid target for an attacker if all of the following are true. This concept is referenced by turret autonomous scanning, attack-move scanning, hold position engagement, and idle auto-attack.

1. The enemy is a Destructible ObjectInstance
2. The enemy is visible to the attacker's owner
3. The enemy's domain is compatible with the attacker's AttackAttributes.TargetDomain (Ground attacks can target Ground and surfaced Underground units, Air attacks can target Air units, Universal attacks can target any unit)

Range and arc checks are applied separately by the specific behavior or context using ValidTarget.

## AttackType
Determines the relationship between the attack animation, projectile, and effect on the target. CanMiss and CanTargetGround are derived directly from the AttackType.

### Value - AttackTypeEnum
### CanMiss - derived (false for FullyConnected/HeadDisjointed, true for TailDisjointed/DoublyDisjointed)
### CanTargetGround - derived (false for FullyConnected/HeadDisjointed, true for TailDisjointed/DoublyDisjointed)
### ValidAttackTarget - derived (UnitTarget only for FullyConnected/HeadDisjointed, UnitTarget | LocationTarget for TailDisjointed/DoublyDisjointed)

## FullyConnected
The animation and the effect on the target are a single unified animation during the Firing phase. Neither the animation nor the effect can miss the target.

### Subtype - FullyConnectedSubtypeEnum (Ranged | Melee)

### Ranged
Standard ranged FullyConnected attack. Has meaningful Range. Benefits from ElevationModifier on range.

### Melee
Close-quarters FullyConnected attack. Fixed short range (adjacent contact). ElevationModifier does NOT apply to range.

## HeadDisjointed
The Firing phase is short and spawns a projectile which tracks the target independently of the attacking unit's further actions. The unit enters Cooldown and Reloading while the projectile is in flight. The effect of the attack is delayed until the projectile reaches the target. The projectile cannot miss.

### RequiresProjectileSpeed - true

## TailDisjointed
The target location is locked at the end of the Aiming phase. The animation and effect are applied to that location. The unit remains locked during the Firing phase. The effect applies to whatever units are at the target location at the end of the Firing phase. Targets can dodge by moving out of the locked location.

## DoublyDisjointed
Combination of Head and Tail disjointed. The Firing phase is short and spawns a projectile which travels to the location the target was at at the end of the Aiming phase. The unit is free after the projectile spawns. The effect applies to whatever units are at the target location when the projectile arrives. Targets can dodge by moving out of the locked location.

### RequiresProjectileSpeed - true

## AttackSource
Determines whether the attack originates from the unit base or from a turret. This affects what actions the unit can perform during each attack phase.

### Value - AttackSourceEnum (UnitBase | Turret)

## UnitBaseSource
The unit attacks from its base. The entire unit must turn to face the target to aim. The unit cannot move during Aiming, Firing, or Cooldown phases.

## TurretSource
The unit attacks from its turret. The turret rotates independently to aim while the unit base is free to move and turn during all attack phases.

## DamageCalculation
Defines how damage is calculated for single-target and AoE attacks, including directional armor.

## DirectionalArmor
For units with DirectionalArmor = true, damage is modified based on the angle of attack. The attack direction is determined by the vector from the attacker's position to the target (for single-target attacks) or from the AoE center to the target (for AoE attacks). Facing the attack source grants a damage reduction bonus, while being hit from the rear grants a damage increase penalty.

## SingleTargetDamage
Only valid against targets whose domain matches the attack's TargetDomain: Ground attacks can target Ground and surfaced Underground units, Air attacks can target Air units, Universal attacks can target any unit. Damage taken = Attack Damage - PointArmor. PointArmor is checked at the projectile hit location on the unit's silhouette. Directional armor modifies PointArmor based on the angle from attacker to target.

## AoEDamage
Damage is uniform across the AoE circle. AoE only affects units whose domain matches the attack's TargetDomain: Ground AoE hits Ground and surfaced Underground units, Air AoE hits Air units, Universal AoE hits all units regardless of domain. A unit's damage share = Attack Damage × (unit overlap area / AoE area). Effective armor = FullArmor × (unit overlap area / unit total area). Damage taken = damage share - effective armor. Directional armor modifies FullArmor based on the angle from the AoE center to the unit's center.

## LocomotionOrientationConstraints[MovementModel]
Defines the valid combinations of Locomotion and Orientation channel states for each movement model, and the constraints on turn rate for each combination. MaxTurnRate is the ceiling on how fast the unit can turn in a given combination — the engine applies up to this rate each tick.

## LocomotionOrientationConstraints[TurnRateMovement]

- Stationary + Turning: maxTurnRate = TurnRate
- Stationary + Maintaining: valid
- Moving + Turning: maxTurnRate = TurnRate
- Moving + Maintaining: valid
- Stopping + Turning: maxTurnRate = TurnRate
- Stopping + Maintaining: valid

## LocomotionOrientationConstraints[FixedTurnRadiusMovement]

- Stationary + Turning: invalid
- Stationary + Maintaining: valid
- Moving + Turning: maxTurnRate = currentSpeed / MinimumTurnRadius
- Moving + Maintaining: valid
- Reversing + Turning: maxTurnRate = currentSpeed / MinimumTurnRadius
- Reversing + Maintaining: valid
- Stopping + Turning: invalid
- Stopping + Maintaining: valid

## LocomotionOrientationConstraints[SpeedTurnRadiusMovement]

- Stationary + Turning: unconstrained
- Stationary + Maintaining: valid
- Moving + Turning: maxTurnRate = f(currentSpeed, SpeedToTurnRadiusRatio)
- Moving + Maintaining: valid
- Reversing + Turning: maxTurnRate = f(currentSpeed, SpeedToTurnRadiusRatio)
- Reversing + Maintaining: valid
- Stopping + Turning: unconstrained
- Stopping + Maintaining: valid

## LocomotionOrientationConstraints[DragMovement]

- Stationary + Turning: maxTurnRate = TurnRate
- Stationary + Maintaining: valid
- Moving + Turning: maxTurnRate = TurnRate
- Moving + Maintaining: valid
- Stopping + Turning: maxTurnRate = TurnRate
- Stopping + Maintaining: valid

## LocomotionOrientationConstraints[GliderMovement]

- Moving + Turning: maxTurnRate = f(currentSpeed, MaxCentripetalAcceleration)
- Moving + Maintaining: valid
