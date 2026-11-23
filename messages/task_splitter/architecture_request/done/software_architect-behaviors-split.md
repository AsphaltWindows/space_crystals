# behaviors-split

## Metadata
- **From**: software_architect
- **To**: task_splitter

## Content

## Architecture Document

artifacts/software_architect/architecture/behaviors-split.md

## Summary

Split the monolithic `src/game/units/systems/behaviors.rs` (4,481 lines) into a `behaviors/` directory module with 6 submodules: movement, tunnel_interaction, construction, resource_gathering, supply_chopper, and mod.rs. The file contains 5 distinct behavior domains (movement, tunnel interaction, construction, resource gathering, supply chopper) that share minimal logic. This split separates them into dedicated files. No behavioral changes.

## Scope

- **Primary**: `src/game/units/systems/behaviors.rs` -> `src/game/units/systems/behaviors/` directory (6 files)
- **Minor update**: `src/game/units/systems/mod.rs` -- `pub mod behaviors` stays the same
- **No changes to**: system signatures, system registration, game logic, or any other module

## QA Instructions

1. `cargo check` must pass with no errors
2. `cargo test --lib` must pass -- all existing unit tests (including ~2,810 lines of behaviors tests) must pass with only import path changes
3. `cargo test --test qa` (integration tests) must pass
4. Verify all behavior systems are correctly re-exported from `behaviors/mod.rs`
5. No file in the new `behaviors/` directory should exceed 1,300 lines

## Testing Requirements

- **No new tests required**: This is a pure reorganization. All existing tests must pass with only `use` path updates.
- **Regression**: All ~2,810 lines of existing behaviors.rs unit tests must continue to pass in their new submodule locations.
- **Integration**: All integration tests that exercise unit behaviors (movement, resource gathering, tunnel entry, supply chopper) must pass.
- **Compile-time check**: Verify that shared helpers like `extract_path_data`, `has_path_deviation`, `tunnel_side_world_position`, and `find_nearest_own_tunnel` are accessible to the submodules that need them via `pub(super)` or `pub(crate)` visibility.
