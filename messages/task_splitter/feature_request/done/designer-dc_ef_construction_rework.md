# dc-ef-construction-rework

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Rework DeploymentCenter and ExtractionFacility construction interfaces as defined in `artifacts/designer/design/gdo_objects.md`.

**NOTE: The DC DefaultState Cancel command (X hotkey) was already sent as a prior feature request (designer-dc-defaultstate-cancel, now in done/) and may already be implemented. This request covers the broader construction flow rework for both DC and EF.**

**DeploymentCenter changes:**
- DefaultState: Q enters BuildMenu, X cancels (construction or ready-to-place). The BuildMenu should NOT auto-open.
- BuildMenu (idle): shows available buildings with costs. Click one to start construction.
- BuildMenu (constructing): shows X (Cancel Construction), other options unavailable, Z returns to DefaultState.
- BuildMenu (ready to place): click completed building to enter AwaitingPlacement. X cancels ready building (75% refund). Z returns to DefaultState.
- AwaitingPlacement: ghost preview with green/red tinting, R/Shift+R rotate, F/Shift+F flip, build area overlay shown. Left-click valid location places building. Escape/right-click returns to BuildMenu.

**ExtractionFacility changes — flat interface (no submenu):**
- DefaultState (idle): Q starts Extraction Plate construction (deducts 75 crystals).
- DefaultState (constructing): X cancels construction (full refund). Real-time progress indicator should be visible.
- DefaultState (ready to place): Q enters AwaitingPlacement. X cancels ready plate (75% refund).
- AwaitingPlacement: ghost over valid SpaceCrystalsPatch within build area. Left-click places. Escape/right-click returns to DefaultState.

**Key difference:** DC uses a BuildMenu submenu because it has multiple building options. EF has only one product (Extraction Plate) so it uses a flat DefaultState interface — no submenu needed.

## QA Instructions

1. Select a DeploymentCenter. Verify DefaultState shows Q (Build) and is NOT auto-opened to BuildMenu.
2. Press Q — verify BuildMenu shows available buildings.
3. Start a construction. Verify DefaultState shows X (Cancel). Press X — verify full refund.
4. Complete a construction. Verify BuildMenu shows the ready building. Click it — verify AwaitingPlacement with ghost preview and build area overlay.
5. R/Shift+R to rotate, F/Shift+F to flip. Verify side labels update.
6. Place the building in a valid location. Verify placement succeeds.
7. Cancel a ready building with X — verify 75% refund (rounded down).
8. Select an ExtractionFacility. Verify DefaultState shows Q (Build Extraction Plate) directly — NO submenu.
9. Press Q — verify construction starts and 75 crystals are deducted. Verify a progress indicator is visible.
10. Press X during construction — verify full refund.
11. Let construction complete. Verify Q now enters AwaitingPlacement. X cancels with 75% refund.
12. In AwaitingPlacement: verify ghost only shows green over valid SpaceCrystalsPatch within build area.
