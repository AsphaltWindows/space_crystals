# Units

## Unit
A mobile Object Type, instances of which can exist on the ground, in the air, or underground. Has a Unit Base which determines its movement and attack behavior.

### Faction - FactionEnum
### Silhouette - Mask
### MaxHP - number
### PointArmor - number
### FullArmor - number
### UnitBase - UnitBaseEnum
### UnitBaseAttributes[UnitBase] - struct
### TurretAttributes[UnitBase] - struct | None
### AttackAttributes - struct | None

## Unit Instance
An Object Instance of a Unit.

### Rotation - number (continuous, degrees)
### CommandQueue - list of UnitCommand
### BaseCommandState[UnitBase] - struct
### BaseBehaviorState[UnitBase] - struct
### TurretCommandState - struct | None
### TurretBehaviorState - struct | None

## UnitCollision
Ground and air units use different collision models.

### Ground Collision
- Ground units are solid obstacles defined by their Silhouette rectangle
- Units cannot overlap — collision is hard, not soft
- Idle units do not move aside for other units
- Moving units must pathfind around occupied space
- Ground units collide with other ground units and with structures

### Air Collision
- Air units do not collide with ground units or structures
- Air units use soft separation with other air units — a gentle repulsion force that prevents stacking but does not hard-block
- SeparationRadius — per unit type, circular, the distance at which air-to-air repulsion activates. Must be larger than the unit's Silhouette

## MovementModel
Defines how a unit moves, turns, accelerates, and decelerates. Each UnitBase uses one of five movement models.

### Value - MovementModelEnum

## TurnRateMovement
Unit turns in place or while moving at a fixed rate. Fast acceleration and deceleration provide responsive, direct movement typical of most units in modern RTS games.

### TurnRate - number
### Acceleration - number
### Deceleration - number
### MaxSpeed - number

## FixedTurnRadiusMovement
Unit cannot turn in place. Turns at a fixed minimum radius regardless of speed. Can stop and reverse. Models a simple wheeled vehicle with no drift or slippage.

### MinimumTurnRadius - number
### ForwardAcceleration - number
### ForwardMaxSpeed - number
### ReverseAcceleration - number
### ReverseMaxSpeed - number
### Deceleration - number

## SpeedTurnRadiusMovement
Unit can rotate in place by spinning treads in opposite directions. Turn radius increases with speed due to the mechanical constraints of tread width and differential tread speeds. Small directional adjustments at speed cost little momentum, but sharp turns require significant slowing or a complete stop.

### SpeedToTurnRadiusRatio - number
### Acceleration - number
### Deceleration - number
### MaxSpeed - number

## DragMovement
Unit accelerates with thrust. OmniDirectionalAcceleration provides thrust in any direction. ForwardAcceleration provides additional thrust in the unit's forward facing only. Total forward thrust is OmniDirectionalAcceleration + ForwardAcceleration. Drag continuously opposes movement, and effective maximum speed is reached when total thrust and drag reach equilibrium (MaxSpeed = (OmniDirectionalAcceleration + ForwardAcceleration) / DragRatio). Air units typically have high drag and high acceleration, relying on drag alone to stop. Ground hover units have lower drag and actively thrust in the opposing direction to decelerate and change direction.

### ForwardAcceleration - number
### OmniDirectionalAcceleration - number
### DragRatio - number
### TurnRate - number

## GliderMovement
Unit must always maintain movement to stay airborne. When idle, the unit circles at a low speed in tight loops. When given orders, the unit accelerates to max speed. Turn radius is governed by centripetal acceleration (r = v²/a), so higher speeds produce wider turns and lower speeds allow tighter circles.

### IdleSpeed - number
### MaxSpeed - number
### Acceleration - number
### Deceleration - number
### MaxCentripetalAcceleration - number

## UnitBase
Determines a Unit's movement behavior, physical characteristics, and interaction with terrain. Each UnitBase has a MovementModel and a set of properties that define how the unit navigates the battlefield.

### Value - UnitBaseEnum
### Domain - Ground | Air | Underground
### HasTurret - boolean
### DirectionalArmor - boolean
### RuggedTerrain - boolean
### Crushable - boolean
### CanTurnInPlace - boolean
### CanReverse - boolean
### MovementModel - MovementModelEnum
### MovementModelAttributes[MovementModel] - struct

## LightInfantry
Ground infantry unit. Small, responsive, and able to traverse rugged terrain. Receives a damage reduction bonus on rugged terrain. Can be crushed by Tracked Vehicles and Mechs.

### Domain - Ground
### HasTurret - false
### DirectionalArmor - false
### RuggedTerrain - true
### Crushable - true
### CanTurnInPlace - true
### CanReverse - false
### MovementModel - TurnRateMovement
### RuggedTerrainDefenseBonus - number

## HeavyInfantry
Ground infantry unit. Sturdier than Light Infantry and cannot be crushed. Traverses rugged terrain but does not receive a defensive bonus there.

### Domain - Ground
### HasTurret - false
### DirectionalArmor - false
### RuggedTerrain - true
### Crushable - false
### CanTurnInPlace - true
### CanReverse - false
### MovementModel - TurnRateMovement

## WheeledVehicle
Ground vehicle that behaves like a simple car. Cannot turn in place, has a fixed turn radius, and can reverse. Has directional armor with frontal damage reduction and rear damage increase. Typically has wide to full turret turn angles.

### Domain - Ground
### HasTurret - true
### DirectionalArmor - true
### RuggedTerrain - false
### Crushable - false
### CanTurnInPlace - false
### CanReverse - true
### MovementModel - FixedTurnRadiusMovement

## TrackedVehicle
Ground vehicle on treads. Can rotate in place by spinning treads in opposite directions. Turn radius widens with speed. Crushes enemy Light Infantry. Has directional armor with frontal damage reduction and rear damage increase. Typically has wide to full turret turn angles.

### Domain - Ground
### HasTurret - true
### DirectionalArmor - true
### RuggedTerrain - false
### Crushable - false
### CanTurnInPlace - true
### CanReverse - true
### MovementModel - SpeedTurnRadiusMovement

## DrillUnit
Underground variant of a tracked vehicle. Operates underground and is visible while subterranean. Can travel across all drillable tiles while underground. Cannot fire while underground. Has an above-ground mode in which it is stationary or behaves as tracked. Has directional armor with frontal damage reduction and rear damage increase. Typically has wide to full turret turn angles.

### Domain - Underground
### HasTurret - true
### DirectionalArmor - true
### RuggedTerrain - false
### Crushable - false
### CanTurnInPlace - true
### CanReverse - true
### MovementModel - SpeedTurnRadiusMovement

## HoverVehicle
Ground vehicle that hovers above the surface. Accelerates omni-directionally with thrust and uses a combination of drag and opposing thrust to decelerate and change direction. Feels momentum-heavy and slidey. Has directional armor with frontal damage reduction and rear damage increase. Typically has wide to full turret turn angles.

### Domain - Ground
### HasTurret - true
### DirectionalArmor - true
### RuggedTerrain - false
### Crushable - false
### CanTurnInPlace - true
### CanReverse - false
### MovementModel - DragMovement

## Mech
Large ground walker that can traverse rugged terrain. Crushes enemy Light Infantry. Has directional armor with frontal damage reduction and rear damage increase. Typically has very narrow to narrow turret turn angles.

### Domain - Ground
### HasTurret - true
### DirectionalArmor - true
### RuggedTerrain - true
### Crushable - false
### CanTurnInPlace - true
### CanReverse - false
### MovementModel - TurnRateMovement

## HoverCraft
Air unit that hovers in place without moving. Relies on high drag to decelerate passively. Can turn in place or while moving. Typically has very narrow to narrow turret turn angles.

### Domain - Air
### HasTurret - true
### DirectionalArmor - false
### RuggedTerrain - false
### Crushable - false
### CanTurnInPlace - true
### CanReverse - false
### MovementModel - DragMovement

## Glider
Air unit that must always maintain movement to stay airborne. Circles at idle speed when not given orders. Cannot turn in place or stop. Turn radius widens at higher speeds due to centripetal acceleration constraints. Typically has very narrow to narrow turret turn angles.

### Domain - Air
### HasTurret - true
### DirectionalArmor - false
### RuggedTerrain - false
### Crushable - false
### CanTurnInPlace - false
### CanReverse - false
### MovementModel - GliderMovement

## TurretAttributes
Physical rotation properties of a unit's turret. Only present on units whose UnitBase has HasTurret = true. The turret is centered on the unit's facing direction, with TurnAngle split equally clockwise and counter-clockwise from center.

### TurnAngle - number (degrees, full arc, max 360)
### TurnRate - number
