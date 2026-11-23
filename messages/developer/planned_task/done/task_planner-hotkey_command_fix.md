# hotkey_command_fix

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-unit_command_system_r1.md

## Task

Fix command hotkeys (Q=Move, W=Reverse, E=HoldPosition, A=Attack, S=Patrol, D=AttackGround, X=Stop) so they respond when units are selected.

**Bug**: None of the command hotkeys respond when a unit is selected. The command panel grid buttons may work via mouse click, but keyboard hotkeys produce no effect. This blocks testing of HoldPosition, Stop, Patrol, Reverse, AttackGround, and AttackMove commands.

**Investigation areas**:
- The `command_panel_hotkeys` system (command_panel.rs line 946) has a guard at line 965: `if !is_panel_visible(...) || !unit_caps.owned_by_local_player { return; }`. Check that `owned_by_local_player` is being set correctly by `update_command_panel_state` (line 324+). If this field defaults to false and the update system does not run before hotkeys, all hotkeys would be silently blocked.
- Check system ordering: `update_command_panel_state` must run before `command_panel_hotkeys` so that `SelectedUnitCapabilities.owned_by_local_player` is accurate.
- Check that `is_panel_visible` returns true when units are selected (line 310-322): it checks `selection.groups.is_empty()` and filters out resource-only selections.
- Check `SelectedUnitCapabilities` resource initialization -- does it default to owned_by_local_player=true or false?
- Check that the legacy `command_input_system` (commands.rs line 14) does not interfere -- it returns early when panel_visible is true (line 25-33), which is correct, but verify the two systems do not conflict.
- Test: select a Peacekeeper, press E (HoldPosition), verify the unit stops. Press Q (Move), verify AwaitingTarget mode. Press A (Attack), verify AwaitingTarget mode. Press X (Stop), verify unit ceases activity.

## Technical Context

### Root Cause

The bug is a **system ordering issue** in Bevy ECS. The `command_panel_hotkeys` system has no `.after()` constraint relative to `update_command_panel_state`, so it can (and does) run before the ownership flag is set.

**Evidence chain**:

1. `SelectedUnitCapabilities` derives `Default` (via `#[derive(Resource, Default)]` at `artifacts/developer/src/ui/types.rs` line 401). The `Default` trait sets all bools to `false`, including `owned_by_local_player`.

2. `update_command_panel_state` (`artifacts/developer/src/ui/command_panel.rs` line 325) sets `unit_caps.owned_by_local_player` to the correct value (lines 340-343) by calling `selection_owned_by_local_player()`.

3. `command_panel_hotkeys` (`artifacts/developer/src/ui/command_panel.rs` line 946) reads `unit_caps.owned_by_local_player` at line 965 as an early-return guard: `if !is_panel_visible(...) || !unit_caps.owned_by_local_player { return; }`.

4. **In `artifacts/developer/src/ui/mod.rs` line 47**, `command_panel_hotkeys` is registered with NO ordering constraint:
   ```
   command_panel::command_panel_hotkeys,  // line 47 -- no .after()!
   ```
   Meanwhile `update_command_panel_state` is registered at line 44 with `.after(command_panel::update_cursor_target)`, but nothing chains hotkeys after it.

5. Because Bevy runs unordered systems in arbitrary order within the same schedule phase, `command_panel_hotkeys` frequently runs BEFORE `update_command_panel_state`. When it does, `owned_by_local_player` is still `false` from the `Default` initialization, and ALL hotkey processing is silently skipped.

### The Fix

**File**: `artifacts/developer/src/ui/mod.rs` (line 47)

Change line 47 from:
```rust
command_panel::command_panel_hotkeys,
```
to:
```rust
command_panel::command_panel_hotkeys.after(command_panel::update_command_panel_state),
```

This ensures `update_command_panel_state` has already computed `owned_by_local_player` (and `is_panel_visible` data via `SelectedUnitCapabilities`) before `command_panel_hotkeys` reads it.

### Additional system that needs the same fix

`right_click_cancel_submenu` (line 48) reads `ObjectInterfaceState` which is also written by `update_command_panel_state`. It should also get `.after(command_panel::update_command_panel_state)` to avoid stale state, though this is less critical since it checks the interface state enum rather than the ownership flag.

### Verification

After the fix, the hotkey flow is:
1. `update_cursor_target` runs (resolves what cursor is over)
2. `update_command_panel_state` runs (sets `owned_by_local_player = true` for player-owned selections)
3. `command_panel_hotkeys` runs (reads `owned_by_local_player`, processes Q/W/E/A/S/D/Z/X/C keys)
4. Key press -> `keycode_to_grid_slot()` -> `get_grid_slot_action()` -> `execute_command_action()` -> command issued

The grid slot mapping in `get_grid_slot_action()` (line 157) for `ObjectInterfaceState::Default`:
- Q(0,0) = UnitMove -> sets AwaitingTarget(Move)
- W(0,1) = UnitReverse (if can_reverse) -> sets AwaitingTarget(Reverse)
- E(0,2) = UnitHoldPosition -> immediate command
- A(1,0) = UnitAttack (if has_attack) -> sets AwaitingTarget(Attack)
- S(1,1) = UnitPatrol -> sets AwaitingTarget(Patrol)
- D(1,2) = UnitAttackGround (if can_target_ground) -> sets AwaitingTarget(AttackGround)
- X(2,1) = UnitStop -> immediate command

### No conflicts with legacy system

The legacy `command_input_system` (`artifacts/developer/src/game/units/systems/commands.rs` line 14) correctly returns early when `panel_visible` is true (line 25-33). It only handles legacy hotkeys (M, A, G, T, P) when the panel is NOT visible. No conflict.

## Dependencies

None. This is a standalone ordering fix in the UI plugin registration. No other tasks need to land first.
