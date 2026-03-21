# control-selection-keybinding-fixes

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-control-state-selection.md

## Task

Fix two keybinding issues in the control/selection system:

### 1. Control Group Add: Change from Shift+Num to Ctrl+Shift+Num

In `game/world/resources.rs`, `control_group_system` (~line 645):
- Currently Ctrl+Num = Assign and Shift+Num = Add
- The design spec requires Ctrl+Num = Assign and Ctrl+Shift+Num = Add
- Fix: Check `ctrl_held && shift_held` BEFORE checking `ctrl_held` alone (order matters since Ctrl+Shift has both flags set)
- The branch order should be: (1) ctrl+shift = Add, (2) ctrl-only = Assign, (3) plain number = Recall

### 2. Shift-Tab Backward Group Cycling

Add backward cycling (Shift-Tab) to complement the existing forward Tab cycling:

**In `shared/types.rs`**: Add `cycle_active_group_backward()` method to `Selection`:
- If active_group_index is Some(idx) and groups is non-empty: set to `(idx + groups.len() - 1) % groups.len()`
- Add unit tests for backward cycling (wrapping, single group, empty)

**In `game/world/resources.rs`**, `active_group_cycle_system` (~line 811):
- Add check for Shift+Tab (keyboard shift pressed AND Tab just pressed) → call `cycle_active_group_backward()`
- Existing Tab-only check should verify shift is NOT pressed to avoid double-cycling

**In `ui/command_panel.rs`** (if Tab cycling is also handled there for commandable groups):
- Add matching Shift-Tab backward cycling logic alongside existing Tab forward cycling
