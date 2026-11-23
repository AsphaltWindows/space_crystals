# expand_command_panel_grid

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

The CommandPanel grid layout has been expanded from 3x3 (9 slots) to 4x3 (12 slots) in `artifacts/designer/design/control_system.md`.

## Changes

### CommandPanel Grid Layout (control_system.md)
- Grid changed from 3x3 to 4x3 (4 columns, 3 rows)
- New hotkey column added: R (top-right), F (middle-right), V (bottom-right)
- Full hotkey grid is now: Q-W-E-R / A-S-D-F / Z-X-C-V

### BasicCombatUnitInterfaceState (control_system.md)
- Grid layout updated to show the new 4x3 format
- New slots R, F, V are currently unassigned (shown as —) for basic combat units
- All existing command assignments unchanged

### What did NOT change
- Standard slot assignments (Z=Back/Cancel, X=Cancel Production, C=Set Rally Point) remain the same
- No new standard assignments for R, F, V — these are available for per-object-type use
- All existing command hotkeys and behaviors unchanged

## QA Instructions

1. Open the command panel UI for any selected unit or structure
2. Verify the panel displays a 4-column by 3-row grid of command slots
3. Verify hotkeys Q, W, E, R map to the top row (left to right)
4. Verify hotkeys A, S, D, F map to the middle row (left to right)
5. Verify hotkeys Z, X, C, V map to the bottom row (left to right)
6. Select a basic combat unit (e.g., Peacekeeper) — verify existing commands (Move, Attack, Patrol, etc.) appear in their original positions and the new R, F, V slots are empty
7. Verify all existing hotkeys (Q, W, E, A, S, D, Z, X, C) still trigger their assigned commands
8. Verify pressing R, F, or V does nothing when those slots are unassigned
9. Select a production building — verify Z (Back), X (Cancel), C (Rally Point) still work in their standard positions
