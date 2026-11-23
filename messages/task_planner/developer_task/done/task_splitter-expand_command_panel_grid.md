# expand_command_panel_grid

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-expand_command_panel_grid.md

## Task

Expand the CommandPanel grid layout from 3x3 (9 slots) to 4x3 (12 slots, 4 columns x 3 rows). The new hotkey layout is Q-W-E-R / A-S-D-F / Z-X-C-V.

### Files to modify

1. **artifacts/developer/src/ui/command_panel/grid_layout.rs**:
   - Change `GRID_HOTKEYS` from `[[char; 3]; 3]` to `[[char; 4]; 3]` — add 'R' to row 0, 'F' to row 1, 'V' to row 2
   - Add `KeyCode::KeyR => Some((0, 3))`, `KeyCode::KeyF => Some((1, 3))`, `KeyCode::KeyV => Some((2, 3))` to `keycode_to_grid_slot`
   - In `get_grid_slot_action`: no changes needed — existing match arms use (row, col) tuples and the new slots (0,3), (1,3), (2,3) will fall through to `_ => None` (unassigned)

2. **artifacts/developer/src/ui/command_panel/ui_rebuild.rs** (~line 337):
   - Change `grid_template_columns: RepeatedGridTrack::flex(3, 1.0)` to `RepeatedGridTrack::flex(4, 1.0)`
   - Change the inner loop from `for col in 0..3u8` to `for col in 0..4u8` (~line 395)

3. **artifacts/developer/src/ui/command_panel/hotkeys.rs** (~line 217):
   - Add `KeyCode::KeyR`, `KeyCode::KeyF`, `KeyCode::KeyV` to the `grid_keys` array
   - NOTE: R and F already have non-grid hotkey uses earlier in the function (R/Shift+R for rotation in placement mode at ~line 190, F for ready-to-place shortcut at ~line 167 and F/Shift+F for flip in placement mode at ~line 202). These are gated by early returns, so the grid_keys section will only be reached when those conditions don't apply. Verify this is correct.

### What must NOT change
- All existing command assignments for all ObjectInterfaceStates remain unchanged
- Standard slot assignments (Z=Back, X=Cancel, C=Rally) stay in the same positions
- The new R, F, V slots are unassigned (return None from get_grid_slot_action) for all current states
- The TunnelExpandMenu and TunnelEjectMenu dynamic grids (build_tunnel_expand_grid / build_tunnel_eject_grid in ui_rebuild.rs) also need their grid template updated to 4 columns if they share the same grid layout

### Testing
- Existing tests should continue to pass with no changes (all test (row,col) values are within the 3x3 range)
- Verify pressing R, F, V does nothing when slots are unassigned (falls through to None)
- Verify all existing hotkeys still work
