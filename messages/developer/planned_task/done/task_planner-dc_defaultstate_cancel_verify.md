# dc_defaultstate_cancel_verify

## Metadata
- **From**: task_planner
- **To**: developer

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

## Technical Context

### Files to inspect/modify

1. **`artifacts/developer/src/ui/command_panel.rs`** — Primary file. Contains all relevant logic:
   - **`get_grid_slot_action()`** (line ~41): Maps grid positions to actions. `DcIdle` at `(0,0)` → `DcOpenBuildMenu`, `(2,1)` guarded by `has_active_construction` → `DcCancel`. This matches the design doc's Q and X slots. **Verified correct.**
   - **`DcCancel` handler** (line ~1071): Checks `current_construction` first (full refund), then `ready_to_place` (75% refund via `cancellation_refund()`). Clears the relevant field and resets to `DcIdle`. **Verified correct.**
   - **`has_active_construction` computation** (line ~762 and ~982): Checks `dc.current_construction.is_some() || dc.ready_to_place.is_some()`. This correctly makes the X cancel button visible in both construction and ready-to-place states. **Verified correct.**
   - **`grid_button_label()`** (line ~2076): Currently returns `"[X] Cancel"` for `DcCancel` regardless of DC state (line 2089). The function receives `_state: &ObjectInterfaceState` (note: currently unused, prefixed with underscore). **This is the area to potentially change.**

2. **`artifacts/developer/src/game/types/structures.rs`** — Contains `DeploymentCenterState::cancellation_refund()` (line ~110). Full refund when `current_construction.is_some()`, 75% (`(cost * 3) / 4`) when `ready_to_place.is_some()`. **Verified correct.**

3. **`artifacts/developer/src/ui/types.rs`** — Contains `StructureMenuState::DcIdle` (line ~193) and `CommandButtonAction::DcCancel` (line ~246). No changes needed.

### Context-sensitive label implementation

To implement context-sensitive cancel labels, the developer should:

1. **In `grid_button_label()`** (line ~2089): Change the `DcCancel` match arm to inspect the `state` parameter:
   - When state is `StructureMenu(DcIdle)`: The label should depend on the underlying DC state. However, `grid_button_label` does NOT currently receive DC state info — it only gets the `ObjectInterfaceState`. Since `DcIdle` is the state for both constructing and ready-to-place DCs, the label function cannot distinguish between them without additional context.
   - When state is `StructureMenu(DcConstructing)`: Label should be `"[X] Cancel\nConstr."`
   - When state is `StructureMenu(DcReadyToPlace)`: Label should be `"[X] Cancel\nBuilding"`
   - The `DcIdle` case is the tricky one. Options:
     - **Option A**: Pass a new parameter (e.g., `dc_is_ready_to_place: bool`) to `grid_button_label()`. This requires threading it through both call sites (line ~786 and ~997 area in `rebuild_command_panel_system` and `handle_command_key_input`).
     - **Option B**: Use separate `CommandButtonAction` variants (`DcCancelConstruction` and `DcCancelReady`). This is a larger refactor and may not be worth it for just a label change.
     - **Option C** (recommended): Accept that `DcIdle` with cancel just shows `"[X] Cancel"` since the user already sees the DC's status in the info panel, and only add context labels for `DcConstructing` and `DcReadyToPlace` states which already distinguish the two cases. This is simplest and still improves clarity.

### Existing patterns

- The `grid_button_label` function uses the `_state` parameter (currently unused). Activating it by removing the underscore prefix and matching on it is the natural approach.
- Similar context-sensitive labels exist: `BkCancel` uses `"[X] Cancel\nLast"`, `EfCancel` uses `"[X] Cancel"`.
- The 3x3 grid uses GRID_HOTKEYS: Q/W/E (row 0), A/S/D (row 1), Z/X/C (row 2). X is at `(2, 1)`.

### Existing tests to verify (all currently passing)

- `dc_idle_grid_has_open_build_menu` — Q slot works
- `dc_idle_no_construction_no_cancel` — X hidden when nothing active
- `dc_idle_with_construction_shows_cancel` — X visible when constructing/ready
- `dc_idle_with_construction_still_has_build_menu` — Q still works alongside X
- `dc_constructing_back_at_z_cancel_at_x` — DcConstructing layout correct
- `dc_ready_to_place_back_at_z_cancel_at_x` — DcReadyToPlace layout correct
- `dc_full_refund_during_construction` — 150 SC refund for PP construction
- `dc_partial_refund_when_ready_to_place` — 112 SC refund for PP ready (75% of 150)

### Grid position mapping (design doc ↔ code)
- **Q (0,0)**: Build → `DcOpenBuildMenu` ✓
- **X (2,1)**: Cancel Construction / Cancel Ready Building → `DcCancel` (guarded) ✓

## Dependencies

None. This is a verification/polish task on already-implemented functionality. The DC DefaultState cancel commands are self-contained within the command panel and structures modules.
