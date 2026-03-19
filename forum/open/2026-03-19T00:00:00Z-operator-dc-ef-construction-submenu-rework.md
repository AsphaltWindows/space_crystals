# DC/EF Construction Sub-Menu Rework Needs Designer Feature Request

## Metadata
- **Created by**: operator
- **Created**: 2026-03-19T00:00:00Z
- **Status**: open

## Close Votes

## Discussion

### [operator] 2026-03-19T00:00:00Z

There is an outstanding task that needs to re-enter the pipeline properly as a **feature_request** message for the **task_splitter**. **Designer**: please review and produce an appropriate feature_request.

---

### DC/EF Should Not Auto-Enter Construction Sub-Menu on Selection

**Problem:** `update_command_panel_state()` forces DC into DcConstructing and EF into EfConstructing every frame when a construction is active. This means selecting a constructing structure dumps the player into a sub-menu, and pressing Z/Back is immediately overridden on the next frame. The player can never stay in the default view while construction is active.

**Fix:** Treat constructing/ready-to-place the same as idle for state preservation. Only set the initial state on target change, and preserve any valid DC/EF state the player has navigated to. Selecting a DC should always land on DcIdle; the player navigates to DcConstructing via the build menu, not by auto-sync.

**Dependency:** This must be completed *before* dc_default_state_cancel_commands (that task depends on this one, since DcCancel in DcIdle is invisible if the player is forced into DcConstructing every frame).

**QA Steps:**
1. [auto] Build a Power Plant from the DC and while it is constructing, deselect and re-select the DC -- verify the interface shows DefaultState (DcIdle with Build Q and Cancel X), not the DcConstructing sub-menu.
2. [human] While DC is constructing, press Q to enter BuildMenu -- verify you now see the Constructing sub-menu with Cancel (X) and Back (Z).
3. [auto] Press Z or Escape from the BuildMenu Constructing view -- verify you return to DefaultState (DcIdle) and stay there (not re-forced into sub-menu).
4. [auto] Let a DC construction complete to ready-to-place state, deselect and re-select the DC -- verify DefaultState is shown (with Cancel X and Build Q), not the ready-to-place sub-menu.
5. [auto] Build an Extraction Plate from the EF and while it is constructing, deselect and re-select the EF -- verify EfIdle is shown (with Cancel X), not EfConstructing.
6. [auto] Press Z or Escape from EfConstructing -- verify you return to EfIdle and stay there.

**Previous QA Failures (must be addressed in the feature_request):**

1. **EF interface design misunderstanding**: The EF should NOT use a sub-menu like DC. The EF default view should directly show "Build Plate" (Q). When constructing, Q slot should show Cancel (X). When ready to place, Q slot should show "Place Plate" and X remains for cancel. The EF flow should be **flat** (no sub-menu), unlike DC which has multiple build options. This is a design clarification, not a code bug.

2. **DC/EF construction progress bar stale in DefaultState**: When viewing DC or EF in DcIdle/EfIdle while a construction is active, the progress bar and InfoPanel "Status" field do not update in real time -- they freeze at the value from when the structure was first selected. Progress only updates when navigating into the constructing sub-menu. Both should reflect live construction progress regardless of which menu state the player is in.

**Action needed from designer**: Produce a feature_request that covers:
- The original fix (stop auto-forcing into construction sub-menus)
- The EF flat interface redesign
- The real-time progress bar fix
- Updated QA steps reflecting the new design expectations
