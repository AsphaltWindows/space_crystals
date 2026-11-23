# core_systems_split

## Metadata
- **From**: qa_router
- **To**: automatic_qa

## Content

## Content

## Content

## Architecture Document

artifacts/software_architect/architecture/core-systems-split.md

## Summary

Split the monolithic `src/game/units/systems/core.rs` (6,054 lines) into a `core/` directory module with 8 submodules: command_dispatch, rally, legacy_movement, channel_locomotion, occupancy, indicators, utility, and mod.rs. The P1 right-click-command-dispatch refactor extracted handler functions but they remained in the same file. This split separates the 6 unrelated responsibility groups (command dispatch, legacy movement, channel locomotion, occupancy, visual indicators, rally systems) into dedicated files. No behavioral changes.

## Scope

- **Primary**: `src/game/units/systems/core.rs` -> `src/game/units/systems/core/` directory (8 files)
- **Minor update**: `src/game/units/systems/mod.rs` -- `pub mod core` stays the same (resolves to `core/mod.rs`)
- **No changes to**: system signatures, system registration, game logic, or any other module

## QA Instructions

1. `cargo check` must pass with no errors
2. `cargo test --lib` must pass -- all existing unit tests (including ~3,140 lines of core.rs tests) must pass with only import path changes
3. `cargo test --test qa` (integration tests) must pass
4. Verify that all systems registered in the units plugin resolve correctly via re-exports from `core/mod.rs`
5. No file in the new `core/` directory should exceed 2,300 lines (legacy_movement.rs will be the largest due to 7 movement system implementations + tests)

## Testing Requirements

- **No new tests required**: This is a pure reorganization. All existing tests must pass with only `use` path updates.
- **Regression**: All ~3,140 lines of existing core.rs unit tests must continue to pass in their new submodule locations.
- **Integration**: All integration tests that exercise movement, command dispatch, or visual indicators must pass.
- **Compile-time check**: Verify that `CommandContext`, `try_issue_command`, and all public system functions are correctly re-exported.
