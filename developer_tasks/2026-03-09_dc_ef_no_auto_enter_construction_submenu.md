# Task: DC/EF should not auto-enter construction sub-menu on selection

## Source Ticket
`tickets/2026-03-09_dc_ef_no_auto_enter_construction_submenu.md`

## Summary
`update_command_panel_state()` forces DC into `DcConstructing` and EF into `EfConstructing` every frame when a construction is active. This means selecting a constructing structure dumps the player into a sub-menu, and pressing Z/Back is immediately overridden. The fix: treat constructing/ready-to-place the same as idle for state preservation — only set the initial state on target change, and preserve any valid DC/EF state the player has navigated to.

## Dependencies
- **None** — This must be completed *before* `2026-03-09_dc_default_state_cancel_commands` (that task depends on this one, since DcCancel in DcIdle is invisible if the player is forced into DcConstructing every frame).

## Technical Context

### File to Change
**`src/ui/command_panel.rs`** — All changes are in this single file.

### Change 1: DC branch in `update_command_panel_state()` (lines 317-339)

**Current logic (lines 318-339):**
```rust
if let Some(dc) = dc_state {
    if dc.current_construction.is_some() {
        let new_state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcConstructing);
        if *interface_state != new_state || target_changed {
            *interface_state = new_state;
        }
    } else if dc.ready_to_place.is_some() {
        // Preserve AwaitingPlacement state when structure is still ready
        if *interface_state != ObjectInterfaceState::StructureMenu(StructureMenuState::DcReadyToPlace)
            && *interface_state != ObjectInterfaceState::StructureMenu(StructureMenuState::DcAwaitingPlacement)
            || target_changed
        {
            *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcReadyToPlace);
        }
    } else if target_changed || !matches!(*interface_state,
        ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle) |
        ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu)
    ) {
        *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle);
    }
}
```

**Problem:** The first branch (`current_construction.is_some()`) forces `DcConstructing` every frame. The second branch (`ready_to_place.is_some()`) similarly forces `DcReadyToPlace` (though it does preserve `DcAwaitingPlacement`).

**Required change:** Unify all DC branches so they only set the initial state on `target_changed`, and otherwise preserve any valid DC sub-state the player has navigated to. The new logic should be:

```rust
if let Some(dc) = dc_state {
    let in_valid_dc_state = matches!(*interface_state,
        ObjectInterfaceState::StructureMenu(
            StructureMenuState::DcIdle |
            StructureMenuState::DcBuildMenu |
            StructureMenuState::DcConstructing |
            StructureMenuState::DcReadyToPlace |
            StructureMenuState::DcAwaitingPlacement
        )
    );
    if target_changed || !in_valid_dc_state {
        *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle);
    }
}
```

This means:
- Selecting a DC (new target) always lands on `DcIdle`
- Re-selecting the same DC preserves whatever DC state the player is in (idle, build menu, constructing sub-menu, etc.)
- Player navigates to `DcConstructing` via the build menu, not by auto-sync
- `DcAwaitingPlacement` is preserved (player entered it via EnterPlacement command)
- The `DcConstructing`/`DcReadyToPlace` states entered via build commands (line 950, 930) still work — this change only removes the *frame-by-frame override*

**Important edge case:** When construction completes (transitions from `current_construction.is_some()` to `ready_to_place.is_some()`), and the player happens to be in `DcConstructing` sub-state, they'll stay in `DcConstructing` even though the DC is now ready-to-place. This is acceptable — the `DcConstructing` sub-menu shows Back and Cancel, and the construction-complete event should still be handled by the command handler at line 1283-1300 which transitions the state. Verify this handler still fires correctly.

### Change 2: EF branch in `update_command_panel_state()` (lines 347-364)

**Current logic (lines 348-364):**
```rust
if let Some(ef) = ef_state {
    if ef.current_construction {
        let new_state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfConstructing);
        if *interface_state != new_state || target_changed {
            *interface_state = new_state;
        }
    } else if ef.ready_to_place {
        if *interface_state != ObjectInterfaceState::StructureMenu(StructureMenuState::EfReadyToPlace)
            && *interface_state != ObjectInterfaceState::StructureMenu(StructureMenuState::EfAwaitingPlacement)
            || target_changed
        {
            *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfReadyToPlace);
        }
    } else if target_changed || !matches!(*interface_state, ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle)) {
        *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle);
    }
}
```

**Required change:** Same pattern as DC:

```rust
if let Some(_ef) = ef_state {
    let in_valid_ef_state = matches!(*interface_state,
        ObjectInterfaceState::StructureMenu(
            StructureMenuState::EfIdle |
            StructureMenuState::EfConstructing |
            StructureMenuState::EfReadyToPlace |
            StructureMenuState::EfAwaitingPlacement
        )
    );
    if target_changed || !in_valid_ef_state {
        *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle);
    }
}
```

Note: EF doesn't have a `BuildMenu` sub-state — `EfBuildPlate` is a direct action from `EfIdle`.

### Change 3: Construction completion handler (lines 1283-1320)

The existing handler at lines 1283-1320 transitions `DcConstructing` → `DcIdle` and `EfConstructing` → `EfIdle` when construction completes or is cancelled. **Verify this still works correctly** — it should, since it matches on the current `interface_state` and updates it. But now that the player might be in `DcIdle` while construction is active (the normal case after this fix), this handler should also handle the case where construction completes while the player is in `DcIdle`. Check if it needs to do anything in that case (likely not — `DcIdle` is already the correct post-completion state).

### Change 4: Tests

Update existing tests in the `#[cfg(test)]` module that test `update_command_panel_state` behavior for DC/EF. Specifically:
- Tests asserting that constructing DCs enter `DcConstructing` should be updated to assert `DcIdle` instead
- Tests asserting that ready-to-place DCs enter `DcReadyToPlace` should assert `DcIdle`
- Add new tests verifying:
  - Selecting a constructing DC → `DcIdle` (not `DcConstructing`)
  - Selecting a constructing EF → `EfIdle` (not `EfConstructing`)
  - Re-selecting same constructing DC while in `DcBuildMenu` → stays in `DcBuildMenu`
  - Re-selecting same constructing DC while in `DcConstructing` → stays in `DcConstructing` (navigated there manually)
  - Z/Back from `DcConstructing` → `DcIdle`, stays `DcIdle` on next frame (no override)

### Pattern Reference
The Tunnel state management (lines 372-388) already uses this "preserve valid state" pattern — it checks `in_tunnel_state` and only sets initial state on `target_changed` or when not in a valid tunnel state. DC and EF should match this pattern exactly.

## QA Steps
1. [auto] Build a Power Plant from the DC and while it is constructing, deselect and re-select the DC — verify the interface shows DefaultState (DcIdle with Build Q and Cancel X), not the DcConstructing sub-menu.
2. [human] While DC is constructing, press Q to enter BuildMenu — verify you now see the Constructing sub-menu with Cancel (X) and Back (Z).
3. [auto] Press Z or Escape from the BuildMenu Constructing view — verify you return to DefaultState (DcIdle) and stay there (not re-forced into sub-menu).
4. [auto] Let a DC construction complete to ready-to-place state, deselect and re-select the DC — verify DefaultState is shown (with Cancel X and Build Q), not the ready-to-place sub-menu.
5. [auto] Build an Extraction Plate from the EF and while it is constructing, deselect and re-select the EF — verify EfIdle is shown (with Cancel X), not EfConstructing.
6. [auto] Press Z or Escape from EfConstructing — verify you return to EfIdle and stay there.

## Expected Experience
Selecting a constructing structure feels natural — the player sees the default command panel showing what's being built (with a Cancel shortcut right there), rather than being dumped into a sub-menu. Navigation into build menus is always the player's choice, never forced. Z/Escape reliably returns to the default view without being immediately overridden.

## QA Failure — 2026-03-09
- Step 1 [human]: PASS — Re-selecting constructing DC shows DcIdle with Build (Q) and Cancel (X).
- Step 2 [human]: FAIL — Pressing Q during construction shows the normal Build Menu (build options) with no visual indicator of the ongoing construction or its progress. Player has no visibility into what's being built.
- **Placement regression (CRITICAL)**: After construction completes and the building is ready-to-place, the DC stays in DcIdle. The player expects to press Q → enter Build Menu → click the same building option to enter placement mode, but this path doesn't work. The ready-to-place building's slot in the Build Menu should context-switch to trigger placement mode (rather than starting a new build). Currently there is no way to place a completed building, breaking the core DC build flow.
- **Same issue affects EF**: Deselecting an EF that's constructing/ready-to-place and re-selecting it also loses access to placement and cancel. Both DC and EF become effectively useless after any deselection during a build. This is a **high-priority regression** that blocks all GDO structure construction if the player ever clicks away from the building structure.

## QA Results — 2026-03-09 (re-test)
- Step 1 [human]: PASS — Re-selecting constructing DC shows DcIdle (not DcConstructing)
- Step 2 [human]: PASS — Q enters BuildMenu with Cancel/Back visible
- Step 3 [auto]: PASS — Z/Escape returns to DcIdle and stays
- Step 4 [auto]: PASS — Ready-to-place DC shows DefaultState on re-select
- Step 4 placement: PASS — Placement regression fixed, can place completed buildings
- Step 5 [auto]: PASS — Re-selecting constructing EF shows EfIdle
- **Step 6 [human]: FAIL — EF interface design is wrong.** The EF should NOT use a sub-menu like DC. The EF default view should directly show "Build Plate" (Q). When constructing, Q slot should show Cancel (X). When ready to place, Q slot should show "Place Plate" and X remains for cancel. Currently the player must navigate Q → BuildMenu → select plate even when ready to place, which is unnecessary complexity for a single-option structure. This is a design misunderstanding, not a code bug — the EF flow should be flat (no sub-menu), unlike DC which has multiple build options.
- **Additional bug: DC/EF construction progress bar stale in DefaultState.** When viewing DC or EF in DcIdle/EfIdle while a construction is active, the progress bar does not update in real time — it freezes at the value it had when the structure was first selected. Progress only updates when navigating into the constructing sub-menu. The InfoPanel "Status" field also does not update in real time. Both should reflect live construction progress regardless of which menu state the player is in.
