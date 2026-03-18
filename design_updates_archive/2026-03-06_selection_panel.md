# Design Update — SelectionPanel & GroupCycling

**Date**: 2026-03-06
**Files modified**: `design/control_system.md`

## Changes

### New: SelectionPanel (control_system.md)

Added `SelectionPanel` section defining the unit portrait grid displayed when 2+ units are selected. Located between InfoPanel and CommandPanel in the HUD layout.

- Grid of unit portraits, one per selected unit instance
- ActiveGroup portraits shown with a sheer highlight
- Hidden when 0 or 1 units selected (single-unit display handled by InfoPanel)

Portrait click interactions:

| Input | Effect |
|-------|--------|
| Left-click | Replace selection with only that unit |
| Shift-click | Remove that unit from selection |
| Ctrl-click | Replace selection with all units of that type in the current selection |
| Ctrl-Shift-click | Remove all units of that type from selection |
| Alt-click | Center camera on that unit (no selection change) |

Edge cases: reducing to 1 unit hides the panel; removing all units empties the selection.

### Updated: GroupCycling (control_system.md)

Added directional cycling with keybindings:
- Tab: advance ActiveGroup forward (wraps around)
- Shift-Tab: advance ActiveGroup backward (wraps around)
