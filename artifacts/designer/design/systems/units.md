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

### Silhouette Rotation
A unit's Silhouette **rotates with its facing** (user, 2026-11-23). A long vehicle facing north is tall and narrow; facing east it is wide and flat. Collision, crushing, range measurement and proximity all use the rotated shape. A long unit fits through a narrow gap only when pointed through it.
- **Turning near obstacles**: see Collision Response (Un-sticking) below. Blocked units don't freeze; they creep and turn out of the contact.

### Collision Response (Un-sticking)
How a ground unit's movement resolves against obstacles each simulation frame (user, 2026-11-23). It is equivalent to **running into the wall every frame**:
1. **Velocity**: add this frame's acceleration to the unit's velocity (per its MovementModel), and compute this frame's turn (heading change).
2. **Displacement**: compute the displacement from the velocity and apply it, together with the turn, to get the new position and facing.
3. **Blocked**: if the movement runs into an obstacle (structure, impassable terrain, another ground unit), the **blocked portion of the movement is dropped** (the unit stops at contact instead of overlapping) and its **velocity is set to 0**.

Consequence: a unit pressed against an obstacle restarts from 0 every frame, so it moves at **single-frame-acceleration speed**. It creeps, but it keeps making its turn while the blocked part of each step is discarded. It gradually scrapes around the obstacle until it is free, then accelerates normally. This matters most for units that can't turn in place (WheeledVehicle: turning needs forward motion), which would otherwise jam against walls.

Applies to all ground units. Overlap caused by the rotation itself (a corner sweeping into a wall) follows the same principle: the blocked part is discarded and the unit never ends up overlapping. The exact geometric method is a technical choice.

### Ground Collision
- Ground units are solid obstacles defined by their Silhouette rectangle
- Units cannot overlap — collision is hard, not soft
- **Friendly yielding** (user, 2026-11-23, replaces "idle units do not move aside"): when a moving unit is blocked by one of its owner's **idle** units (no active command, including units that have arrived), the idle unit takes **one short sidestep** out of the way. Enemy units never yield. Two **moving** units never yield to each other or swap places, even friendly ones.
- **Moving units steer around blockers themselves** (user, 2026-11-26, Q0088: documents the movement already built in M1, the Director's `docs/movement.md > 6.3`; the user wants this sidestepping in general): a moving unit looks about **4 body lengths ahead** and shifts to a clear lane; on contact it **steps aside**, switching sides after 1 s. **Which side** (user, 2026-11-26; Q0129): at a **standing** blocker, it goes round **on the side of the blocker it is already on**; **right first** only when it is dead centre. Two **moving** units meeting head-on both step **right**, and a moving blocker is otherwise passed the way the unit leans (unchanged). **The contact step** (user, 2026-11-26; Q0111 context; replaces the 45° step): for **infantry (LightInfantry, HeavyInfantry), hover vehicles (HoverVehicle), DrillUnits (the Subterranean Transport is a tracked vehicle in its above-ground mode; user, 2026-11-26; Q0128) and TrackedVehicles**, the step is at **90°** to its heading (straight sideways), and its length is **the unit's own width** (the size of its Silhouette across its heading), so it clears the lane it was in. **WheeledVehicles** instead **turn and graze** around the obstacle (user, 2026-11-26). Sliding along the blocker's face is unchanged. **Mechs** turn to face the side, then step sideways the same way (user, 2026-11-26; no Mech units are planned yet). Two units meeting head-on in the open pass each other almost without delay. Only if that fails (e.g. a one-unit-wide corridor) does the stuck rule apply: re-route, then eventually give up (systems/controls/command_behaviors.md > MovingToLocation). Chokepoints and body-blocking stay meaningful.
  - **Who yields** (user, 2026-11-23): only units with **no active command** yield. Units on **Hold Position** don't (so players can deliberately wall a passage), and neither do units busy with a stationary task (attacking from where they stand, mining, constructing).
  - **Unit modes don't matter**: a mode such as the Peacekeeper's raised Riot Shield isn't a task. An idle Peacekeeper with the shield raised yields like any idle unit.
  - **Vehicles** (user, 2026-11-26; Q0119): an **idle vehicle doesn't yield** (it can't take a sidestep; it stands like a unit on Hold Position). **Which vehicles** (user, 2026-11-26; Q0172): **WheeledVehicle, TrackedVehicle and a DrillUnit in its above-ground mode** stand (they don't yield and don't take the idle push-apart nudge). The **HoverVehicle yields** like infantry, since it can move any way. **Own idle infantry yield to a moving vehicle** as to any unit. A vehicle **reverses only on the Reverse order** (never by itself, e.g. no automatic three-point turns), and a point inside its turning circle (e.g. just behind it) is reached by a **forward loop**.
  - **Units that can't move never yield**: a deployed Artillery Rig is treated like a building, a fixed obstacle.
  - **No yielding within the same group order**: a unit that has arrived under a group order (one order given to several units at once, systems/controls/commands.md > Group Destinations) does **not** yield to members of that same order that are still arriving. They stop against it (see systems/controls/command_behaviors.md > MovingToLocation > Occupied target location). It yields normally to every other own unit. This lasts until the unit receives a new order (user, 2026-11-23).
  - TBD: sidestep distance; whether allied (not own) units yield once teams exist.
- Moving units must pathfind around occupied space
- **Exception, gathering Drones**: Drones on a Gather job pass through each other, of any player (factions/colonists/units/drone.md > Gathering > Gathering Drones pass through each other).
- **Units that already overlap** (user, 2026-11-24): two ground units that overlap without being allowed to (e.g. a Drone ordered to Move while it sits inside another Drone it had passed through) **ignore each other until they have separated**; from then on, normal collision applies between them.
  - **Idle units push apart** (user, 2026-11-24): an **idle** unit (no active command, the same units that yield; see Who yields) that overlaps another unit moves off automatically, a short unordered nudge, until they no longer overlap. Two idle overlapping units both move apart. Units with an active command (moving, mining, Hold Position) are not moved; they separate by carrying on with their order. So units never stay stacked on top of each other.
- Ground units collide with other ground units, with structures, and with **Space Crystals patches and Supply Delivery Stations** (their footprints are solid; user, 2026-11-23). A depleted patch disappears and its tile opens up, so the map's paths can change during a game.

### Underground Collision
- Underground units **do not collide with each other** (user, 2026-11-23): they pass through one another freely, so underground traffic never jams.
- **Surfacing**: an underground unit is **not allowed to surface** where its Silhouette would overlap an above-ground unit (user, 2026-11-23), nor a structure or terrain impassable for its surface form. It stays underground until the player surfaces it somewhere clear. It neither waits automatically nor gets moved to another spot.
- **Reserved while surfacing**: once a unit has started surfacing, its surface footprint is **reserved**. Ground units treat it as occupied (an obstacle) and can't move onto it, and nothing can be built there, until surfacing completes (user, 2026-11-23).

### Air Collision
- Air units ignore terrain: they can fly over every tile of the map (user, 2026-11-23).
- **Map edge**: HoverCraft stay inside the map. **Gliders may unavoidably travel a little outside the map edge** while turning or circling, since they can't stop (user, 2026-11-23). Orders can only target points inside the map (a Glider never goes outside on orders, only while turning or circling, e.g. its idle circle around a point near the edge). A Glider outside the map edge **stays visible and attackable as if it were at the nearest edge tile** (user, 2026-11-23), so the edge is never a safe parking spot. TBD: maximum overshoot.
- Air units do not collide with ground units or structures
- Air units use soft separation with other air units — a gentle repulsion force that prevents stacking but does not hard-block
- SeparationRadius — per unit type, circular, the distance at which air-to-air repulsion activates. Must be larger than the unit's Silhouette
- **Gliders push apart slowly** (user, 2026-11-27; Q0148): gliders do separate, but with a **much weaker** repulsion than other air units (**a third** of the HoverCraft strength, user, 2026-11-26, Q0152, to try; was a quarter), so they drift apart gradually, mostly while circling, and their attack runs and loops are barely bent. Expect them to take **roughly 10 seconds or more** to spread out after bunching (at a quarter, six Sparrows sent to one point took 11–17 s), not a few seconds. The SeparationRadius rule above applies to them too.

### Crushing
Units whose UnitBase can crush (TrackedVehicle, Mech, and a **DrillUnit in its above-ground mode**: user, 2026-11-26, Q0153; never underground) crush enemy units whose UnitBase has Crushable = true (Light Infantry).
- **Effect**: instant death of the crushed unit.
- **Trigger**: the crusher's Silhouette overlaps a crushable enemy **while the crusher has any speed** (Locomotion = Moving, Reversing or **Stopping**: a braking or coasting crusher still crushes; user, 2026-11-27; Q0178).
- **Pathing exception to hard collision**: crushers may path and drive through crushable enemies — those enemies are not obstacles to them.
- **Stationary crushers do not overlap**: a crusher with no speed (Stationary and not turning; turning in place still crushes, below) remains a normal hard obstacle; crushable units cannot walk into it.
- **Crushable units avoid crushers**: crushable units still treat crushers as obstacles in their own pathing (they never walk into a crusher).
- **No friendly crushing**: crushers cannot run over their own or allied crushable units; those remain hard obstacles and crushers must path around them.
- **Turning in place crushes** (user, 2026-11-26; Q0153): a crusher turning in place (Stationary + Turning) **crushes the crushable enemies its turning Silhouette sweeps over**; they don't block the spin. Own and allied crushable units still block it (No friendly crushing): the spin stops at contact. A crusher that is stationary and **not** turning stays a hard obstacle, as above.

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

### TrackWidth - number (space units: the distance between the two treads)
### Acceleration - number
### Deceleration - number
### MaxSpeed - number
### ReverseMaxSpeed - number

**The turning law** (user, 2026-11-26; Q0153; replaces SpeedToTurnRadiusRatio, whose radius-proportional-to-speed law gave the same turning rate at every speed). It follows the two treads: forward speed is their average, turning comes from their difference, and each tread's top speed is MaxSpeed. At full speed both treads are already at maximum, so the vehicle **can't turn at all**.
- **Spin rate** (stopped, treads in opposite directions): **SpinRate = 2 × MaxSpeed ÷ TrackWidth** radians per frame.
- **Sharpest turn at speed v**: **maxTurnRate = SpinRate × (1 − |v| ÷ MaxSpeed)**. It falls in a straight line from SpinRate when stopped to **0 at MaxSpeed**.
- **Tightest radius at speed v**: **(TrackWidth ÷ 2) × |v| ÷ (MaxSpeed − |v|)**. At half speed it is TrackWidth ÷ 2 (pivoting round the stopped inner tread).
- **Reversing** uses the same law with the speed's size |v|; reverse is capped at **ReverseMaxSpeed** (placeholder: half MaxSpeed). Vehicles reverse only on the Reverse order (UnitCollision, "Vehicles").
- Turning costs no speed by itself (no skid drag); to turn more sharply, the vehicle slows with its Deceleration.
- **Example** (Hand of Heaven, MaxSpeed 5, TrackWidth 30): spins at about 19° a frame (90° in about 0.3 s); at half speed about 9.5° a frame on a 15 su radius; at 4 su/frame about 3.8° a frame on a 60 su radius; at 4.5 about 1.9° on 135 su; at 5 it can't turn.
- **Placeholder TrackWidth** (designer, 2026-11-26): about **¾ of the Silhouette's width** (the treads sit just inside the hull).

**Pathfinding note** (user, 2026-11-23): tracked units can reach the same place along many different paths by trading speed for turning rate (slow down for a sharp turn vs keep speed on a wide arc). To choose the optimal path, the implementation may use **pre-computed values** (e.g. tables of maneuvers: time and space needed to turn by a given angle from a given speed).

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
### CanHaveTurret - boolean (renamed from HasTurret, user 2026-11-24: whether units of this base **can** carry a turret; whether a particular unit actually has one is set by its own TurretAttributes, e.g. the unarmed Drone's are None)
### DirectionalArmor - boolean
### RuggedTerrain - boolean (can the unit enter Rugged tiles? If false, Rugged tiles are **impassable** for it, like any non-Traversible tile: its Silhouette may never overlap a Rugged tile. User, 2026-11-23. Ground domain only: air units ignore terrain, so this flag has no effect for them)
### Crushable - boolean
### CanTurnInPlace - boolean
### CanReverse - boolean
### MovementModel - MovementModelEnum
### MovementModelAttributes[MovementModel] - struct

## LightInfantry
Ground infantry unit. Small, responsive, and able to traverse rugged terrain. Receives a damage reduction bonus on rugged terrain. Can be crushed by Tracked Vehicles and Mechs.

### Domain - Ground
### CanHaveTurret - false
### DirectionalArmor - false
### RuggedTerrain - true
### Crushable - true
### CanTurnInPlace - true
### CanReverse - false
### MovementModel - TurnRateMovement
### RuggedTerrainDefenseBonus - number

**Which tile counts** (user, 2026-11-23): a unit is "on Rugged terrain" when the tile under its **centre** is Rugged. This applies to the RuggedTerrainDefenseBonus and to the HoverVehicle's double drag. (Passability is different: a unit that can't enter Rugged terrain may not overlap a Rugged tile at all.)

## HeavyInfantry
Ground infantry unit. Sturdier than Light Infantry and cannot be crushed. Traverses rugged terrain but does not receive a defensive bonus there.

### Domain - Ground
### CanHaveTurret - false
### DirectionalArmor - false
### RuggedTerrain - true
### Crushable - false
### CanTurnInPlace - true
### CanReverse - false
### MovementModel - TurnRateMovement

## WheeledVehicle
Ground vehicle that behaves like a simple car. Cannot turn in place, has a fixed turn radius, and can reverse. Has directional armor with frontal damage reduction and rear damage increase. Typically has wide to full turret turn angles.

### Domain - Ground
### CanHaveTurret - true
### DirectionalArmor - true
### RuggedTerrain - false
### Crushable - false
### CanTurnInPlace - false
### CanReverse - true
### MovementModel - FixedTurnRadiusMovement

## TrackedVehicle
Ground vehicle on treads. Can rotate in place by spinning treads in opposite directions. Turn radius widens with speed. Crushes enemy Light Infantry. Has directional armor with frontal damage reduction and rear damage increase. Typically has wide to full turret turn angles.

### Domain - Ground
### CanHaveTurret - true
### DirectionalArmor - true
### RuggedTerrain - false
### Crushable - false
### CanTurnInPlace - true
### CanReverse - true
### MovementModel - SpeedTurnRadiusMovement

## DrillUnit
Underground variant of a tracked vehicle. Operates underground. **While underground it is hidden** from other players unless **revealed by a detector**, and can only be targeted while revealed (user, 2026-11-25; replaces "visible while subterranean"; detection TBD). **Area (AoE) damage still affects it** underground (user, 2026-11-25, Q0071): see systems/combat.md > AoEDamage. Can travel across all drillable tiles while underground. Cannot fire while underground. Has an above-ground mode in which it is stationary or behaves as tracked. Has directional armor with frontal damage reduction and rear damage increase. Typically has wide to full turret turn angles.

### Domain - Underground
### CanHaveTurret - true
### DirectionalArmor - true
### RuggedTerrain - false
### Crushable - false
### CanTurnInPlace - true
### CanReverse - true
### MovementModel - SpeedTurnRadiusMovement

## HoverVehicle
Ground vehicle that hovers above the surface. Accelerates omni-directionally with thrust and uses a combination of drag and opposing thrust to decelerate and change direction. Feels momentum-heavy and slidey. Has directional armor with frontal damage reduction and rear damage increase. Typically has wide to full turret turn angles.

### Domain - Ground
### CanHaveTurret - true
### DirectionalArmor - true
### RuggedTerrain - true, with **double drag**: while over Rugged terrain, the HoverVehicle's DragRatio is doubled (halving its effective MaxSpeed and making it decelerate faster). No defense bonus. (User, 2026-11-23.)
### Crushable - false
### CanTurnInPlace - true
### CanReverse - false
### MovementModel - DragMovement

## Mech
Large ground walker that can traverse rugged terrain. Crushes enemy Light Infantry. Has directional armor with frontal damage reduction and rear damage increase. Typically has very narrow to narrow turret turn angles.

### Domain - Ground
### CanHaveTurret - true
### DirectionalArmor - true
### RuggedTerrain - true
### Crushable - false
### CanTurnInPlace - true
### CanReverse - false
### MovementModel - TurnRateMovement

## HoverCraft
Air unit that can hover in place (hold a stationary position in the air). Relies on high drag to decelerate passively. Can turn in place or while moving. Typically has very narrow to narrow turret turn angles.

### Domain - Air
### CanHaveTurret - true
### DirectionalArmor - false
### RuggedTerrain - false
### Crushable - false
### CanTurnInPlace - true
### CanReverse - false
### MovementModel - DragMovement

## Glider
Air unit that must always maintain movement to stay airborne. Circles at idle speed when not given orders. Cannot turn in place or stop. Turn radius widens at higher speeds due to centripetal acceleration constraints. Typically has very narrow to narrow turret turn angles.

### Domain - Air
### CanHaveTurret - true
### DirectionalArmor - false
### RuggedTerrain - false
### Crushable - false
### CanTurnInPlace - false
### CanReverse - false
### MovementModel - GliderMovement

## TurretAttributes
Physical rotation properties of a unit's turret. Only present on units whose UnitBase has CanHaveTurret = true; a unit of such a base may still have TurretAttributes = None (no turret). The turret is centered on the unit's facing direction, with TurnAngle split equally clockwise and counter-clockwise from center.

### TurnAngle - number (degrees, full arc, max 360)
### TurnRate - number
