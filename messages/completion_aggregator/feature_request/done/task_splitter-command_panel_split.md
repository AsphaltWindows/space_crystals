# command_panel_split

## Metadata
- **From**: task_splitter
- **To**: completion_aggregator

## Content

## Content

## Architecture Document

artifacts/software_architect/architecture/command-panel-split.md

## Summary

Split the monolithic `src/ui/command_panel.rs` (6,357 lines) into a `command_panel/` directory module with 9 submodules: grid_layout, state, ui_rebuild, hotkeys, actions, tunnel_actions, pointer, helpers, and mod.rs. Also decompose the 560-line `execute_command_action` function into sub-dispatchers by category (DC, production, unit commands, tunnel, placement). This is a pure file reorganization with one function decomposition -- no behavioral changes.

## Scope

- **Primary**: `src/ui/command_panel.rs` -> `src/ui/command_panel/` directory (9 files)
- **Minor update**: `src/ui/mod.rs` -- module declaration unchanged (Rust resolves `mod command_panel` to `command_panel/mod.rs`)
- **No changes to**: game logic, system signatures, system registration order, or any other module

## QA Instructions

1. `cargo check` must pass with no errors
2. `cargo test --lib` must pass -- all existing unit tests (including ~3,350 lines of command panel tests) must pass with only import path changes
3. `cargo test --test qa` (integration tests) must pass
4. Verify that `src/ui/mod.rs` references the same system functions (e.g., `command_panel::update_cursor_target`, `command_panel::rebuild_command_panel_ui`) and they resolve correctly via re-exports
5. No file in the new `command_panel/` directory should exceed 1,500 lines

## Testing Requirements

- **No new tests required**: This is a pure reorganization. All existing tests must pass with only `use` path updates.
- **Regression**: All ~3,350 lines of existing command_panel unit tests must continue to pass in their new submodule locations.
- **Integration**: All integration tests in `tests/qa/` that exercise command panel functionality must pass.
- **Compile-time check**: Verify that all `pub use` re-exports in `command_panel/mod.rs` correctly expose the systems that `ui/mod.rs` and other modules import.
