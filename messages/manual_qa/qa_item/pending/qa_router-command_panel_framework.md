# command-panel-framework

## Metadata
- **From**: qa_router
- **To**: manual_qa

## Content

## Content

## Content

# command-panel-framework

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Implement the CommandPanel framework and standard slot assignments as defined in `artifacts/designer/design/control_system.md` under CommandPanel.

**CommandPanel:**
Displays available commands for the current selection, derived from ControlState and game state each tick.
- **CommonCommands**: commands available to EVERY object in the Selection. Visually distinguished from group commands. Clicking issues to ALL selected objects.
- **GroupCommands**: commands available only to the ActiveGroup. Clicking issues only to ActiveGroup objects.

**Grid Layout — 3x3 grid with default hotkeys:**
```
| Q | W | E |
| A | S | D |
| Z | X | C |
```

**Standard Slot Assignments** (consistent across all object types that use them):
- **Z (bottom-left)**: Back / Cancel menu. In any multi-stage state (BuildMenu, ExpandMenu, EjectMenu, AwaitingTarget, AwaitingPlacement), Z returns to the previous state (StateOnlyTransition). Equivalent to Escape or right-click cancel.
- **X (bottom-center)**: Cancel Production / Cancel Upgrade. In production buildings, cancels last queued item and refunds cost. In structures with upgrades, cancels in-progress upgrade and refunds cost.
- **C (bottom-right)**: Set Rally Point. In unit-producing structures, enters AwaitingTarget[SetRallyPoint]. Left-click ground or object sets rally point (CommandIssuingTransition, returns to DefaultState).

**Production Building Default Right-Click:**
For ALL unit-producing structures, right-click from DefaultState sets the rally point:
- Right-click Ground: sets rally point to that location
- Right-click Object: sets rally point to that object

## QA Instructions

1. Select any unit — verify the command panel shows a 3x3 grid layout.
2. Verify hotkeys match grid positions: Q=top-left, W=top-center, E=top-right, A=mid-left, etc.
3. Select a production building (e.g., Barracks) — verify C shows Set Rally Point and X shows Cancel Production.
4. Enter a sub-menu (e.g., BuildMenu) — verify Z shows Back/Cancel and returns to previous state.
5. Press Escape from a sub-menu — verify same behavior as Z.
6. Right-click from a sub-menu — verify same behavior as Z.
7. Right-click ground while a production building is selected in DefaultState — verify rally point is set.
8. Select mixed unit types — verify CommonCommands (shared by all) are visually distinguished from GroupCommands (active group only).
9. Click a CommonCommand — verify it issues to ALL selected objects.
10. Click a GroupCommand — verify it issues only to the ActiveGroup.
