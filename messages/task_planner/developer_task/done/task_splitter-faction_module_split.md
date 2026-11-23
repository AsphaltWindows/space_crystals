# faction_module_split

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-faction_split.md

## Task

Split the monolithic `src/game/world/faction.rs` (4,387 lines) into a `faction/` directory module with 9 files following the architecture document at `artifacts/software_architect/architecture/faction-split.md`.

### What to do

1. Create `src/game/world/faction/` directory
2. Create the following submodules by moving code from `faction.rs`:
   - **mod.rs** (~50 lines): Module declarations and `pub use` re-exports for ALL public functions currently in faction.rs, especially `spawn_or_update_rally_marker` and `despawn_rally_marker_for` (imported by `units/systems/core.rs`)
   - **setup.rs** (~465 lines with tests): `setup_player_resources`, `spawn_faction_and_player`, all `setup_*_game_start` functions, `center_camera_on_start`, `setup_enemy_test_units`
   - **power.rs** (~100 lines with tests): `compute_power_grid`, `get_power_ratio_for_owner`
   - **production.rs** (~850 lines with tests): All 5 production tick systems (`barracks_production_tick_system`, `headquarters_production_tick_system`, `supply_tower_production_tick_system`, `recruitment_center_production_system`, `armory_training_tick_system`) plus `issue_rally_command` helper
   - **construction.rs** (~670 lines with tests): `dc_construction_tick_system`, `ef_construction_tick_system`, `construction_hp_tick_system`, `tunnel_construction_tick_system`, `cults_construction_tick_system`, `cults_construction_cancel_system`, `debug_construction_input`
   - **resources.rs** (~190 lines with tests): `extraction_plate_mining_system`, `depleted_patch_despawn_system`, `display_resources_system`
   - **rally.rs** (~565 lines with tests): `rally_target_cleanup_system`, `production_rally_point_system`, `compute_rally_target_from_click`, `spawn_or_update_rally_marker`, `despawn_rally_marker_for`
   - **placement.rs** (~1030 lines with tests): `manage_placement_ghost`, `update_placement_ghost`, `placement_click_system`, `manage_build_area_overlay`, `has_structure_overlap`, `build_area_mesh`, `tunnel_area_mesh`
   - **ejection.rs** (~515 lines with tests): `ejection_tick_system`, `armory_eject_tick_system`, `armory_exit_side_position`, `recruitment_tile_claiming_system`, `cults_unit_control_aggregation_system`
3. Delete the original `faction.rs`
4. Ensure `mod.rs` re-exports all public items so that:
   - `FactionPlugin::build()` in `world/mod.rs` continues to reference systems as `faction::system_name`
   - `core.rs` (or `core/command_dispatch.rs` if the core split has landed) can still import `spawn_or_update_rally_marker` and `despawn_rally_marker_for` from `game::world::faction`
5. Move each `#[cfg(test)] mod tests` block to the submodule containing the code it tests, updating `use` paths as needed

### Key constraints

- NO behavioral changes -- this is a pure file reorganization
- All system signatures stay the same
- All `pub` functions remain accessible via the same `faction::` path (through re-exports)
- No file in the new directory should exceed 1,200 lines
- All existing unit tests (~1,800 lines) must pass with only import path changes
- `cargo check`, `cargo test --lib`, and `cargo test --test qa` must all pass

### Architecture reference

Read `artifacts/software_architect/architecture/faction-split.md` for the full analysis including line ranges in the current file, expected sizes, and rationale for each grouping.
