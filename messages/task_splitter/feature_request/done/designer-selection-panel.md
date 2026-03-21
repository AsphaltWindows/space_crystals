# selection-panel

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Implement the SelectionPanel as defined in `artifacts/designer/design/control_system.md` under 'SelectionPanel'.

The SelectionPanel displays a grid of unit portraits for the current selection.

**Visibility:**
- Visible when Selection contains 2+ units
- Hidden when Selection contains 0 or 1 units (single-unit display is handled by the InfoPanel)

**Display:**
- Each cell shows the portrait of one selected unit instance
- Portraits belonging to the ActiveGroup are shown with a sheer highlight

**Interactions:**

| Input | Effect |
|-------|--------|
| Left-click portrait | Replace selection with only that unit |
| Shift-click portrait | Remove that unit from selection |
| Ctrl-click portrait | Replace selection with all units of that type in the current selection |
| Ctrl-Shift-click portrait | Remove all units of that type from selection |
| Alt-click portrait | Center camera on that unit (no selection change) |

**Edge cases:**
- If an action reduces the selection to 1 unit, the SelectionPanel hides and the InfoPanel displays that unit
- If an action removes all remaining units, the selection becomes empty

## QA Instructions

1. Select a single unit — verify SelectionPanel is NOT visible.
2. Select 2+ units (box select or shift-click) — verify SelectionPanel appears with individual portraits.
3. Verify portraits belonging to the ActiveGroup have a highlight effect.
4. Tab to change ActiveGroup — verify the highlight updates to the new group's portraits.
5. Left-click a portrait — verify selection is replaced with only that unit and SelectionPanel hides.
6. Re-select multiple units. Shift-click a portrait — verify that unit is removed from selection.
7. If removing leaves 1 unit, verify SelectionPanel hides and InfoPanel shows.
8. Select multiple units of mixed types. Ctrl-click a portrait — verify selection becomes all units of that type.
9. Ctrl-Shift-click a portrait — verify all units of that type are removed from selection.
10. Alt-click a portrait — verify camera centers on that unit without changing selection.
