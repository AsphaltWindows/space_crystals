# Command Panel Module Split

## Date: 2026-03-24
## Status: Architecture request sent
## Priority: 2
## Parent: codebase-refactoring-priorities.md

## Problem Statement

`src/ui/command_panel.rs` is 6,357 lines -- the largest file in the codebase. It mixes five distinct responsibilities into a single file: grid layout/slot mapping, UI rebuild/rendering, action execution, hotkey handling, pointer display resolution, and a large collection of helper functions. This makes the file difficult to navigate, slows down task planning, and increases merge conflict risk when multiple features touch different parts of the panel.

## Current Structure Analysis

The file contains these logical groups:

### 1. Grid Layout and Slot Mapping (lines 1-202, ~200 lines)
- `GRID_HOTKEYS` constant
- `keycode_to_grid_slot()` - maps KeyCode to grid position
- `get_grid_slot_action()` - 160-line match tree mapping (InterfaceState, row, col) to CommandButtonAction

### 2. Cursor Target and Panel State (lines 203-515, ~310 lines)
- `update_cursor_target()` - system that updates CursorTarget resource from mouse raycast
- `selection_owned_by_local_player()` - helper
- `is_panel_visible()` - helper
- `update_command_panel_state()` - system that updates ObjectInterfaceState and SelectedUnitCapabilities

### 3. UI Rebuild and Rendering (lines 516-914, ~400 lines)
- `rebuild_command_panel_ui()` - system that despawns/respawns command panel UI nodes
- Includes all the UI node spawning logic: grid buttons, labels, progress text

### 4. Click and Hotkey Handlers (lines 915-1259, ~340 lines)
- `handle_command_button_clicks()` - system handling button click events
- `command_panel_hotkeys()` - system handling keyboard shortcuts (260 lines)
- `right_click_cancel_target()` - helper
- `right_click_cancel_submenu()` - system

### 5. Action Execution (lines 1260-1891, ~630 lines)
- `execute_command_action()` - the largest function (~560 lines), dispatches all button actions
- `update_command_panel_progress()` - system updating progress bars

### 6. Helper Functions (lines 1893-2780, ~890 lines)
- `compute_selected_unit_capabilities()` - computes unit caps from ECS queries
- `attack_type_can_target_ground()` - utility
- `get_player_sc_from_owners()`, `find_player_resources_mut()` - resource lookup helpers
- `get_dc_construction_info()` - DC info helper
- `execute_tunnel_upgrade()`, `execute_tunnel_cancel_upgrade()`, `execute_tunnel_select_expansion()`, `execute_tunnel_eject_unit()` - tunnel action helpers (~250 lines)
- `build_tunnel_expand_grid()`, `build_tunnel_eject_grid()` - tunnel grid layout helpers (~130 lines)
- `tier_at_least()`, `get_tunnel_upgrade_cost()`, `get_syndicate_supplies()` - tunnel utility helpers
- `grid_button_enabled_ext()` - complex button enablement logic (~80 lines)
- `find_syndicate_resources_mut()` - resource helper
- `is_unit_action()`, `object_type_supports_action()`, `is_common_command()` - action classification
- `active_group_entities()`, `command_target_entities()` - selection helpers
- `grid_button_label()`, `grid_button_enabled()`, `is_action_active()` - UI helpers
- `spawn_panel_title()`, `spawn_progress_text()`, `spawn_info_text()`, `spawn_grid_button()`, `spawn_empty_grid_cell()` - UI node spawning helpers

### 7. Pointer Display Resolution (lines 2780-3008, ~230 lines)
- `CursorEntityInfo` struct
- `resolve_pointer_display()` - pure function for pointer display resolution
- `resolve_awaiting_target()` - helper
- `resolve_default_state()` - helper
- `resolve_pointer_display_type()` - Bevy system wrapping the pure function

### 8. Tests (lines 3009-6357, ~3,350 lines)
- Extensive unit tests covering pointer display, button enablement, grid slots, etc.

## Proposed Refactoring

Convert `command_panel.rs` into a `command_panel/` directory module with these submodules:

### New Directory Structure

```
src/ui/command_panel/
    mod.rs              -- re-exports public systems, registers nothing (plugin stays in ui/mod.rs)
    grid_layout.rs      -- GRID_HOTKEYS, keycode_to_grid_slot, get_grid_slot_action
    state.rs            -- update_cursor_target, update_command_panel_state, selection helpers
    ui_rebuild.rs       -- rebuild_command_panel_ui, spawn_* UI node helpers
    hotkeys.rs          -- command_panel_hotkeys, handle_command_button_clicks, right_click_cancel_submenu
    actions.rs          -- execute_command_action, split into sub-dispatchers by category
    tunnel_actions.rs   -- execute_tunnel_upgrade, execute_tunnel_cancel_upgrade, execute_tunnel_select_expansion, execute_tunnel_eject_unit, build_tunnel_expand_grid, build_tunnel_eject_grid, tunnel utility helpers
    pointer.rs          -- CursorEntityInfo, resolve_pointer_display, resolve_awaiting_target, resolve_default_state, resolve_pointer_display_type
    helpers.rs          -- compute_selected_unit_capabilities, resource lookup helpers, button enablement, action classification, progress update
```

### Key Decisions

1. **Tests stay with their module**: Each submodule gets a `#[cfg(test)] mod tests` block containing the tests relevant to that module's functionality. The ~3,350 lines of tests will be distributed among the submodules they test.

2. **`mod.rs` is thin**: Only contains `pub mod` declarations and `pub use` re-exports for the public systems that `ui/mod.rs` references. No logic.

3. **`execute_command_action` split**: The 560-line function should be broken into sub-dispatchers:
   - `execute_dc_action()` - Deployment Center build/cancel/placement actions
   - `execute_production_action()` - Barracks/HQ/SupplyTower/RC/Armory train/cancel actions
   - `execute_unit_command_action()` - Move/Stop/Patrol/Attack/Hold/Reverse/Enter/Gather actions
   - `execute_tunnel_action()` - All tunnel-specific actions (delegates to tunnel_actions.rs)
   - `execute_placement_action()` - Enter placement mode actions
   The main `execute_command_action` becomes a thin match dispatcher.

4. **Tunnel logic extraction**: Tunnel-related helpers total ~380 lines and form a cohesive group (upgrade, cancel, select expansion, eject, grid builders). They deserve their own file.

5. **No behavioral changes**: All systems maintain the same signatures, same registration order, same behavior. This is a pure file reorganization with one function decomposition (`execute_command_action`).

## Scope

### Files Created
- `src/ui/command_panel/mod.rs`
- `src/ui/command_panel/grid_layout.rs`
- `src/ui/command_panel/state.rs`
- `src/ui/command_panel/ui_rebuild.rs`
- `src/ui/command_panel/hotkeys.rs`
- `src/ui/command_panel/actions.rs`
- `src/ui/command_panel/tunnel_actions.rs`
- `src/ui/command_panel/pointer.rs`
- `src/ui/command_panel/helpers.rs`

### Files Deleted
- `src/ui/command_panel.rs` (replaced by directory module)

### Files Modified
- `src/ui/mod.rs` -- update `mod command_panel` (no path change needed if directory has mod.rs)

### Expected Line Counts (approximate)
- grid_layout.rs: ~200 lines + ~200 lines tests = ~400
- state.rs: ~310 lines + ~300 lines tests = ~610
- ui_rebuild.rs: ~400 lines + ~100 lines tests = ~500
- hotkeys.rs: ~340 lines + ~400 lines tests = ~740
- actions.rs: ~630 lines + ~600 lines tests = ~1,230
- tunnel_actions.rs: ~380 lines + ~200 lines tests = ~580
- pointer.rs: ~230 lines + ~1,200 lines tests = ~1,430
- helpers.rs: ~350 lines + ~350 lines tests = ~700
- mod.rs: ~30 lines

No file exceeds ~1,500 lines. The largest (pointer.rs) is heavy on tests, which is acceptable.

## Constraints

- Behavioral equivalence: identical gameplay behavior before and after
- System registration in `ui/mod.rs` must reference the same system functions (via re-exports from `command_panel/mod.rs`)
- All existing unit tests must pass without behavioral changes (only import path updates)
- All integration tests must pass
- No new Bevy systems -- only reorganization and function decomposition

## Success Metrics

- `command_panel.rs` replaced by 9 files, none exceeding 1,500 lines
- `execute_command_action` reduced from ~560 lines to a ~30-line dispatcher
- Each submodule has a single, clear responsibility
- Adding a new structure menu state requires editing at most 2 files (grid_layout.rs + actions.rs)
