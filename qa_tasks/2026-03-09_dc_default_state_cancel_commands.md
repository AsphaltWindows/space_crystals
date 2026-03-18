# Task: Add Cancel commands to DC DefaultState

## Source Ticket
`tickets/2026-03-09_dc_default_state_cancel_commands.md`

## Summary
DC DefaultState (DcIdle) should show a conditional Cancel (X) command at slot (2,1) when the DC has an active construction or a ready-to-place building. This matches EF's existing pattern and gives players one-keypress cancel access.

## Dependencies
- **`2026-03-09_dc_ef_no_auto_enter_construction_submenu`** — Must be completed first. Currently `update_command_panel_state()` forces `DcConstructing` state whenever `current_construction.is_some()`, so the player can never stay in DcIdle while constructing. Without that fix, the cancel button added here would never be visible.

## Technical Context

### Files to Change

**`src/ui/command_panel.rs`** — All changes are in this file.

#### 1. Add `dc_has_construction` parameter to `get_grid_slot_action()` (line 42)

Current signature:
```rust
fn get_grid_slot_action(state: &ObjectInterfaceState, row: u8, col: u8, bk_has_queue: bool, caps: &SelectedUnitCapabilities) -> Option<CommandButtonAction>
```

Add a `dc_has_construction: bool` parameter (follows the `bk_has_queue` pattern for conditional slot visibility). This bool should be true when either `dc.current_construction.is_some()` or `dc.ready_to_place.is_some()`.

#### 2. Add DcCancel to DcIdle grid (lines 51-54)

Current:
```rust
StructureMenuState::DcIdle => match (row, col) {
    (0, 0) => Some(CommandButtonAction::DcOpenBuildMenu),
    _ => None,
},
```

Change to:
```rust
StructureMenuState::DcIdle => match (row, col) {
    (0, 0) => Some(CommandButtonAction::DcOpenBuildMenu),
    (2, 1) if dc_has_construction => Some(CommandButtonAction::DcCancel),
    _ => None,
},
```

#### 3. Compute `dc_has_construction` at both call sites

**UI rendering call site (line 704):** Already has access to DC state via `selected_owners` query area. Compute the bool from `panel_target.entity` + `dc_query` (similar to `bk_has_queue` computation at lines 685-695). The DC query is already available in this system — check `dc_query` or the selected structures query for `DeploymentCenterState`.

**Hotkey handler call site (line 897):** Has access to `dc_query` (mutable). Compute `dc_has_construction` similarly:
```rust
let dc_has_construction = panel_target.entity
    .and_then(|e| dc_query.get(e).ok())
    .map(|(_, dc)| dc.current_construction.is_some() || dc.ready_to_place.is_some())
    .unwrap_or(false);
```

#### 4. Update all test call sites

Every call to `get_grid_slot_action` in tests needs the new parameter. There are ~80 call sites in tests — all currently pass `false` for `bk_has_queue` and should similarly pass `false` for `dc_has_construction` unless the specific test is testing DC cancel visibility.

#### 5. Add tests for the new behavior

Add tests verifying:
- `DcIdle` with `dc_has_construction=false` → slot (2,1) returns `None`
- `DcIdle` with `dc_has_construction=true` → slot (2,1) returns `Some(DcCancel)`
- `DcIdle` with `dc_has_construction=true` → slot (0,0) still returns `DcOpenBuildMenu`

### Existing Handler Reuse

The `DcCancel` handler (line 959-983) already handles both `current_construction` and `ready_to_place` cancellation with correct refunds (full during construction, 75% when ready). **No changes needed to the handler** — it works regardless of which menu state triggered it.

### Label Function

`grid_button_label()` already has a `DcCancel` match arm (line 1803) that returns `"[X] Cancel"`. No changes needed.

### Pattern Notes
- Follow the `bk_has_queue` pattern exactly — it's the established convention for conditional slot visibility
- The `DcCancel` action is already defined in `CommandButtonAction` enum — no new variants needed
- EF already has a similar need (Cancel in EfIdle when constructing) but that's scoped to the companion ticket, not this one

## QA Steps
1. [auto] Select a DC that is not constructing — verify only Build (Q) appears in the command panel, no Cancel (X).
2. [auto] Start a construction (e.g., Power Plant) and return to DefaultState — verify Cancel Construction (X) now appears alongside Build (Q).
3. [auto] Press X during construction from DefaultState — verify full refund is received (150 SC for Power Plant) and the DC returns to idle.
4. [auto] Start another construction, let it complete to ready-to-place, return to DefaultState — verify Cancel Ready Building (X) appears.
5. [auto] Press X from DefaultState when ready to place — verify 75% refund (rounded down, e.g., 112 SC for Power Plant) is received and the DC returns to idle.
6. [auto] Enter BuildMenu (Q) while constructing — verify Cancel (X) is still available inside BuildMenu as well (both paths work).
7. [auto] Verify the X slot position is consistent: Cancel should appear at (2,1) in both DefaultState and BuildMenu contexts.

## Expected Experience
The player always has Cancel within one keypress when a DC is constructing or has a building ready to place. There's no need to remember to enter the BuildMenu first — X from the default view handles it.
