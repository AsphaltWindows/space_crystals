# Ticket: Unit Type and Unit Instance

## Current State
Object Type and Object Instance exist from the entity system, but there is no Unit-specific extension of these types. TurretAttributes do not exist.

## Desired State
A `Unit` struct extending Object Type and a `UnitInstance` struct extending Object Instance, plus `TurretAttributes` for turret-bearing units.

### Unit (extends Object Type)
- Faction: FactionEnum
- Silhouette: 2D Mask (in SpaceUnits)
- MaxHP: number
- PointArmor: number
- FullArmor: number
- UnitBase: UnitBaseEnum
- UnitBaseAttributes: struct (parameterized by UnitBase — contains the movement model attributes and any base-specific fields like LightInfantry's RuggedTerrainDefenseBonus)
- TurretAttributes: TurretAttributes | None (present only when UnitBase.HasTurret = true)
- AttackAttributes: struct | None (present only when the unit can attack)

### Unit Instance (extends Object Instance)
- Rotation: continuous degrees
- CommandQueue: list of UnitCommand
- BaseCommandState: struct (parameterized by UnitBase)
- BaseBehaviorState: struct (parameterized by UnitBase)
- TurretCommandState: struct | None (present only for turret units)
- TurretBehaviorState: struct | None (present only for turret units)

### TurretAttributes
- TurnAngle: degrees (full arc, max 360), centered on unit facing, split equally clockwise/counter-clockwise
- TurnRate: number (independent of unit base turn rate)

## Justification
Defined in `features/unit_system.md`. Unit and UnitInstance are the central types that tie together all unit system components (bases, movement, turrets, combat). They extend the entity system's Object Type / Object Instance hierarchy as specified in the entity_system dependency.

## QA Steps
1. Verify `Unit` struct exists and contains all listed fields: Faction, Silhouette, MaxHP, PointArmor, FullArmor, UnitBase, UnitBaseAttributes, TurretAttributes, AttackAttributes.
2. Verify `UnitInstance` struct exists and contains: Rotation, CommandQueue, BaseCommandState, BaseBehaviorState, TurretCommandState, TurretBehaviorState.
3. Verify `TurretAttributes` struct has TurnAngle and TurnRate fields.
4. Verify TurretAttributes and TurretCommandState/TurretBehaviorState are optional (None for non-turret units).
5. Verify Unit extends or composes Object Type (however the entity system implements inheritance).
6. Verify UnitInstance extends or composes Object Instance.
7. Verify Rotation is stored as continuous degrees (not discrete/snapped).
8. Verify CommandQueue is typed as a list/vec of UnitCommand.

## Expected Experience
Inspecting the code shows a Unit type that clearly builds on Object Type with all combat-relevant fields (HP, armor, silhouette) plus a UnitBase reference and optional turret/attack attributes. UnitInstance builds on Object Instance with runtime state: rotation, command queue, and parameterized command/behavior states. TurretAttributes is a small struct with angle and rate. The optional fields (turret, attack) are clearly represented as Option types or equivalent.
