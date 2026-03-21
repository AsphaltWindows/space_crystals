# ef-flat-interface-rework

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-dc-ef-construction-rework.md

## Task

Rework the ExtractionFacility interface from a submenu pattern (EfIdle/EfConstructing/EfReadyToPlace with Z=Back) to a flat DefaultState interface where all states are handled within EfIdle with dynamic button visibility.

### Design spec (from gdo_objects.md):
- DefaultState (idle): Q=Build Extraction Plate (starts construction, deducts 75 SC)
- DefaultState (constructing): X=Cancel Construction (full refund). Progress indicator visible.
- DefaultState (ready to place): Q=Place Plate (enters AwaitingPlacement). X=Cancel Ready Plate (75% refund).
- AwaitingPlacement: ghost over valid SpaceCrystalsPatch within build area. Escape/right-click returns to DefaultState (EfIdle).

## Technical Context

### Files to modify

**1. `artifacts/developer/src/ui/types.rs` (lines 191-226)**
- Remove `EfConstructing` (line 207-208) and `EfReadyToPlace` (line 209-210) from `StructureMenuState` enum.
- Keep `EfIdle` and `EfAwaitingPlacement`.

**2. `artifacts/developer/src/ui/command_panel.rs`** — This is the main file with ~15 change sites:

**(a) `get_grid_slot_action()` (lines 41-49, 81-96):**
- The function already receives `has_active_construction: bool` param (line 46). Currently this controls the conditional X=EfCancel in EfIdle (line 83). It combines both `ef.current_construction || ef.ready_to_place` (computed at lines 760-763 and 1001-1004).
- Add a new parameter `has_ready_plate: bool` to distinguish ready_to_place from constructing. Alternatively, repurpose existing logic.
- **EfIdle grid (lines 81-85):** Change `(0,0)` from always-EfBuildPlate to: if `has_ready_plate` → `EnterPlacement`, else → `EfBuildPlate`. The `(2,1)` EfCancel conditional already works correctly via `has_active_construction`.
- **Remove EfConstructing grid (lines 86-90):** Delete entire match arm.
- **Remove EfReadyToPlace grid (lines 91-96):** Delete entire match arm.
- Update all call sites (lines 779 and 1012) to pass the new `has_ready_plate` param. Compute it as: `panel_target.entity.and_then(|e| ef_query.get(e).ok()).map(|ef| ef.ready_to_place).unwrap_or(false)`.

**(b) `execute_command_action()` — EfBuildPlate handler (lines 1298-1322):**
- Lines 1300-1308: Remove the routing logic that redirects to EfConstructing/EfReadyToPlace when construction is already active (these states won't exist).
- Line 1316: Change `EfConstructing` transition to stay in `EfIdle` (or simply remove the state transition — the panel will auto-refresh via set_changed).
- After starting construction, just call `interface_state.set_changed()` to refresh the grid (showing the new X=Cancel button).

**(c) `execute_command_action()` — EnterPlacement handler (lines 1360-1370):**
- Line 1365: Change the match from `EfReadyToPlace` to `EfIdle` — when Q=EnterPlacement is clicked in EfIdle (ready_to_place=true), transition to `EfAwaitingPlacement`.

**(d) `execute_command_action()` — Back handler (lines 1339-1358):**
- Lines 1346-1348: Remove the EfConstructing/EfReadyToPlace → EfIdle match arms (those states no longer exist).

**(e) Escape handler in `command_panel_hotkeys` (line 892-893):**
- Change `EfAwaitingPlacement` escape target from `EfReadyToPlace` to `EfIdle`.
- Lines 903-905: Remove EfConstructing/EfReadyToPlace escape handlers (states don't exist).

**(f) KeyF shortcut (lines 936-938):**
- Change the `EfReadyToPlace` match to `EfIdle` — pressing F in EfIdle when plate is ready should enter AwaitingPlacement. Note: this requires checking EF state. May need to guard with an ef_query check, or simply let the EnterPlacement action handle it (which already has the state transition).

**(g) `right_click_cancel_target()` (lines 1025+, 1041-1044):**
- Remove the EfConstructing/EfReadyToPlace match arms.

**(h) Right-click cancel in placement system (`game/world/faction.rs` line 1335-1336):**
- Change `EfAwaitingPlacement` right-click target from `EfReadyToPlace` to `EfIdle`.

**(i) `update_command_panel_progress` / progress sync (lines 1603-1619):**
- Remove the `EfConstructing` branch (lines 1603-1612) and `EfReadyToPlace` branch (lines 1614-1619).
- The EfIdle branch already handles progress display (lines 528-540) — it already shows both 'Building Plate...' and 'Plate ready!' text. Just need to add `interface_state.set_changed()` in the EfIdle progress sync to refresh the grid when construction completes (so Q switches from EfBuildPlate to EnterPlacement).

**(j) `update_command_panel_state` / state validation (lines 337-349):**
- Remove `EfConstructing` and `EfReadyToPlace` from the valid states list (lines 342-343). Only `EfIdle` and `EfAwaitingPlacement` should be valid.

**(k) Info display (lines 576-588):**
- Remove the `EfConstructing` info display branch (lines 576-585) and `EfReadyToPlace` info display branch (lines 586-588). The EfIdle branch (lines 528-540) already covers both cases.

**(l) Title display (lines 488-489):**
- Remove `EfConstructing` and `EfReadyToPlace` from the title match.

**3. `artifacts/developer/src/game/world/resources.rs` (line 974):**
- Remove `EfConstructing | EfReadyToPlace` from the match arm. Keep `EfIdle | EfAwaitingPlacement`.

### Patterns to follow
- **DcIdle flat pattern (partially done):** DcIdle uses `has_active_construction` for conditional X=Cancel. The EF rework extends this pattern by also making Q context-dependent based on `ready_to_place` state.
- **set_changed() for refresh:** When EF state changes but interface state stays EfIdle, call `interface_state.set_changed()` to force the command panel to rebuild (showing updated buttons/progress).

### Tests to update (lines 4096+)
- Tests referencing `EfConstructing` and `EfReadyToPlace` must be updated or removed:
  - Line 4142-4149: `EfConstructing` grid test → remove
  - Line 4155-4164: `EfReadyToPlace` grid test → remove  
  - Lines 4315-4397: Multiple grid slot tests using these states → update
  - Lines 4515-4569: Valid state tests → update  
  - Lines 4592-4596: Back/escape tests → update
  - Lines 4658-4678: `has_active_construction` conditional tests → keep/extend
  - Lines 4882-4891: `right_click_cancel_target` tests → remove EfConstructing/EfReadyToPlace entries
- **New tests needed:**
  - EfIdle with `has_ready_plate=true` → Q shows `EnterPlacement`
  - EfIdle with `has_ready_plate=false`, `has_active_construction=true` → Q shows `EfBuildPlate`, X shows `EfCancel`
  - EfIdle with both false → Q shows `EfBuildPlate`, no X
  - Escape from EfAwaitingPlacement → EfIdle

## Dependencies

- **task_splitter-dc-buildmenu-remove-ef (DONE):** That task removed the ExtractionFacility option from DC's build menu. Must be completed first since it may have already modified some shared code paths. It is already done.
