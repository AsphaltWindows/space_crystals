# expand_command_panel_grid

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-expand_command_panel_grid.md

## Task

Expand the CommandPanel grid layout from 3x3 (9 slots) to 4x3 (12 slots, 4 columns x 3 rows). The new hotkey layout is Q-W-E-R / A-S-D-F / Z-X-C-V.

### Files to modify

1. **artifacts/developer/src/ui/command_panel/grid_layout.rs**:
   - Change GRID_HOTKEYS from [[char; 3]; 3] to [[char; 4]; 3] — add 'R' to row 0, 'F' to row 1, 'V' to row 2
   - Add KeyCode::KeyR => Some((0, 3)), KeyCode::KeyF => Some((1, 3)), KeyCode::KeyV => Some((2, 3)) to keycode_to_grid_slot
   - In get_grid_slot_action: no changes needed — existing match arms use (row, col) tuples and the new slots (0,3), (1,3), (2,3) will fall through to _ => None (unassigned)

2. **artifacts/developer/src/ui/command_panel/ui_rebuild.rs** (~line 337):
   - Change grid_template_columns: RepeatedGridTrack::flex(3, 1.0) to RepeatedGridTrack::flex(4, 1.0)
   - Change the inner loop from for col in 0..3u8 to for col in 0..4u8 (~line 395)

3. **artifacts/developer/src/ui/command_panel/hotkeys.rs** (~line 217):
   - Add KeyCode::KeyR, KeyCode::KeyF, KeyCode::KeyV to the grid_keys array
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

## Technical Context

### File 1: grid_layout.rs (artifacts/developer/src/ui/command_panel/grid_layout.rs)
- **Line 7**: `GRID_HOTKEYS: [[char; 3]; 3]` — change to `[[char; 4]; 3]`, adding 'R', 'F', 'V' as the 4th element in each row.
- **Lines 14-27**: `keycode_to_grid_slot()` — add three new arms: `KeyCode::KeyR => Some((0, 3))`, `KeyCode::KeyF => Some((1, 3))`, `KeyCode::KeyV => Some((2, 3))`. The wildcard arm on line 25 already handles unknown keys.
- **Lines 31-185**: `get_grid_slot_action()` — NO changes needed. All existing match arms use specific `(row, col)` tuples. The new column 3 slots will naturally fall through to `_ => None` in every state's match block.

### File 2: ui_rebuild.rs (artifacts/developer/src/ui/command_panel/ui_rebuild.rs)
- **Line 311**: Dynamic tunnel grid container uses `RepeatedGridTrack::flex(3, 1.0)` — change to `flex(4, 1.0)`.
- **Line 337**: Standard grid container uses `RepeatedGridTrack::flex(3, 1.0)` — change to `flex(4, 1.0)`.
- **Line 394-395**: Nested loop `for row in 0..3u8 { for col in 0..3u8 {` — change inner loop to `for col in 0..4u8`.
- **Line 396**: `GRID_HOTKEYS[row as usize][col as usize]` — will automatically work with the expanded array.
- The `spawn_grid_button` and `spawn_empty_grid_cell` helper calls inside the loop need no changes; they already accept arbitrary (row, col) values.

### File 3: hotkeys.rs (artifacts/developer/src/ui/command_panel/hotkeys.rs)
- **Lines 217-221**: The `grid_keys` array — add `KeyCode::KeyR`, `KeyCode::KeyF`, `KeyCode::KeyV`. Suggested layout to match grid order:
  ```rust
  let grid_keys = [
      KeyCode::KeyQ, KeyCode::KeyW, KeyCode::KeyE, KeyCode::KeyR,
      KeyCode::KeyA, KeyCode::KeyS, KeyCode::KeyD, KeyCode::KeyF,
      KeyCode::KeyZ, KeyCode::KeyX, KeyCode::KeyC, KeyCode::KeyV,
  ];
  ```
- **R key safety**: Line 189-199 handles R for rotation in placement mode. Line 188 checks `interface_state.is_placement_mode()` and the block ends with `return;` at line 213. So if placement mode is active, R is consumed for rotation and the grid_keys section is never reached. Safe.
- **F key safety**: Line 167-186 handles F as a shortcut to enter placement mode from DcReadyToPlace / EfIdle states. It only triggers a `return` when one of those specific states matches. Line 200-211 handles F/Shift+F for flip in placement mode, which is inside the `is_placement_mode()` guard that returns at line 213. So F will only reach the grid_keys section when: (a) not in placement mode, AND (b) not in DcReadyToPlace or EfIdle-with-ready-plate states. This is correct — F at grid position (1,3) would be unassigned in those cases anyway.
- **V key**: No existing handler for V key — no conflicts.

### File 4: tunnel_actions.rs (artifacts/developer/src/ui/command_panel/tunnel_actions.rs)
- **build_tunnel_expand_grid() (line 185)**: Uses `slot / 3` and `slot % 3` for row/col math. This must change to `slot / 4` and `slot % 4` to match 4-column layout. The hotkeys vector on line 205 (`['Q','W','E','A','S','D','Z','X','C']`) must expand to include 'R','F','V' in the correct positions: `['Q','W','E','R','A','S','D','F','Z','X','C','V']`. The `slot < 6` break guard on line 209 should change to `slot < 8` (reserve row 2 for Back at slot 8). Empty fill loop on line 230 (`while slot < 6`) should become `while slot < 8`. Back button at line 236 stays at (2, 0). Empty fill for remaining cells on lines 239-240 should fill slots 9, 10, 11 (3 more cells after Back).
- **build_tunnel_eject_grid() (line 246)**: Same pattern. Hotkeys array on line 274 (`['Q','W','E','A','S','D']`) must expand to `['Q','W','E','R','A','S','D','F']` (8 slots before Back row). The `slot / 3` and `slot % 3` math (lines 279-280) must become `slot / 4` and `slot % 4`. The `slot >= 6` break on line 277 becomes `slot >= 8`. The fill-to-slot-6 loop (line 298) becomes fill-to-8. Back button stays at (2,0). Remaining empty cells fill slots 9-11.

### Pattern to follow
All grid buttons are spawned via `spawn_grid_button(grid, label, action, enabled, active, is_common, row, col)` from helpers.rs. Empty cells use `spawn_empty_grid_cell(grid)`. These helpers already support arbitrary row/col values — no changes needed there.

### Bevy ECS considerations
- No new systems, components, or resources are introduced. This is purely a UI layout change.
- The `rebuild_command_panel_ui` system runs on change detection (`interface_state.is_changed()`), so the expanded grid will rebuild correctly.
- The `command_panel_hotkeys` system processes one key per frame (line 266 `break`), which remains correct with more keys in the array.

## Dependencies

None. This is a self-contained UI layout expansion. All existing command assignments remain unchanged — new slots are unassigned and will be populated by future features.
