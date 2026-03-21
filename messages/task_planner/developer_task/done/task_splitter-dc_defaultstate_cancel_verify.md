# dc_defaultstate_cancel_verify

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-dc-defaultstate-cancel.md

## Task

Verify and polish the DeploymentCenter DefaultState cancel commands to match the updated design doc in `artifacts/designer/design/gdo_objects.md`.

**What the design doc specifies (ObjectInterfaceState[DeploymentCenter] DefaultState):**
- **X: Cancel Construction** — visible when CurrentConstruction is set. Full refund, clears CurrentConstruction.
- **X: Cancel Ready Building** — visible when ReadyToPlace is set. 75% refund (rounded down), clears ReadyToPlace.
- **Q: Build** — enters BuildMenu (StateOnlyTransition).

**Current codebase state (likely already implemented):**
The code in `command_panel.rs` already has:
- `DcIdle` grid slot `(0,0)` → `DcOpenBuildMenu` (Q slot) ✓
- `DcIdle` grid slot `(2,1)` → `DcCancel` guarded by `has_active_construction` (X slot) ✓
- `has_active_construction` checks both `current_construction.is_some()` and `ready_to_place.is_some()` ✓
- `DcCancel` handler differentiates full refund (construction) vs 75% refund (ready-to-place) ✓
- `cancellation_refund()` in `structures.rs` computes `(cost * 3) / 4` for ready-to-place ✓
- Tests exist for both refund cases ✓

**What to do:**
1. Verify the implementation matches the design doc exactly — confirm grid positions, hotkeys, visibility conditions, and refund amounts are correct.
2. Consider updating the cancel button label from the generic `"[X] Cancel"` to context-sensitive labels like `"[X] Cancel\nConstruction"` and `"[X] Cancel\nBuilding"` to match the design doc's distinction between Cancel Construction and Cancel Ready Building. This would require making the label function aware of the DC state.
3. Ensure no regressions — the BuildMenu cancel commands should remain unchanged.
4. If the implementation is fully correct, produce the task_completion. If label changes are needed, implement them with appropriate tests.
