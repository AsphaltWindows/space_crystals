# Control System

## ControlState
The client-side state of a player's control interface. Contains the player's current Selection and the ObjectInterfaceState for the active selection. ControlState is not part of the game simulation — it lives entirely on the client. Each tick, ControlState is validated against the current game state: selected objects that no longer exist are removed from the Selection, and if the ObjectInterfaceState becomes invalid as a result, it resets to the default state for the current selection.

### Selection - Selection
### ObjectInterfaceState[ActiveGroup] - ObjectInterfaceState[ObjectEnum]

## Selection
The set of ObjectInstances currently selected by the player, grouped by object type. Each group contains references to all selected instances of that type. The ActiveGroup determines which group's commands are displayed in the command panel and which ObjectInterfaceState is active.

### Groups - array of SelectionGroup
### ActiveGroup - ObjectEnum | None (None when selection is empty)

### Constraints:
- If any selected object is not owned by the player, the selection must contain exactly one object and one group
- If all selected objects are owned by the player, no limit on count or mixing of types
- Ungroupable objects (Groupable = false) always occupy their own SelectionGroup, even when selected alongside other instances of the same type
- ActiveGroup must be a type that exists in the current Groups

## SelectionGroup
A group of selected ObjectInstances that share the same object type. For Groupable objects, all selected instances of that type are combined into one group. For Ungroupable objects, each instance is its own group.

### Type - ObjectEnum
### Instances - array of ObjectInstance references

## BoxSelection
When the player drags a selection box, the objects captured are filtered by a 5-tier priority system. The highest-priority tier containing any objects wins; all lower tiers are excluded.

### Priority tiers (highest to lowest):
1. Own units — all own units in the box are selected (multi-select)
2. Own buildings — single-select: the own building closest to the box center
3. Enemy units — single-select: the enemy unit closest to the box center
4. Enemy buildings — single-select: the enemy building closest to the box center
5. Neutral objects — single-select: the neutral object closest to the box center

Only tier 1 (own units) produces a multi-selection. All other tiers select exactly one object — the one closest to the center of the box.

## ControlGroups
A set of 10 saved selections (indexed 0-9) stored per player as part of ControlState. Each control group holds an array of ObjectInstance references. An entity can belong to multiple control groups simultaneously. Control groups persist for the duration of the game. When a control group is recalled, destroyed entities are silently removed from the group.

### Groups - array of 10 ObjectInstance reference arrays

### Operations:
- Assign: replace a control group's contents with the current Selection
- Add: merge the current Selection into an existing control group (no duplicates)
- Recall: replace the current Selection with the control group's contents
- Recall and Center: replace the current Selection with the control group's contents and center the camera on the group's centroid

## ObjectInterfaceState[ObjectEnum]
The current state of the command flow for the active selection group. Parameterized by ObjectEnum — each object type defines its own concrete set of interface states and transitions. Reset to the default state when the Selection or ActiveGroup changes. The available InterfaceTransitions from any given ObjectInterfaceState depend on the game state of the instances in the active SelectionGroup. Each tick, the ObjectInterfaceState is validated against the active SelectionGroup's game state — if the current state is no longer valid (e.g., a structure's construction completed, changing what commands are available), it resets to the default state. For Ungroupable objects, the SelectionGroup always contains exactly one instance, so the ObjectInterfaceState can reliably read that instance's specific game state.

## InterfaceTransition
A transition from one ControlState to another, triggered by player input. Each InterfaceTransition either only modifies the ControlState or also issues a Command to the selected objects.

### Value - InterfaceTransitionEnum

## StateOnlyTransition
An InterfaceTransition that only modifies ControlState. No Command is issued to any game object. Examples: pressing an attack hotkey to enter target selection, opening a build menu, cycling the active group.

## CommandIssuingTransition
An InterfaceTransition that modifies ControlState and also issues a Command to one or more selected objects. Examples: left-clicking a target after pressing attack, confirming a building placement.

## GroupCycling
A StateOnlyTransition that changes the ActiveGroup to an adjacent SelectionGroup in the Selection.

### Forward (Tab): advance ActiveGroup to the next SelectionGroup. Wraps around to the first group after the last.
### Backward (Shift-Tab): advance ActiveGroup to the previous SelectionGroup. Wraps around to the last group before the first.

## SelectionPanel
Displays a grid of unit portraits for the current selection. Only visible when 2 or more units are selected. Each cell shows the portrait of one selected unit instance. Portraits belonging to the ActiveGroup are shown with a sheer highlight.

### Visibility:
- Visible when Selection contains 2+ units
- Hidden when Selection contains 0 or 1 units (single-unit display is handled by the InfoPanel)

### Interactions:

| Input | Effect |
|-------|--------|
| Left-click portrait | Replace selection with only that unit |
| Shift-click portrait | Remove that unit from selection |
| Ctrl-click portrait | Replace selection with all units of that type in the current selection |
| Ctrl-Shift-click portrait | Remove all units of that type from selection |
| Alt-click portrait | Center camera on that unit (no selection change) |

### Edge cases:
- If an action reduces the selection to 1 unit, the SelectionPanel hides and the InfoPanel displays that unit
- If an action removes all remaining units, the selection becomes empty

## CommandPanel
The display of available commands for the current selection, derived from ControlState and game state each tick. The CommandPanel is only displayed when all objects in the Selection are owned by the player. When selecting enemy or neutral objects, only the InfoPanel is shown — no commands are available, no right-click resolution is performed, and no InterfaceTransitions can be initiated. Commands can only be issued to objects owned by the player.

When visible, commands available to all objects in the Selection are visually distinguished from commands available only to the ActiveGroup. Clicking a common command issues it to all selected objects. Clicking a group-specific command issues it only to objects in the ActiveGroup.

### CommonCommands - commands available to every object in Selection
### GroupCommands - commands available to objects of type ActiveGroup

### Grid Layout
The CommandPanel is a 3x3 grid of command slots. Each slot has a default hotkey:

```
| Q | W | E |
| A | S | D |
| Z | X | C |
```

### Standard Slot Assignments
Certain slots have standardized functions across all object types that use them:

- **Z (bottom-left)**: Back / Cancel menu. In any multi-stage menu (BuildMenu, ExpandMenu, EjectMenu, AwaitingTarget, AwaitingPlacement), Z returns to the previous state (StateOnlyTransition). Equivalent to Escape or right-click cancel.
- **X (bottom-center)**: Cancel Production / Cancel Upgrade. In production buildings, cancels the last queued item and refunds cost. In structures with upgrades, cancels the in-progress upgrade and refunds cost.
- **C (bottom-right)**: Set Rally Point. In unit-producing structures, enters AwaitingTarget[SetRallyPoint]. Left-click ground or object sets the rally point (CommandIssuingTransition, returns to DefaultState).

### Production Building Default Right-Click
For all unit-producing structures, right-click from DefaultState sets the rally point:
- Right-click Ground: sets rally point to that location
- Right-click Object: sets rally point to that object

## DefaultState
The default ObjectInterfaceState for a selection. The command panel displays available commands. Right-click performs a context-sensitive command based on what is under the cursor. Command hotkeys or button clicks initiate transitions to AwaitingTarget or issue immediate commands.

## RightClickResolution
Context-sensitive command issued by right-clicking from DefaultState. The command is determined by what is under the cursor and what the selected objects can do. Resolution is evaluated against the ActiveGroup's capabilities.

## AwaitingTarget[CommandType]
An ObjectInterfaceState entered after the player selects a command that requires a target. The command panel indicates which command is pending. Left-clicking a valid target issues the command via a CommandIssuingTransition and returns to DefaultState. The target may resolve to a different command than the one that initiated the state, depending on what is under the cursor. Escape or right-click cancels via a StateOnlyTransition back to DefaultState.

### PendingCommand - CommandType
### Resolution - (CursorTarget) -> Command

## CursorTarget
What is under the player's cursor at the time of a click. Used by AwaitingTarget and RightClickResolution to determine which command to issue.

### Value - CursorTargetEnum (Ground | EnemyObject | FriendlyObject | NeutralObject)
### Location - Coordinates
### Object - ObjectInstance | None

## PointerDisplayType
The visual appearance of the player's cursor, communicating what action will occur on click. Determined each tick by the current ControlState, the ActiveGroup's capabilities, and the CursorTarget. Only one PointerDisplayType is active at a time.

### Types:

| Type | Visual Feel | Description |
|------|------------|-------------|
| Inactive | Muted, subdued | Nothing will happen on click. Shown when nothing is selected, or when hovering over an invalid target for the current context. |
| Move | — | Unit will move to this location or object. |
| Attack | — | Unit will attack this target. |
| AttackGround | — | Unit will attack this ground location. |
| Patrol | — | Unit will patrol to this location. |
| GatherResources | — | Unit will gather from this resource. |
| ReturnResources | — | Unit will return carried resources to this drop-off point. |
| Enter | — | Unit will enter this object (e.g., Syndicate unit entering a Tunnel). |

### Resolution Rules

PointerDisplayType is resolved based on the current ObjectInterfaceState:

#### DefaultState (right-click preview):
The pointer previews the action that right-click would perform, based on the ActiveGroup's RightClickResolution:

- Nothing selected: **Inactive**
- Cursor over enemy object (and selection can attack): **Attack**
- Cursor over ground (and selection can move): **Move**
- Cursor over friendly/neutral object (and selection can move): **Move**
- Cursor over own Tunnel (Syndicate unit, tier sufficient): **Enter**
- Cursor over resource (resource gatherer selected): **GatherResources**
- Cursor over drop-off point (resource gatherer carrying resources): **ReturnResources**
- Selection is a production building: **Move** (rally point)
- No valid action for current cursor target: **Inactive**

#### AwaitingTarget[CommandType]:
The pointer reflects the pending command and whether the current CursorTarget is valid:

- AwaitingTarget[Attack] over enemy object: **Attack**
- AwaitingTarget[Attack] over ground: **Attack** (will issue AttackMove)
- AwaitingTarget[Attack] over friendly/neutral object: **Inactive**
- AwaitingTarget[Move] over ground or any object: **Move**
- AwaitingTarget[Patrol] over ground: **Patrol**
- AwaitingTarget[Patrol] over non-ground: **Inactive**
- AwaitingTarget[AttackGround] over ground: **AttackGround**
- AwaitingTarget[AttackGround] over non-ground: **Inactive**
- AwaitingTarget[Reverse] over ground: **Move**
- AwaitingTarget[Reverse] over non-ground: **Inactive**
- AwaitingTarget[ScheduleDeliveries] over valid target: **GatherResources**
- AwaitingTarget[ScheduleDeliveries] over invalid target: **Inactive**
- AwaitingTarget[SetRallyPoint] over ground or object: **Move**

#### AwaitingPlacement:
No pointer display type — the building ghost preview serves as the cursor.

## BasicCombatUnitInterfaceState
The ObjectInterfaceState template used by combat units. Defines the available commands, right-click resolution, and awaiting-target resolution rules common to most combat units.

### DefaultState commands:

Grid layout:
```
[Q] Move      [W] Reverse*    [E] HoldPosition
[A] Attack    [S] Patrol      [D] AttackGround*
[Z] —         [X] Stop        [C] —
```
*Conditional: W (Reverse) only available if UnitBase has CanReverse = true. D (AttackGround) only available if AttackType has CanTargetGround = true.*

Immediate commands (CommandIssuingTransition, no target required):
- **E: HoldPosition**: issues HoldPosition command
- **X: Stop**: issues Stop command

Target commands (StateOnlyTransition to AwaitingTarget):
- **A: Attack**: enters AwaitingTarget[Attack]
- **Q: Move**: enters AwaitingTarget[Move]
- **S: Patrol**: enters AwaitingTarget[Patrol]
- **D: AttackGround**: enters AwaitingTarget[AttackGround] (only available if unit's AttackType has CanTargetGround = true)
- **W: Reverse**: enters AwaitingTarget[Reverse] (only available if unit's UnitBase has CanReverse = true)

### RightClickResolution:
- Cursor over EnemyObject: issues Attack command targeting that object
- Cursor over Ground: issues Move command to that location
- Cursor over own Tunnel (Syndicate units only, tier sufficient): issues Enter command targeting that Tunnel
- Cursor over FriendlyObject or NeutralObject (all other cases): issues Move command to that object

### AwaitingTarget[Attack] resolution:
- Left-click EnemyObject: issues Attack command targeting that object
- Left-click Ground: issues AttackMove command to that location

### AwaitingTarget[Move] resolution:
- Left-click Ground: issues Move command to that location
- Left-click any Object: issues Move command to that object

### AwaitingTarget[Patrol] resolution:
- Left-click Ground: issues Patrol command to that location

### AwaitingTarget[AttackGround] resolution:
- Left-click Ground: issues AttackGround command to that location

### AwaitingTarget[Reverse] resolution:
- Left-click Ground: issues Reverse command to that location

## CommandIndicators
Visual markers displayed at command targets for selected units. Indicators are only visible when a unit or building with that active command is part of the current Selection. All active command indicators in the selection are shown simultaneously. Indicators are removed when the unit is deselected or the command completes.

### Location Indicator
A marker on the ground at the target coordinates.

### Object Indicator
A marker surrounding the target object's perimeter.

### Indicator assignments:

| Command | Indicator Type | Color |
|---------|---------------|-------|
| Move | Location | Green |
| Attack | Object | Red |
| AttackMove | Location | Orange |
| AttackGround | Location | Red |
| Patrol (origin) | Location | Orange |
| Patrol (destination) | Location | Orange |
| Reverse | Location | Green |
| Enter | Object | Green |

### Color language:
- Green = peaceful movement
- Red = hostile target
- Orange = aggressive movement

## Unit Command
An order issued to a unit, produced by a CommandIssuingTransition from the Control System. A unit maintains a command queue; the current command is dequeued and sets the BaseCommandState, which drives the unit's behavior. When the current command's behavior completes, the next command is dequeued. Commands can be queued by the player (e.g., shift-click). Commands only mutate BaseCommandState — TurretCommandState is managed by the base behavior.

### Value - UnitCommandEnum

## Move
Order the unit to move to a target location or object.

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
Order the unit to move back and forth between its current location and a target location, engaging enemies encountered along the way.

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
Order the unit to enter a Tunnel Network via a specific Tunnel. The unit walks to Side A of the target Tunnel and enters the network. Only available to Syndicate units when the target Tunnel's tier is sufficient for the unit's base category.

### Sets BaseCommandState:
- CommandType = Enter
- TargetLocation = None
- TargetObject = Tunnel (ObjectInstance)

## BaseCommandState[UnitBase]
The parameters set by the currently executing command that define the unit's objective. Set when a command is dequeued and executed. Read by the base behavior to determine what the unit should be doing. Parameterized by UnitBase as different bases may require different command parameters.

### CommandType - UnitCommandEnum
### TargetLocation - Coordinates | None
### TargetObject - ObjectInstance | None

## BaseBehaviorState[UnitBase]
Internal data maintained by the base behavior as it executes its algorithm in response to the BaseCommandState. Includes computed data such as planned paths, progress tracking, and any other cached information the behavior needs across frames. Parameterized by UnitBase as different bases may need to store different kinds of intermediate results (e.g., a Glider's circling parameters vs. a Wheeled Vehicle's turn radius planning).

## TurretCommandState
The parameters that define the turret's current objective. Set by the base behavior (not directly by commands). When the base behavior identifies a target, it updates the TurretCommandState. When the base behavior does not specify a target, the turret retains its previous TurretCommandState. If the turret has no assigned target, it falls back to autonomous target scanning.

### LockedTarget - ObjectInstance | None

## TurretBehaviorState
Internal data maintained by the turret behavior as it executes its algorithm in response to the TurretCommandState. May include scan state, last known target position, or other cached information.

## BaseBehavior
The algorithm the unit base executes based on its BaseCommandState, in response to game circumstances. Reads and writes BaseBehaviorState as internal working data. Composed of a sequence of base actions. Also responsible for updating TurretCommandState when appropriate (e.g., assigning a target for the turret to engage).

## TurretBehavior
The algorithm the unit's turret executes based on its TurretCommandState. When the turret has a locked target, it engages that target. When the turret has no locked target, it autonomously scans for and engages the best available target within range and turn angle. Reads and writes TurretBehaviorState as internal working data. Composed of a sequence of turret actions.

## TurretAutonomousScanning
When the turret has no locked target (TurretCommandState.LockedTarget = None), it autonomously selects the best available target from ValidTarget enemies within weapon range and turret arc.

### Target Selection Priority:
1. **Threatening units first**: Prefer ValidTargets whose AttackAttributes.TargetDomain includes this unit's domain (Ground, Air, or Universal). These are units that have the capability to attack this unit.
2. **Least rotation**: Among equal-priority targets, prefer the target requiring the least turret rotation from the turret's current facing.
3. **Closest distance**: Among targets requiring equal rotation, prefer the closest target by distance.

### Algorithm:
1. Gather all ValidTarget enemies within Range and within TurnAngle arc
2. Partition into threatening (can target this unit's domain) and non-threatening
3. From the highest-priority non-empty partition, select the target with least required turret rotation, breaking ties by closest distance
4. Set TurretCommandState.LockedTarget to the selected target
5. If no valid targets exist, turret remains inactive (TurretAttackChannel = TurretInactive)

## BaseAutoTargeting
When the unit has no active behavior (idle) or is executing HoldingPosition, the base autonomously scans for and engages ValidTarget enemies. For turret units, the base sets TurretCommandState.LockedTarget. For non-turret units (infantry), the base engages directly via BaseAttackChannel.

### Target Selection Priority:
Same as TurretAutonomousScanning: threatening targets first, then least rotation (unit facing for infantry, turret angle for turret units), then closest distance.

### Active during Idle:
When a unit has no active command (command queue empty), it scans for ValidTarget enemies within SightRange. On acquiring a target, the unit switches to an AttackingObject sub-behavior targeting the acquired enemy. The unit's position at the time of acquisition is recorded as the IdleOrigin. If the unit's distance from IdleOrigin exceeds IdleLeashDistance, the unit disengages and returns to IdleOrigin via MovingToLocation. If the target is destroyed or leaves SightRange, the unit returns to idle at its current position (new IdleOrigin).

### Global Constant: IdleLeashDistance - 4 grid units

### Active during HoldPosition:
Same target scanning as idle, but Locomotion is locked to Stationary. The unit never chases. For turret units, the turret engages within range and arc. For non-turret units with CanTurnInPlace, Orientation = Turning(enemy position) and engage via BaseAttackChannel. For non-turret units without CanTurnInPlace, can only engage targets already in the unit's current facing and weapon range. If the target leaves range, disengage immediately.

### Not active during:
- Move — pure movement, no base engagement. Turret units' TurretAutonomousScanning still operates independently via TurretBehavior, but the base does not acquire targets or set TurretCommandState.LockedTarget.
- AttackTarget — already has an explicit target
- AttackGround — deliberate location-targeted attack, auto-acquiring a different target would override player intent
- Stop — halting all activity

### Not applicable to:
- AttackMove, Patrol — these behaviors have their own explicit enemy scanning logic in their behavior algorithms

## MovingToLocation
Base behavior for moving to a static target location.

### Reads: BaseCommandState.TargetLocation

### BaseBehaviorState contains:
- Plan: sequence of (Locomotion, Orientation) channel state pairs, respecting LocomotionOrientationConstraints[MovementModel]
- CurrentStep: index into the plan
- ExpectedPosition: the unit's expected position at the current step

### Algorithm:
1. Compute path from unit's current position to TargetLocation
2. Translate path into a plan of (Locomotion, Orientation) pairs
3. Store plan in BehaviorState, set CurrentStep = 0, compute ExpectedPosition
4. Each tick:
   - If unit position deviates from ExpectedPosition: recompute plan from step 1
   - If current step is complete: advance CurrentStep, update ExpectedPosition
   - Set action channels to the current step's values
5. Final step: Locomotion = Stopping, Orientation = Maintaining
6. When stopped: behavior complete

### GliderMovement exception:
Final step is Locomotion = Moving(idle circle over TargetLocation), Orientation = Turning(circle path). Behavior does not complete — continues circling until a new command is issued.

### Failure:
If the unit repeatedly fails to make progress toward TargetLocation, set Locomotion = Stopping. When stopped, behavior complete.

## MovingToObject
Base behavior for moving to a mobile or static object instance. Similar to MovingToLocation but the target position may change over time, requiring smarter replanning.

### Reads: BaseCommandState.TargetObject

### BaseBehaviorState contains:
- Plan: sequence of (Locomotion, Orientation) channel state pairs
- CurrentStep: index into the plan
- ExpectedPosition: the unit's expected position at the current step
- Additional intermediate state for optimizing recomputation when TargetObject moves (to be defined)

### Algorithm:
Same as MovingToLocation, except:
- Target position is derived from TargetObject's current location
- Recomputation is also triggered when TargetObject has moved sufficiently from its position at plan computation time
- Optimization of recomputation frequency is stubbed out for future refinement

### Completion:
Behavior complete when the unit is within proximity of the TargetObject.

### Failure:
Same as MovingToLocation.

## AttackingObject
Base behavior for attacking a destructible object instance. The unit moves toward the target and engages when the target becomes attackable.

### Reads: BaseCommandState.TargetObject

### BaseBehaviorState contains:
- MovingToObject sub-behavior state (for approach)
- AttackPhaseState: current phase of the attack sequence when engaged

### Algorithm:
1. Begin MovingToObject toward TargetObject
2. Each tick: check if TargetObject is within Range, beyond MinRange, and within turret arc (turret units) or unit facing (infantry)
3. When targetable:
   - Locomotion = Stopping, Orientation = Maintaining (or Turning toward target for infantry)
   - Begin attack sequence on BaseAttackChannel (infantry) or set TurretCommandState.LockedTarget = TargetObject (turret units)
4. If target moves out of range or arc: resume MovingToObject toward TargetObject
5. If target is destroyed: behavior complete

### GliderMovement exception:
Gliders cannot stop. Instead of stopping when in range, the Glider performs strafing runs:
1. Fly toward TargetObject
2. When in range and turret arc: fire while passing through
3. Continue past the target
4. Loop around for another pass
5. Repeat until target is destroyed or a new command is issued

### Failure:
If the unit repeatedly fails to make progress toward the target, behavior complete.

## AttackingLocation
Base behavior for attacking a ground location. Only available to units whose AttackType has CanTargetGround = true.

### Reads: BaseCommandState.TargetLocation

### BaseBehaviorState contains:
- MovingToLocation sub-behavior state (for approach)
- AttackPhaseState: current phase of the attack sequence when engaged

### Algorithm:
Same as AttackingObject, except:
- Target position is BaseCommandState.TargetLocation (static)
- Uses MovingToLocation sub-behavior instead of MovingToObject
- Range/arc check is against the target location
- TurretCommandState.LockedTarget is not set (turret fires at location, not a unit)
- Behavior complete after attack effect is applied

### GliderMovement exception:
Same strafing run pattern as AttackingObject, targeting the ground location.

## ReversingToLocation
Base behavior for reversing to a static target location. Only available to bases with CanReverse = true. Identical to MovingToLocation except the plan uses Reversing locomotion instead of Moving.

### Reads: BaseCommandState.TargetLocation

### BaseBehaviorState contains:
- Same as MovingToLocation

### Algorithm:
Same as MovingToLocation, except:
- Plan uses Reversing(path) instead of Moving(path)
- LocomotionOrientationConstraints[MovementModel] are evaluated for Reversing combinations

### GliderMovement: not applicable (CanReverse = false)

### Failure:
Same as MovingToLocation.

## AttackMovingToLocation
Base behavior for moving to a target location while engaging enemies encountered along the way. The unit follows its path but breaks off to attack enemies that enter its SightRange, returning to the path if the chase strays too far.

### Reads: BaseCommandState.TargetLocation

### Global Constant: AttackMoveLeashDistance - 6 grid units

### BaseBehaviorState contains:
- MovingToLocation sub-behavior state (for pathing to TargetLocation)
- AttackingObject sub-behavior state (when engaged with a target)
- CurrentTarget: ObjectInstance | None
- PathReference: the planned path to TargetLocation (used for leash distance calculation)

### Algorithm:
1. Begin MovingToLocation toward TargetLocation
2. Each tick while moving:
   - Scan for ValidTarget enemies within SightRange
   - If enemy detected: set CurrentTarget, switch to AttackingObject sub-behavior targeting CurrentTarget
   - For turret units: set TurretCommandState.LockedTarget = CurrentTarget
3. While engaged with CurrentTarget:
   - If unit's perpendicular distance from PathReference exceeds AttackMoveLeashDistance: disengage, clear CurrentTarget, resume MovingToLocation from current position toward TargetLocation (recompute path)
   - If CurrentTarget is destroyed: clear CurrentTarget, resume MovingToLocation from current position toward TargetLocation (recompute path)
4. When unit arrives at TargetLocation: behavior complete

### GliderMovement exception:
Glider follows its path, performing strafing runs on enemies within SightRange. Leash is measured the same way. On arrival at TargetLocation, transitions to idle circling.

### Failure:
Same as MovingToLocation.

## Patrolling
Base behavior for moving back and forth between two locations while engaging enemies encountered along the way. Combines AttackMovingToLocation with automatic waypoint cycling.

### Reads: BaseCommandState.TargetLocation (patrol destination), unit's position at time of command (patrol origin)

### BaseBehaviorState contains:
- AttackMovingToLocation sub-behavior state
- PatrolOrigin: Coordinates (unit's position when the Patrol command was issued)
- PatrolDestination: Coordinates (BaseCommandState.TargetLocation)
- CurrentLeg: origin-to-destination | destination-to-origin

### Algorithm:
1. Set PatrolOrigin = unit's current position, PatrolDestination = TargetLocation
2. Begin AttackMovingToLocation toward PatrolDestination
3. When AttackMovingToLocation completes (arrived at destination): swap legs, begin AttackMovingToLocation toward the other endpoint
4. Repeat indefinitely until a new command is issued

### Behavior never completes on its own — continues cycling until interrupted by a new command.

## HoldingPosition
Base behavior for remaining stationary. The unit does not move for any reason but will turn and fight if able.

### Reads: BaseCommandState (no target parameters needed)

### Algorithm:
1. Locomotion = Stationary at all times
2. For turret units: TurretCommandState is unchanged — turret continues autonomous scanning and engaging targets within range and arc
3. For non-turret units with CanTurnInPlace = true: scan for ValidTarget enemies within weapon range. If enemy detected, Orientation = Turning(enemy position), engage via BaseAttackChannel
4. For non-turret units with CanTurnInPlace = false: can only engage ValidTarget enemies that happen to be in the unit's current facing and weapon range
5. Behavior never completes — continues until a new command is issued

## StoppingBehavior
Base behavior for ceasing all current activity.

### Reads: BaseCommandState (no target parameters needed)

### Algorithm:
1. Locomotion = Stopping, Orientation = Maintaining
2. Clear TurretCommandState.LockedTarget (turret falls back to autonomous scanning)
3. When stopped: behavior complete, unit has no active behavior

## BaseActionChannels
The unit base operates on three concurrent action channels. Each channel has one active value at a time. Valid combinations of channel states are constrained by UnitBase and current attack phase.

## LocomotionChannel
Controls the unit base's movement.

### Value - LocomotionEnum

## Moving
The unit base is moving along a path from BehaviorState.

### Path - from BaseBehaviorState

## Reversing
The unit base is moving backward along a path. Only available to bases with CanReverse = true (WheeledVehicle, TrackedVehicle, DrillUnit).

### Path - from BaseBehaviorState

## Stopping
The unit base is slowing down to a stop.

## Stationary
The unit base is not moving.

## OrientationChannel
Controls the unit base's facing direction.

### Value - OrientationEnum

## Turning
The unit base is rotating toward a target position. The engine computes the required angle and applies the base's turn rate each tick. For FixedTurnRadiusMovement bases, orientation is constrained by locomotion.

### TargetPosition - Coordinates

## Maintaining
The unit base is holding its current facing direction.

## BaseAttackChannel
Controls the unit base's attack state. Only active for non-turret units. For turret units this channel is always None.

### Value - BaseAttackEnum | None

## BaseAiming
The unit base is aiming at an attack target. Orientation channel is overridden to face the target. Locomotion must be Stationary. If the unit is given a different command, the attack sequence is interrupted.

### Target - AttackTarget

## BaseFiring
The unit base is executing the firing phase. Locomotion must be Stationary. Orientation is locked.

### Target - AttackTarget

## BaseCooldown
The unit base is in the cooldown phase after firing. Locomotion must be Stationary. Orientation is locked.

## BaseReloading
The unit base is in the reloading phase between attacks. Locomotion and Orientation are free.

## TurretActionChannels
The turret operates on two concurrent action channels, running independently of the base action channels. Only present on turret-bearing units.

## TurretOrientationChannel
Controls the turret's facing direction, independent of the unit base's orientation.

### Value - TurretOrientationEnum

## TurretTurning
The turret is rotating toward a target position. The engine computes the required angle and applies the turret's turn rate each tick, constrained by TurnAngle.

### TargetPosition - Coordinates

## TurretMaintaining
The turret is holding its current facing direction.

## TurretAttackChannel
Controls the turret's attack state.

### Value - TurretAttackEnum

## TurretAiming
The turret is aiming at an attack target. TurretOrientation is overridden to face the target.

### Target - AttackTarget

## TurretFiring
The turret is executing the firing phase of its attack sequence.

### Target - AttackTarget

## TurretCooldown
The turret is in the cooldown phase after firing.

## TurretReloading
The turret is in the reloading phase between attacks.

## TurretInactive
The turret is not performing any attack action.
