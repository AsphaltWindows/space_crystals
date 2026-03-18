# Ticket: Attack Attributes, Types, and Targeting

## Current State
No combat data model exists. There is no way to define what attacks a unit can perform, what targets are valid, or how attacks behave.

## Desired State
Implement the core attack data model:

**AttackAttributes** struct on units that can attack:
- AttackType: AttackTypeEnum (FullyConnected | HeadDisjointed | TailDisjointed | DoublyDisjointed)
- TargetDomain: TargetDomainEnum (Ground | Air | Universal)
- TargetType: TargetTypeEnum (SingleTarget | AoE)
- AoERadius: number (only when TargetType = AoE)
- Damage, Range, MinRange: numbers
- ProjectileSpeed: number (only for HeadDisjointed / DoublyDisjointed)
- AimDuration, FiringDuration, CooldownDuration, ReloadDuration: numbers

**AttackSource** (derived from UnitBase.HasTurret):
- UnitBaseSource: entire unit turns to aim, cannot move during Aiming/Firing/Cooldown
- TurretSource: turret rotates independently, unit base free to move/turn during all phases

**AttackType** with derived properties:
- FullyConnected: CanMiss=false, CanTargetGround=false, ValidAttackTarget=UnitTarget only
- HeadDisjointed: CanMiss=false, CanTargetGround=false, ValidAttackTarget=UnitTarget only, RequiresProjectileSpeed=true
- TailDisjointed: CanMiss=true, CanTargetGround=true, ValidAttackTarget=UnitTarget|LocationTarget
- DoublyDisjointed: CanMiss=true, CanTargetGround=true, ValidAttackTarget=UnitTarget|LocationTarget, RequiresProjectileSpeed=true

**AttackTarget**:
- UnitTarget: references a specific ObjectInstance (tracks target)
- LocationTarget: references map Coordinates (fixed location)

**TargetDomain compatibility**:
- Ground attacks: hit Ground and surfaced Underground units
- Air attacks: hit Air units
- Universal attacks: hit any unit

**ValidTarget** reusable filter (enemy unit is valid if all true):
1. Destructible ObjectInstance
2. Visible to attacker's owner
3. Domain compatible with attacker's TargetDomain

Range and arc checks are applied separately by context.

## Justification
Required by `features/combat_system.md`. These are the foundational data structures for the entire combat system. AttackAttributes define per-unit combat stats, AttackType determines projectile/hit behavior, and ValidTarget is reused across turret scanning, attack-move, hold position, and idle auto-attack.

## QA Steps
1. Create a unit with AttackType=FullyConnected and verify CanMiss=false, CanTargetGround=false, ValidAttackTarget accepts only UnitTarget.
2. Create a unit with AttackType=DoublyDisjointed and verify CanMiss=true, CanTargetGround=true, ValidAttackTarget accepts UnitTarget and LocationTarget, and ProjectileSpeed is required.
3. Create AttackAttributes with TargetType=AoE and verify AoERadius is required.
4. Create AttackAttributes with TargetType=SingleTarget and verify AoERadius is not present/required.
5. Verify ValidTarget filter rejects: non-Destructible objects, non-Visible enemies, domain-incompatible units (e.g., Ground attack vs Air unit).
6. Verify ValidTarget filter accepts: Destructible + Visible + domain-compatible enemy units.
7. Verify Ground TargetDomain matches Ground and surfaced Underground units but not Air.
8. Verify Air TargetDomain matches Air units only.
9. Verify Universal TargetDomain matches Ground, Air, and Underground units.

## Expected Experience
Unit tests confirm that AttackAttributes enforce conditional field requirements (AoERadius for AoE, ProjectileSpeed for Head/DoublyDisjointed). AttackType derived properties (CanMiss, CanTargetGround, ValidAttackTarget) return correct values for all four variants. ValidTarget correctly filters based on destructibility, visibility, and domain compatibility. TargetDomain compatibility rules match the spec precisely.
