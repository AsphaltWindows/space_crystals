# Feature: Unit System

## Overview
Mobile object types with 9 unit bases and 5 movement models defining movement physics, terrain interaction, and physical characteristics.

## Design Sources
- `design/units.md`

## Specifications

### Unit (extends Object Type)
- Faction (FactionEnum), Silhouette (2D Mask in space units), MaxHP, PointArmor, FullArmor
- UnitBase (UnitBaseEnum), UnitBaseAttributes (parameterized by UnitBase)
- TurretAttributes (struct | None), AttackAttributes (struct | None)

### Unit Instance (extends Object Instance)
- Rotation: continuous degrees
- CommandQueue: list of UnitCommand
- BaseCommandState, BaseBehaviorState (parameterized by UnitBase)
- TurretCommandState, TurretBehaviorState (present only for turret units)

### UnitBase Properties
| Property | Description |
|----------|-------------|
| Domain | Ground, Air, or Underground |
| HasTurret | boolean |
| DirectionalArmor | boolean |
| RuggedTerrain | boolean (can traverse rugged terrain) |
| Crushable | boolean (can be crushed by Tracked/Mechs) |
| CanTurnInPlace | boolean |
| CanReverse | boolean |
| MovementModel | MovementModelEnum |

### 9 Unit Bases

| Base | Domain | Turret | DirArmor | Rugged | Crushable | TurnInPlace | Reverse | Movement |
|------|--------|--------|----------|--------|-----------|-------------|---------|----------|
| LightInfantry | Ground | No | No | Yes | Yes | Yes | No | TurnRate |
| HeavyInfantry | Ground | No | No | Yes | No | Yes | No | TurnRate |
| WheeledVehicle | Ground | Yes | Yes | No | No | No | Yes | FixedTurnRadius |
| TrackedVehicle | Ground | Yes | Yes | No | No | Yes | Yes | SpeedTurnRadius |
| DrillUnit | Underground | Yes | Yes | No | No | Yes | Yes | SpeedTurnRadius |
| HoverVehicle | Ground | Yes | Yes | No | No | Yes | No | Drag |
| Mech | Ground | Yes | Yes | Yes | No | Yes | No | TurnRate |
| HoverCraft | Air | Yes | No | No | No | Yes | No | Drag |
| Glider | Air | Yes | No | No | No | No | No | Glider |

### Special Properties
- LightInfantry: RuggedTerrainDefenseBonus (unique to this base)
- TrackedVehicle, Mech: Crushes enemy LightInfantry
- DrillUnit: Visible while underground. Can traverse Drillable tiles underground. Cannot fire underground. Above-ground mode: stationary or tracked behavior.

### TurretAttributes (turret-bearing units only)
- TurnAngle: degrees (full arc, max 360), centered on unit facing, split equally clockwise/counter-clockwise
- TurnRate: independent of unit base turn rate
- Typical ranges: Wheeled/Tracked/Drill/Hover = wide to full; Mech/HoverCraft/Glider = very narrow to narrow

### 5 Movement Models

**TurnRateMovement**: TurnRate, Acceleration, Deceleration, MaxSpeed
- Used by: LightInfantry, HeavyInfantry, Mech

**FixedTurnRadiusMovement**: MinimumTurnRadius, ForwardAcceleration, ForwardMaxSpeed, ReverseAcceleration, ReverseMaxSpeed, Deceleration
- Used by: WheeledVehicle. Cannot turn in place.

**SpeedTurnRadiusMovement**: SpeedToTurnRadiusRatio, Acceleration, Deceleration, MaxSpeed
- Used by: TrackedVehicle, DrillUnit. Can rotate in place, wider turns at speed.

**DragMovement**: ForwardAcceleration, OmniDirectionalAcceleration, DragRatio, TurnRate
- Used by: HoverVehicle, HoverCraft. MaxSpeed = (Omni + Forward) / DragRatio.

**GliderMovement**: IdleSpeed, MaxSpeed, Acceleration, Deceleration, MaxCentripetalAcceleration
- Used by: Glider. Must always maintain movement. Idle circling. Turn radius = v^2/a.

### UnitCollision
Ground and air units use different collision models.

**Ground Collision**:
- Ground units are solid obstacles defined by their Silhouette rectangle
- Hard collision — units cannot overlap
- Idle units do not move aside for other units
- Moving units must pathfind around occupied space
- Ground units collide with other ground units and with structures

**Air Collision**:
- Air units do not collide with ground units or structures
- Soft separation with other air units — gentle repulsion force that prevents stacking but does not hard-block
- SeparationRadius: per unit type, circular, distance at which air-to-air repulsion activates. Must be larger than unit's Silhouette

### LocomotionOrientationConstraints
Each movement model defines valid combinations of Locomotion and Orientation channel states with maxTurnRate constraints. See `design/combat.md` for full constraint tables.

## Dependencies
- `entity_system` (Unit extends Object Type, Unit Instance extends Object Instance)
- `simulation_core` (SpaceUnit for silhouettes and speeds, GridUnit for ranges)

## Open Questions
- How exactly does silhouette rotation work at non-90-degree angles?
- DrillUnit above-ground mode transition details
- Underground collision model: DrillUnit (Domain=Underground) not covered by Ground or Air collision rules. Do underground units collide with each other? With ground units above? Likely separate plane (no ground collision) with hard collision between underground units, but unspecified.
