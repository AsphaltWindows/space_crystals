# behaviors_split

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-behaviors_split.md

## Task

Split the monolithic `src/game/units/systems/behaviors.rs` (4,481 lines) into a `behaviors/` directory module with 6 files. This is a pure structural refactoring with no behavioral changes.

### Architecture Document

Read `artifacts/software_architect/architecture/behaviors-split.md` for the full plan.

### Summary of Changes

1. **Create** `src/game/units/systems/behaviors/` directory with these files:
   - `mod.rs` (~30 lines) -- re-exports all public systems and shared constants
   - `movement.rs` (~1,265 lines incl. tests) -- moving_to_location_system, moving_to_object_system, reversing_to_location_system, stopping_behavior_system, behavior_completion_system, plus helper functions (update_path_index, set_glider_circling, extract_path_data, has_path_deviation)
   - `tunnel_interaction.rs` (~625 lines incl. tests) -- enter_command_dispatch_system, entering_tunnel_behavior_system, enter_armory_dispatch_system, entering_armory_behavior_system
   - `construction.rs` (~750 lines incl. tests) -- building_behavior_system, building_tunnel_behavior_system, cults_recruit_enter_construction_system
   - `resource_gathering.rs` (~1,030 lines incl. tests) -- gathering_resource_behavior_system, dropping_off_resources_behavior_system, plus shared helpers (tunnel_side_world_position, find_nearest_own_tunnel, drop_off_side_for_carry) with pub(super) visibility
   - `supply_chopper.rs` (~710 lines incl. tests) -- all supply_chopper_* systems and supply_chopper_repair_system

2. **Delete** `src/game/units/systems/behaviors.rs` (replaced by directory module)

3. **No changes to** `src/game/units/systems/mod.rs` -- `pub mod behaviors` resolves to the new directory module automatically

### Key Requirements

- All existing unit tests (~2,810 lines) must pass in their new submodule locations with only import path updates
- Shared helpers used across submodules must use `pub(super)` visibility
- Constants like WAYPOINT_REACHED_THRESHOLD must remain accessible to all consuming submodules
- No file should exceed 1,300 lines
- System registration paths must work via re-exports from mod.rs
- `cargo check`, `cargo test --lib`, and `cargo test --test qa` must all pass
