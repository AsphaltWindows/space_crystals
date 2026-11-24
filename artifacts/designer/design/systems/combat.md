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

### Range Measurement
Range and MinRange are measured **edge to edge** (user, 2026-11-23): the shortest distance between the attacker's shape and the target's shape, i.e. the gap between them. Shapes are a unit's Silhouette and a structure's footprint. "Range 5" means the gap between attacker and target is at most 5 grid units.
- Big targets are not harder to reach than small ones, and a big attacker's or turret's own body doesn't eat into its range.
- Melee (adjacent contact) is a gap of about 0.
- Ground-location targets (Attack Ground, TailDisjointed artillery): measured from the attacker's shape to the target point.
- ElevationModifier adds to Range as before (core/vision.md > ElevationModifier).

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

The modifier is a **multiplier on damage after armor is applied** (it does not change the armor value). One global rule for every unit with DirectionalArmor = true:

| Zone (angle between the unit's facing and the direction to the attack source) | Damage multiplier |
|---|---|
| Front: within ±45° of facing (90° arc) | × 0.75 |
| Sides: the two 90° arcs between front and rear | × 1.0 |
| Rear: within ±45° of directly behind (90° arc) | × 1.25 |

Order of application: armor → directional multiplier → MinimumDamage.

## SingleTargetDamage
Only valid against targets whose domain matches the attack's TargetDomain: Ground attacks can target Ground and surfaced Underground units, Air attacks can target Air units, Universal attacks can target any unit. Damage taken = max((Attack Damage × pre-armor multipliers - PointArmor) × directional multiplier, 0.5) (see DirectionalArmor and MinimumDamage; the multiplier is 1 for units without directional armor). PointArmor is checked at the projectile hit location on the unit's silhouette. The directional zone is determined by the angle from the target to the attacker.

## AoEDamage
Damage is uniform across the AoE circle. AoE only affects units whose domain matches the attack's TargetDomain: Ground AoE hits Ground and surfaced Underground units, Air AoE hits Air units, Universal AoE hits all units regardless of domain. A unit's damage share = Attack Damage × (unit overlap area / AoE area). Effective armor = FullArmor × (unit overlap area / unit total area). Damage taken = max((damage share × pre-armor multipliers - effective armor) × directional multiplier, 0.5 × overlap fraction), where overlap fraction = unit overlap area / unit total area (see DirectionalArmor and MinimumDamage). The directional zone is determined by the angle from the unit's center to the AoE center.

## DamageModifierOrder
All damage against HP is resolved in this order (shared by single-target and AoE):

1. **Pre-armor percentage reductions** — each is a multiplier on the incoming damage (or AoE damage share). Multiple reductions **multiply** together (e.g. 50% and 20% → × 0.5 × 0.8 = 40% of the damage remains). Current sources:
   - RuggedTerrainDefenseBonus (Light Infantry on Rugged tiles, systems/units.md)
   - Null Chamber exit buff damage reduction (factions/syndicate/structures.md > Tunnel Expansions > Null Chamber)
   - Peacekeeper Riot Shield, 50% while raised (factions/gdo/units.md > Peacekeeper > Riot Shield)
2. **Armor** — subtract PointArmor (single-target) or effective FullArmor (AoE).
3. **Post-armor multipliers** — DirectionalArmor (front × 0.75 / sides × 1.0 / rear × 1.25).
4. **MinimumDamage** — floor of 0.5 (× overlap fraction for AoE; capped at shield overflow).

Colonists Shields are resolved before all of this: Shields take incoming damage at full value, and only the overflow enters step 1.

## MinimumDamage
Every hit deals at least a minimum amount of damage, regardless of armor. HP is tracked as a fractional value (AoE damage shares and the minimum below can produce fractions).

### MinimumDamage - 0.5 per hit
- **Single-target**: the minimum is 0.5.
- **AoE**: the minimum is 0.5 × overlap fraction — a unit entirely inside the AoE takes at least 0.5; a unit only partly inside takes at least that fraction of 0.5 (e.g. 20% inside → at least 0.1).
- Armor and then the directional multiplier are applied first; the minimum is applied to the result.
- **Shield overflow** (Colonists): when damage breaks through Shields, the minimum on the HP portion is capped at the overflow amount — HP damage = max(overflow - armor, min(minimum, overflow)). The minimum can never make HP damage exceed the damage that actually passed the Shields.

## Shields
A Colonists-only stat. Every Colonists unit and building has a MaxShields value in addition to MaxHP.

### MaxShields - number (Colonists units/buildings only; defined per-entity alongside MaxHP)
### ShieldRegenRate - 4 Shields per second (0.25 per frame). Constant base rate shared by all Colonists units and buildings. Shields regenerate continuously (not gated by a delay after taking damage). Upgradeable via the Fusion Core's shield upgrades (factions/colonists/structures.md > Fusion Core > Research), each level increasing the rate; per-level values TBD.

### Shield Damage Resolution
Incoming damage is applied to Shields first, at full value — PointArmor and FullArmor do NOT reduce damage against Shields. If damage exceeds current Shields, Shields drop to 0 and the remaining (overflow) damage is applied to HP using the normal SingleTargetDamage / AoEDamage armor calculations above (PointArmor for single-target, FullArmor for AoE), with the MinimumDamage capped at the overflow amount (see MinimumDamage > Shield overflow). If a unit's current Shields fully absorb the incoming damage, no HP is lost and armor is never consulted.

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
