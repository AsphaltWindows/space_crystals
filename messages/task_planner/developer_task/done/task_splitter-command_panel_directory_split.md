# command_panel_directory_split

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-command_panel_split.md

## Task

Split the monolithic `src/ui/command_panel.rs` (6,357 lines) into a `command_panel/` directory module with 8 submodules plus mod.rs. This is a pure file reorganization -- no behavioral changes.

### Directory Structure to Create

```
src/ui/command_panel/
    mod.rs              -- pub mod declarations + pub use re-exports for all public systems
    grid_layout.rs      -- GRID_HOTKEYS, keycode_to_grid_slot(), get_grid_slot_action() (~200 lines code + ~200 lines tests)
    state.rs            -- update_cursor_target(), update_command_panel_state(), selection_owned_by_local_player(), is_panel_visible() (~310 lines code + ~300 lines tests)
    ui_rebuild.rs       -- rebuild_command_panel_ui(), spawn_panel_title(), spawn_progress_text(), spawn_info_text(), spawn_grid_button(), spawn_empty_grid_cell() (~400 lines code + ~100 lines tests)
    hotkeys.rs          -- command_panel_hotkeys(), handle_command_button_clicks(), right_click_cancel_target(), right_click_cancel_submenu() (~340 lines code + ~400 lines tests)
    actions.rs          -- execute_command_action(), update_command_panel_progress() (~630 lines code + ~600 lines tests)
    tunnel_actions.rs   -- execute_tunnel_upgrade(), execute_tunnel_cancel_upgrade(), execute_tunnel_select_expansion(), execute_tunnel_eject_unit(), build_tunnel_expand_grid(), build_tunnel_eject_grid(), tier_at_least(), get_tunnel_upgrade_cost(), get_syndicate_supplies() (~380 lines code + ~200 lines tests)
    pointer.rs          -- CursorEntityInfo struct, resolve_pointer_display(), resolve_awaiting_target(), resolve_default_state(), resolve_pointer_display_type() (~230 lines code + ~1,200 lines tests)
    helpers.rs          -- compute_selected_unit_capabilities(), attack_type_can_target_ground(), get_player_sc_from_owners(), find_player_resources_mut(), get_dc_construction_info(), find_syndicate_resources_mut(), grid_button_enabled_ext(), grid_button_enabled(), grid_button_label(), is_action_active(), is_unit_action(), object_type_supports_action(), is_common_command(), active_group_entities(), command_target_entities() (~350 lines code + ~350 lines tests)
```

### Implementation Steps

1. Create directory `src/ui/command_panel/`
2. Delete `src/ui/command_panel.rs`
3. Create each submodule file, moving the corresponding code blocks from the original file (refer to the architecture document at `artifacts/software_architect/architecture/command-panel-split.md` for exact line ranges)
4. Create `mod.rs` with:
   - `pub mod` declarations for all 8 submodules
   - `pub use` re-exports for all public systems that `ui/mod.rs` references: update_cursor_target, update_command_panel_state, rebuild_command_panel_ui, handle_command_button_clicks, command_panel_hotkeys, right_click_cancel_submenu, update_command_panel_progress, resolve_pointer_display_type
5. Update `use` paths within submodules -- functions that were previously in the same file now need `use super::` or `use crate::` imports to reference each other across submodules
6. Move each test block into the submodule that contains the code it tests, updating `use` paths as needed
7. Verify `src/ui/mod.rs` does NOT need changes (Rust resolves `mod command_panel` to `command_panel/mod.rs` automatically)

### Constraints

- No behavioral changes -- identical gameplay before and after
- No file in the new directory should exceed 1,500 lines
- All existing unit tests must pass with only import path changes
- All integration tests must pass
- System registration in ui/mod.rs must continue to work via the re-exports

### Verification

- `cargo check` must pass
- `cargo test --lib` must pass (all ~1,900+ unit tests including ~3,350 lines of command panel tests)
- `cargo test --test qa` must pass
