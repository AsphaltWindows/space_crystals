# ef-flat-interface-rework

## Metadata
- **From**: task_splitter
- **To**: task_planner

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

### Changes needed in command_panel.rs:

1. **EfIdle grid (get_grid_slot_action)**: Make Q context-dependent. Currently Q always maps to EfBuildPlate. Need a new parameter or check: when EF has ready_to_place=true, Q at (0,0) should map to EnterPlacement instead of EfBuildPlate. The has_active_construction flag already handles X=EfCancel visibility correctly.

2. **Remove EfConstructing grid**: The `StructureMenuState::EfConstructing` match arm (Z=Back, X=Cancel) should be removed — constructing state is now handled within EfIdle.

3. **Remove EfReadyToPlace grid**: The `StructureMenuState::EfReadyToPlace` match arm (Q=EnterPlacement, Z=Back, X=Cancel) should be removed — ready state is now handled within EfIdle.

4. **EfBuildPlate action (execute_command_action)**: Currently transitions to EfConstructing on success (line ~1314). Change to stay in EfIdle. Also remove the routing logic at lines ~1300-1306 that redirects to EfConstructing/EfReadyToPlace when construction is active.

5. **Escape handler**: Change EfAwaitingPlacement escape target from EfReadyToPlace to EfIdle (line ~892).

6. **Right-click cancel (right_click_cancel_target)**: Remove entries for EfConstructing and EfReadyToPlace (lines ~1344-1346) since those states won't exist.

7. **Back handler**: Remove the Back entries for EfConstructing/EfReadyToPlace.

8. **Progress sync (update_command_panel_progress)**: Remove the EfConstructing and EfReadyToPlace branches (lines ~1570-1587). Instead, add an EfIdle branch that calls set_changed() when EF state changes (to refresh the dynamic Q button when construction completes).

9. **State validation (update_command_panel_state)**: In the ExtractionFacility branch (~line 338-351), remove EfConstructing/EfReadyToPlace from valid states. Only EfIdle and EfAwaitingPlacement should be valid.

10. **EfIdle info display**: Ensure the construction progress indicator is shown when in EfIdle and EF is constructing (check the info/progress text section around line 516+). The progress bar should be visible directly in DefaultState.

11. **Optionally remove StructureMenuState::EfConstructing and EfReadyToPlace** enum variants from types.rs if they are no longer used anywhere. If other files reference them, update accordingly.

12. **Tests**: Update all tests referencing EfConstructing/EfReadyToPlace. Add tests verifying:
    - EfIdle with ready_to_place shows EnterPlacement at Q
    - EfIdle with constructing shows progress and X cancel
    - EfIdle with idle shows EfBuildPlate at Q
    - Escape from EfAwaitingPlacement returns to EfIdle
