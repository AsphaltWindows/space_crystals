# Feature: Control System

## Overview
Client-side player control interface: selection, control groups, object interface states, command panel, and interface transitions. Entirely outside the game simulation.

## Design Sources
- `design/control_system.md`

## Specifications

### ControlState (client-side, per player)
- Selection: current selected objects
- ObjectInterfaceState[ActiveGroup]: command flow state for active selection
- Validated against game state each tick (dead objects removed, invalid state resets)

### Selection
- Array of SelectionGroups, each with Type (ObjectEnum) and Instances (array of ObjectInstance refs)
- ActiveGroup: which group's commands are displayed
- **Constraints**:
  - Enemy/unowned: selection must contain exactly one object in one group
  - Own objects: no limit on count or mixing of types
  - Ungroupable objects (Groupable=false): always occupy their own SelectionGroup

### BoxSelection
When the player drags a selection box, objects are filtered by a 5-tier priority system. The highest-priority tier containing any objects wins; all lower tiers excluded.

| Priority | Tier | Selection Mode |
|----------|------|----------------|
| 1 (highest) | Own units | Multi-select (all in box) |
| 2 | Own buildings | Single-select (closest to box center) |
| 3 | Enemy units | Single-select (closest to box center) |
| 4 | Enemy buildings | Single-select (closest to box center) |
| 5 (lowest) | Neutral objects | Single-select (closest to box center) |

Only own units (tier 1) produce multi-selection. All other tiers select exactly one object closest to box center.

### ControlGroups
- 10 saved selections (indexed 0-9) per player
- Operations: Assign, Add, Recall, Recall and Center (centers camera on centroid)
- Entities can belong to multiple groups simultaneously
- Dead entities silently removed on recall

### GroupCycling
- StateOnlyTransition: changes ActiveGroup to an adjacent SelectionGroup
- **Forward (Tab)**: advance to next SelectionGroup, wraps around after last
- **Backward (Shift-Tab)**: advance to previous SelectionGroup, wraps around before first

### SelectionPanel
- Grid of unit portraits for the current selection
- **Visible** when Selection contains 2+ units
- **Hidden** when Selection contains 0 or 1 units (single-unit display handled by InfoPanel)
- Portraits belonging to the ActiveGroup shown with a sheer highlight

**Portrait Interactions**:

| Input | Effect |
|-------|--------|
| Left-click | Replace selection with only that unit |
| Shift-click | Remove that unit from selection |
| Ctrl-click | Replace selection with all units of that type in the current selection |
| Ctrl-Shift-click | Remove all units of that type from selection |
| Alt-click | Center camera on that unit (no selection change) |

**Edge Cases**:
- Reducing selection to 1 unit hides SelectionPanel; InfoPanel displays that unit
- Removing all remaining units empties the selection

### CommandPanel
- Displays available commands for current selection, derived from ControlState + game state each tick
- CommonCommands: available to every object in Selection (visually distinguished)
- GroupCommands: available to objects of type ActiveGroup
- Common commands go to all selected objects; group-specific to ActiveGroup only

### InterfaceTransition Types
- **StateOnlyTransition**: Modifies only ControlState (e.g., entering target selection, opening menu)
- **CommandIssuingTransition**: Modifies ControlState AND issues a Command to game objects

### DefaultState
- Command panel shows available commands
- Right-click: context-sensitive command (RightClickResolution)
- Hotkeys/buttons: transition to AwaitingTarget or issue immediate commands

### AwaitingTarget[CommandType]
- Entered after selecting a command that requires a target
- Left-click valid target: CommandIssuingTransition, returns to DefaultState
- Escape/right-click: StateOnlyTransition back to DefaultState

### CursorTarget
- Ground | EnemyObject | FriendlyObject | NeutralObject
- With Location (Coordinates) and Object (ObjectInstance | None)

### BasicCombatUnitInterfaceState
Template for combat units:
- **Immediate commands**: HoldPosition, Stop
- **Target commands**: Attack, Move, Patrol, AttackGround (if CanTargetGround), Reverse (if CanReverse)
- **RightClick**: Enemy -> Attack; Ground -> Move; Own Tunnel (Syndicate units, tier sufficient) -> Enter; Friendly/Neutral -> Move
- **AwaitingTarget[Attack]**: Enemy -> Attack; Ground -> AttackMove
- **AwaitingTarget[Move]**: Ground -> Move; Object -> Move to object
- **AwaitingTarget[Patrol]**: Ground -> Patrol
- **AwaitingTarget[AttackGround]**: Ground -> AttackGround
- **AwaitingTarget[Reverse]**: Ground -> Reverse

### CommandIndicators
Visual markers displayed at command targets for selected units. Indicators are only visible when a unit with that active command is part of the current Selection. All active command indicators shown simultaneously. Removed on deselect or command completion.

**Indicator Types**:
- **Location Indicator**: Marker on the ground at target coordinates
- **Object Indicator**: Marker surrounding target object's perimeter

**Indicator Assignments**:

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

**Color Language**: Green = peaceful movement, Red = hostile target, Orange = aggressive movement

## Dependencies
- `entity_system` (Object Types, Instances, Groupable property)
- `unit_commands_and_behaviors` (Commands issued by transitions)
