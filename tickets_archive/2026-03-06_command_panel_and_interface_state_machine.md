# Ticket: Command Panel and Interface State Machine

## Current State
No interface transition system, command panel, or object interface state machine exists. There is no mechanism for translating player input into command flows or displaying available commands.

## Desired State
Implement the command panel, interface transition types, and the object interface state machine:

**CommandPanel**:
- Derived from ControlState + game state each tick
- CommonCommands: commands available to every object in the entire Selection (visually distinguished)
- GroupCommands: commands available to objects of the ActiveGroup type only
- Clicking a common command issues it to all selected objects
- Clicking a group-specific command issues it only to ActiveGroup objects

**InterfaceTransition** (Value: InterfaceTransitionEnum):
- **StateOnlyTransition**: Modifies only ControlState; no Command issued (e.g., entering target selection, opening build menu, cycling active group)
- **CommandIssuingTransition**: Modifies ControlState AND issues a Command to selected game objects (e.g., confirming a target, placing a building)

**GroupCycling**:
- A StateOnlyTransition that changes ActiveGroup to an adjacent SelectionGroup in the Selection
- **Forward (Tab)**: advance to next SelectionGroup, wraps around after last
- **Backward (Shift-Tab)**: advance to previous SelectionGroup, wraps around before first

**ObjectInterfaceState[ObjectEnum]**:
- Parameterized by object type; each type defines its own concrete states and transitions
- Resets to default state when Selection or ActiveGroup changes
- Validated each tick against active SelectionGroup's game state; resets if current state is no longer valid
- For Ungroupable objects, SelectionGroup always has exactly one instance, so state can read that instance's specific game state

**DefaultState** (the default ObjectInterfaceState):
- Command panel shows available commands
- Right-click: context-sensitive command via RightClickResolution (evaluated against ActiveGroup's capabilities)
- Hotkeys/buttons: transition to AwaitingTarget or issue immediate commands

**AwaitingTarget[CommandType]**:
- Entered after selecting a command that requires a target
- Command panel indicates which command is pending
- Left-click valid target: CommandIssuingTransition, returns to DefaultState
- Target may resolve to a different command than the pending one based on CursorTarget
- Escape or right-click: StateOnlyTransition back to DefaultState

**CursorTarget**:
- Value: CursorTargetEnum (Ground | EnemyObject | FriendlyObject | NeutralObject)
- Location: Coordinates
- Object: ObjectInstance | None

## Justification
Defined in `features/control_system.md` (CommandPanel, InterfaceTransition, ObjectInterfaceState, DefaultState, AwaitingTarget, CursorTarget sections). This is the core state machine that translates player input into game commands. Without it, no player interaction beyond selection is possible.

## QA Steps
1. Select owned combat units. Verify the command panel displays commands relevant to those units.
2. Select units of two different types. Verify CommonCommands (shared by all) are visually distinguished from GroupCommands (ActiveGroup only).
3. Click a common command. Verify it is issued to all selected objects, not just the ActiveGroup.
4. Click a group-specific command. Verify it is issued only to ActiveGroup objects.
5. Press Tab. Verify ActiveGroup advances to the next SelectionGroup. Press Tab repeatedly to verify forward wrap-around (last -> first).
6. Press Shift-Tab. Verify ActiveGroup moves to the previous SelectionGroup. Press Shift-Tab repeatedly to verify backward wrap-around (first -> last).
7. Press a target command hotkey (e.g., Attack). Verify the interface enters AwaitingTarget[Attack] state and the command panel indicates the pending command.
8. While in AwaitingTarget, left-click a valid target. Verify the command is issued (CommandIssuingTransition) and the interface returns to DefaultState.
9. While in AwaitingTarget, press Escape. Verify the interface returns to DefaultState without issuing a command.
10. While in AwaitingTarget, right-click. Verify the interface returns to DefaultState without issuing a command.
11. Hover cursor over an enemy object. Verify CursorTarget resolves to EnemyObject with correct ObjectInstance and Location.
12. Hover cursor over empty ground. Verify CursorTarget resolves to Ground with Location and Object=None.
13. Hover cursor over a friendly object. Verify CursorTarget resolves to FriendlyObject.
14. Change the Selection. Verify ObjectInterfaceState resets to DefaultState.
15. Change the ActiveGroup (via Tab). Verify ObjectInterfaceState resets to DefaultState.
16. While a unit is selected, destroy that unit externally. Verify the ObjectInterfaceState resets if the state becomes invalid due to the removal.

## Expected Experience
The command panel should update every tick to reflect available commands. Transitioning between DefaultState and AwaitingTarget should feel instant. Group cycling should visually update the command panel to show the new ActiveGroup's commands. CursorTarget resolution should be seamless -- the player never sees the resolution logic, only the resulting command. Canceling with Escape or right-click should feel snappy with no delay.
