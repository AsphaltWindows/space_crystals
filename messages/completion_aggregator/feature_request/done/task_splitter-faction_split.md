# faction_split

## Metadata
- **From**: task_splitter
- **To**: completion_aggregator

## Content

## Architecture Document

artifacts/software_architect/architecture/faction-split.md

## Summary

Split the monolithic `src/game/world/faction.rs` (4,387 lines) into a `faction/` directory module with 8 submodules: setup, production, construction, placement, rally, resources, cults, and mod.rs. The file contains 7+ distinct responsibility domains (faction setup, production, construction, placement, rally points, resource display, Cults-specific logic). This split separates them into dedicated files. Importantly, grouping all 5 production tick systems into one file makes the copy-paste duplication visible and prepares for future production trait deduplication. No behavioral changes.

## Scope

- **Primary**: `src/game/world/faction.rs` -> `src/game/world/faction/` directory (8 files)
- **Minor update**: `src/game/world/mod.rs` -- `pub mod faction` stays the same
- **Cross-module dependency**: `spawn_or_update_rally_marker` and `despawn_rally_marker_for` are imported by `src/game/units/systems/core.rs` -- they must remain `pub` and be re-exported from `faction/mod.rs`
- **No changes to**: system signatures, FactionPlugin system registration, game logic

## QA Instructions

1. `cargo check` must pass with no errors
2. `cargo test --lib` must pass -- all existing unit tests (including ~1,800 lines of faction tests) must pass with only import path changes
3. `cargo test --test qa` (integration tests) must pass
4. Verify that `FactionPlugin::build()` in `world/mod.rs` correctly resolves all system references via re-exports
5. Verify that `core.rs` (or `core/command_dispatch.rs` if the core split lands first) can still import `spawn_or_update_rally_marker` and `despawn_rally_marker_for` from `game::world::faction`
6. No file in the new `faction/` directory should exceed 1,200 lines

## Testing Requirements

- **No new tests required**: This is a pure reorganization. All existing tests must pass with only `use` path updates.
- **Regression**: All ~1,800 lines of existing faction.rs unit tests must continue to pass in their new submodule locations.
- **Integration**: All integration tests that exercise production, placement, rally points, or faction setup must pass.
- **Compile-time check**: Verify that cross-module imports (rally marker functions used by units/systems/core) resolve correctly via re-exports.
