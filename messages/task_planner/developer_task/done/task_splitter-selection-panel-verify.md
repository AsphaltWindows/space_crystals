# selection-panel-verify

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-selection-panel.md

## Task

**Verification task — the SelectionPanel is already fully implemented.**

Verify that the existing SelectionPanel implementation in `ui/hud.rs` matches the design spec. The following should already be working:

1. **Display system** (`update_selected_units_grid_system`): Renders a grid of unit portraits in the `UnitsGridSection` when 2+ entities are selected. Single-entity selection shows the InfoPanel instead. Portraits for entities in the ActiveGroup should have a highlight (sheer tint).

2. **Click interaction system** (`selection_portrait_click_system`): Handles all 5 interaction modes via `SelectionPortrait` component:
   - Left-click: Replace selection with only that unit
   - Shift-click: Remove that unit from selection
   - Ctrl-click: Replace selection with all units of same type in current selection
   - Ctrl-Shift-click: Remove all units of same type from selection
   - Alt-click: Center camera on that unit (no selection change)

3. **Edge cases**: Selection reducing to 1 unit hides panel and shows InfoPanel. Selection reducing to 0 becomes empty.

**What to verify:**
- Run `cargo test` — all existing tests pass
- Read `update_selected_units_grid_system` and confirm it shows portraits when 2+ selected, hides when 0-1
- Read `selection_portrait_click_system` and confirm all 5 click modes are implemented correctly
- Confirm `SelectionPortrait` component stores the referenced entity
- Confirm active group highlighting logic exists (portraits in ActiveGroup get visual distinction)
- If any gap is found, implement the fix
