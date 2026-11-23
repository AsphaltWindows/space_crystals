# core_systems_split

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-core_systems_split.md

## Task

Split the monolithic `src/game/units/systems/core.rs` (6,054 lines) into a `src/game/units/systems/core/` directory module with 8 submodules. This is a pure structural refactoring with no behavioral changes.

### Architecture Document

Read `artifacts/software_architect/architecture/core-systems-split.md` for the full analysis including exact line ranges for each responsibility group.

### What to Do

1. Create directory `src/game/units/systems/core/`
2. Create the following files by extracting from the current `core.rs`:

   - **mod.rs** (~40 lines): Declare submodules, re-export all public items so that existing `use crate::game::units::systems::core::*` imports continue to work unchanged.

   - **command_dispatch.rs** (~710 code + ~800 test lines): `CommandContext` struct, `try_issue_command()`, `try_issue_command_no_attack_reset()`, all `handle_*` functions (attack, dropoff, enter, pickup, attach, chopper, agent, recruit, guard), all ground-click handlers (move, patrol, attack, attack_move, attack_ground_location, reverse), `ground_pathfind_and_issue()`, and `right_click_move_command()`. Currently lines 244-952.

   - **rally.rs** (~150 code + ~100 test lines): `set_rally_point_click_system()` and `schedule_deliveries_click_system()`. Currently lines 953-1104.

   - **legacy_movement.rs** (~1,060 code + ~1,200 test lines): `unit_movement_system()`, `unit_rotation_system()`, and the 5 parameterized movement systems (`turn_rate_movement_system`, `fixed_turn_radius_movement_system`, `speed_turn_radius_movement_system`, `drag_movement_system`, `glider_movement_system`). Also the `WAYPOINT_ARRIVAL_THRESHOLD` constant. Currently lines 1105-2163.

   - **channel_locomotion.rs** (~400 code + ~400 test lines): `channel_turnrate_locomotion_system()`, `channel_fallback_locomotion_system()`, `channel_orientation_system()`, `crushing_system()`. Currently lines 2516-2915.

   - **occupancy.rs** (~100 code + ~200 test lines): `rebuild_occupancy_map()` and `collision_repath_system()`. Currently lines 2164-2261.

   - **indicators.rs** (~250 code + ~300 test lines): `emissive_from_color()`, `DesiredIndicator` struct, `command_indicator_sync_system()`, `air_unit_separation_system()`. Currently lines 2262-2515.

   - **utility.rs** (~210 code + ~200 test lines): `grid_position_sync_system()`, `unit_selection_display()`, `spawn_test_units()`. Currently lines 27-242.

3. Distribute the tests from the single `mod tests` block (lines 2916-6054) into `#[cfg(test)] mod tests { ... }` blocks within each submodule, placing each test with the code it exercises.

4. Delete the original `core.rs` file.

5. `src/game/units/systems/mod.rs` already has `pub mod core;` which Rust will resolve to `core/mod.rs` -- no change needed there.

### Import Handling

- Each submodule needs its own `use` imports (copy relevant imports from the top of the current core.rs)
- `mod.rs` must re-export everything public so external code is unaffected
- Tests use `use super::*;` -- update these to reference the correct module scope
- Internal cross-references between submodules (e.g., if rally.rs needs something from command_dispatch) should use `use super::command_dispatch::...` or be re-exported through mod.rs

### Verification

- `cargo check` must pass
- `cargo test --lib` must pass (all ~1,900+ existing tests)
- `cargo test --test qa` must pass (same pass/fail as before -- no regressions)
- No file in the new `core/` directory should exceed 2,300 lines
