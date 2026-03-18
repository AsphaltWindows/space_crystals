# Ticket: SelectionPanel

## Current State
No selection panel exists. When multiple units are selected, there is no UI element showing the individual units in the selection or allowing per-unit interactions.

## Desired State
Implement the SelectionPanel as a grid of unit portraits for the current selection:

**Visibility**:
- **Shown** when Selection contains 2+ units
- **Hidden** when Selection contains 0 or 1 units (single-unit display is handled by the InfoPanel)

**Portrait Display**:
- Each selected unit has a portrait in the grid
- Portraits belonging to the ActiveGroup are shown with a sheer highlight to visually distinguish them from other groups

**Portrait Interactions**:

| Input | Effect |
|-------|--------|
| Left-click | Replace selection with only that unit |
| Shift-click | Remove that unit from selection |
| Ctrl-click | Replace selection with all units of that type in the current selection |
| Ctrl-Shift-click | Remove all units of that type from selection |
| Alt-click | Center camera on that unit (no selection change) |

**Edge Cases**:
- Reducing selection to 1 unit hides SelectionPanel; the remaining unit is displayed via InfoPanel
- Removing all remaining units via shift-click/ctrl-shift-click empties the selection entirely

## Justification
Defined in `features/control_system.md` (SelectionPanel section, lines 48-66). The SelectionPanel is a standard RTS UI element that gives the player granular control over their selection. Without it, players cannot inspect individual units in a group, remove specific units from a selection, or quickly filter by type.

## QA Steps
1. Select 2+ owned units. Verify the SelectionPanel appears as a grid of portraits.
2. Verify portraits of units in the ActiveGroup have a sheer highlight. Verify portraits of units in other groups do not.
3. Press Tab to change ActiveGroup. Verify the sheer highlight moves to the new ActiveGroup's portraits.
4. Select exactly 1 unit. Verify the SelectionPanel is hidden.
5. Select 0 units (click empty ground). Verify the SelectionPanel is hidden.
6. With 3+ units selected, left-click a portrait. Verify the selection is replaced with only that unit and the SelectionPanel hides (now 1 unit).
7. With 3+ units selected, shift-click a portrait. Verify that unit is removed from the selection. Verify the SelectionPanel updates (one fewer portrait).
8. With 3+ units selected (including 2+ of the same type), ctrl-click a portrait. Verify the selection is replaced with all units of that type from the previous selection.
9. With a mixed-type selection (e.g., 3 Peacekeepers + 2 other units), ctrl-shift-click a Peacekeeper portrait. Verify all Peacekeepers are removed from the selection. Verify remaining units stay selected.
10. With 2+ units selected, alt-click a portrait. Verify the camera centers on that unit. Verify the selection does not change.
11. With exactly 2 units selected, shift-click one portrait. Verify the selection reduces to 1 unit and the SelectionPanel hides.
12. With 2 units of the same type selected, ctrl-shift-click a portrait. Verify all units of that type are removed, emptying the selection entirely.

## Expected Experience
The SelectionPanel should appear instantly when 2+ units are selected and disappear instantly when the selection drops to 0 or 1. The ActiveGroup sheer highlight should update immediately on group cycling. Portrait click interactions should feel responsive with no delay. Left-click and ctrl-click should replace the selection cleanly. Shift-click and ctrl-shift-click should smoothly remove portraits from the grid without visual glitches. Alt-click should smoothly pan the camera to the clicked unit.
