# Controls: Unit Commands

## Unit Command
An order issued to a unit, produced by a CommandIssuingTransition from the Control System. A unit maintains a command queue; the current command is dequeued and sets the BaseCommandState, which drives the unit's behavior. When the current command's behavior completes, the next command is dequeued. Commands are queued by holding **Shift** while issuing them (right-click, command hotkey + target click, etc.): the new command is appended to the end of the queue instead of replacing it. Without Shift, issuing a command replaces the whole queue (user, 2026-11-23). Commands only mutate BaseCommandState — TurretCommandState is managed by the base behavior.

### Value - UnitCommandEnum

## Group Destinations (Magic Box)
When a location-targeted movement order (Move, Attack Move, Patrol) is given to several units at once, each unit's destination is decided by the **magic box** (user, 2026-11-23; StarCraft's model, the user's reference):
- **The box** is the bounding rectangle around the **full Silhouettes** (collision shapes) of the units receiving the order, not just their centre points, so a click on or between the units of a straight line counts as inside. The group's centre (used for offsets) is the centre of this box (user, 2026-11-23).
- **Click outside the box: keep the arrangement.** Each unit's destination is the click point plus that unit's current offset from the group's center. The group arrives in the same shape it had (a line stays a line).
- **Click inside the box: converge.** Every unit's destination is the clicked point itself. The units gather around it, stopping where the crowd blocks them (see systems/controls/behaviors.md > MovingToLocation for when a blocked unit stops).
- **Queued (Shift) group orders: the box is built from where the units will be** (user, 2026-11-23). For a Shift-queued order, each unit's position for the box and offsets is the **destination of its last queued movement order** (the place it will be standing when this order starts), not its current position. Example: queue a group to X, then back to the start. The return trip is outside the box around X, so the group returns in formation instead of converging. If the previous queued move was a converge, all planned destinations are one point and the future box is tiny, so the next queued click moves the group on as the blob it will be.
  - This assumes every unit reaches its planned spot. If a unit falls short (blocked, stuck, crowd), it still walks to its pre-computed spot on the next leg and rejoins the shape there.
  - Fallback: a unit with no queued order that ends at a known location (empty queue, or its last queued order targets an object, e.g. Attack a unit or Move to an object) uses its current position.
- **Destinations are resolved when the order is given** (user, 2026-11-23). At the moment of the click, the game computes every unit's own destination (offset point, or the converge point) for the whole group and stores that **resolved destination** in each unit's command queue. A unit later executes it alone, whenever it reaches that point in its queue, with no recalculation. The group shape therefore appears at each destination even though units travel and start legs individually. (Recomputing when each unit starts the leg would break this: each unit would be a group of one and everyone would converge on the click point.)
- **No formation movement:** each unit travels to its destination at its own speed. The group does not wait for its slowest member and strings out if speeds differ.

Rejected alternative (user): a fresh compact grid of spots around the click point. It felt "a little bit too random from a player's perspective".

An offset destination that is unreachable is handled like any unreachable location: the unit goes to the nearest reachable point (systems/controls/behaviors.md > MovingToLocation > Unreachable target location).

TBD: whether the box has a size limit (for widely scattered selections the box covers most of the map, so most clicks converge).

## Move
Order the unit to move to a target location or object. Moving to a **unit** makes the ordered unit **follow** it until given another order (systems/controls/behaviors.md > MovingToObject > Completion). Right-clicking one of your own units therefore makes the selected units follow it.

### Sets BaseCommandState:
- CommandType = Move
- TargetLocation = location | None
- TargetObject = ObjectInstance | None (one of TargetLocation or TargetObject must be set)

## Attack
Order the unit to attack a specific destructible object instance (unit or structure).

### Sets BaseCommandState:
- CommandType = Attack
- TargetLocation = None
- TargetObject = Destructible ObjectInstance

## AttackGround
Order the unit to attack a specific ground location. Only available to units whose AttackType has CanTargetGround = true.

### Sets BaseCommandState:
- CommandType = AttackGround
- TargetLocation = location
- TargetObject = None

## AttackMove
Order the unit to move to a target location, engaging enemies encountered along the way.

### Sets BaseCommandState:
- CommandType = AttackMove
- TargetLocation = location
- TargetObject = None

## Patrol
Order the unit to move back and forth between its location **when the Patrol starts executing** (for a queued Patrol: where the previous order ended) and a target location, engaging enemies encountered along the way.

### Sets BaseCommandState:
- CommandType = Patrol
- TargetLocation = location
- TargetObject = None

## HoldPosition
Order the unit to remain stationary. The unit will not move for any reason.

### Sets BaseCommandState:
- CommandType = HoldPosition
- TargetLocation = None
- TargetObject = None

## Stop
Order the unit to cease all current activity.

### Sets BaseCommandState:
- CommandType = Stop
- TargetLocation = None
- TargetObject = None

## Reverse
Order the unit to reverse to a target location. Only available to bases with CanReverse = true (WheeledVehicle, TrackedVehicle, DrillUnit).

### Sets BaseCommandState:
- CommandType = Reverse
- TargetLocation = location
- TargetObject = None

## Enter
Order the unit to enter an own structure that can hold units. The unit walks to the structure's entrance side and goes inside. Valid targets:

- **Syndicate — Tunnel**: any Syndicate unit, when the target Tunnel's tier is sufficient for the unit's base category. The unit walks to Side A and enters the Tunnel Network.
- **Cults — Altar**: any Cults infantry (Recruit, Soldier, Gunner, Acolyte), up to the Altar's InternalUnitCapacity.
- **Cults — Armory, Cathedral**: Recruits only (to be trained), up to the building's internal capacity.
- **Cults — Temple, Gunworks, Machine Shop**: Recruits only (to staff research).
- **Cults — Shrine**: Recruits only (to join the ritual).
- **Cults — Garrison**: Recruits, Soldiers, Gunners, and Acolytes (up to 4; Recruits and Acolytes fire as Soldiers; Soldiers and Gunners get +1 range).
- **GDO — Bunker**: all GDO infantry (up to 4; fire their own weapons, no range bonus).

Each faction's unit and building docs define the right-click resolution and capacity details (factions/syndicate/, factions/cults/).

### Sets BaseCommandState:
- CommandType = Enter
- TargetLocation = None
- TargetObject = Tunnel (ObjectInstance)

## BaseCommandState[UnitBase]
The parameters set by the currently executing command that define the unit's objective. Set when a command is dequeued and executed. Read by the base behavior to determine what the unit should be doing. Parameterized by UnitBase as different bases may require different command parameters.

### CommandType - UnitCommandEnum
### TargetLocation - Coordinates | None
### TargetObject - ObjectInstance | None

## TurretCommandState
The parameters that define the turret's current objective. Set by the base behavior (not directly by commands). When the base behavior identifies a target, it updates the TurretCommandState. When the base behavior does not specify a target, the turret retains its previous TurretCommandState. If the turret has no assigned target, it falls back to autonomous target scanning.

### LockedTarget - ObjectInstance | None
