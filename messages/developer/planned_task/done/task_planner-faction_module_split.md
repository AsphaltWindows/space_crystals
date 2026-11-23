# faction_module_split

## Metadata
- **From**: task_planner
- **To**: developer

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

## Technical Context

### Source file

`artifacts/developer/src/game/world/faction.rs` — 4,387 lines total (2,585 lines of code, 1,801 lines of tests starting at line 2586).

### Current imports block (lines 1-16)

The file has a large shared imports block at the top. Each new submodule will need its own subset of these imports. The full block is:
```rust
use bevy::prelude::*;
use crate::types::{Owner, ObjectEnum, MainCamera, GridPosition, LocalPlayer, InvisibleEntity, Selected, SelectionBounds, SelectedFaction, FactionEnum};
use crate::game::types::*;
use crate::game::utils::{
    spawn_deployment_center, spawn_power_plant, spawn_barracks,
    spawn_extraction_facility, spawn_extraction_plate, spawn_peacekeeper,
    spawn_tunnel, spawn_headquarters, spawn_supply_tower, spawn_supply_chopper,
    spawn_recruitment_center, spawn_cults_soldier, spawn_cults_gunner,
    spawn_cults_storage_under_construction,
};
use super::types::{GdoBuildArea, SpaceCrystalPatch, Tile, TilePreset, FogOfWarMap, MapStartingPositions};
use super::utils::{expand_build_area, world_to_grid, grid_to_world, can_place_building, rotated_building_size};
use crate::ui::types::{ObjectInterfaceState, StructureMenuState, AgentMenuState, CultsRecruitMenuState, CommandPanelTarget, PlacementGhost, PlacementState, CursorOverUi, BuildAreaOverlay, ArmoryEjectionQueue};
use crate::game::units::types::state::UnitCommand;
use crate::game::units::types::state::behavior::BuildingTunnelBehavior;
```
Each submodule should only import what it actually uses. Note that `super::types` and `super::utils` paths change to `super::super::types` (or use `crate::game::world::types`) since the submodules are one level deeper.

### Cross-submodule function dependencies (CRITICAL)

These private/internal functions are called across what will become submodule boundaries:

1. **`get_power_ratio_for_owner`** (line 1158, currently private) — Called by:
   - `dc_construction_tick_system` (line 329) → construction.rs
   - `barracks_production_tick_system` (line 375) → production.rs
   - `extraction_plate_mining_system` (line 634) → resources.rs
   - `ef_construction_tick_system` (line 953) → construction.rs
   - `supply_tower_production_tick_system` (line 2145) → production.rs

   **Action**: Move to `power.rs`, make `pub(super)`, import in construction.rs, production.rs, and resources.rs via `use super::power::get_power_ratio_for_owner;`

2. **`issue_rally_command`** (line 425, currently private) — Called by:
   - `barracks_production_tick_system` (line 404) → production.rs
   - `headquarters_production_tick_system` (line 582) → production.rs
   - `armory_training_tick_system` (line 2542) → production.rs

   **Action**: All callers are within production.rs. Keep it private in production.rs. No cross-module concern.

3. **`spawn_faction_and_player`** (line 50, currently private) — Only called by `setup_player_resources` (same file). Keep private in setup.rs.

4. **`compute_rally_target_from_click`** (line 830), **`has_structure_overlap`** (line 1829), **`build_area_mesh`** (line 1856), **`tunnel_area_mesh`** (line 1906), **`armory_exit_side_position`** (line 2475) — All callers are within the same target submodule. No cross-module concern.

### External references to faction module (CRITICAL)

Two files import from `crate::game::world::faction`:

1. **`artifacts/developer/src/game/units/systems/core.rs` line 16**:
   ```rust
   use crate::game::world::faction::{spawn_or_update_rally_marker, despawn_rally_marker_for};
   ```
   These functions must be re-exported from `faction/mod.rs` so this import path continues to work.

2. **`artifacts/developer/src/game/combat/mod.rs` line 43**:
   ```rust
   crate::game::world::faction::cults_construction_cancel_system.before(systems::remove_dead_entities_system),
   ```
   This system is registered in `CombatPlugin`, NOT in `FactionPlugin`. It must remain accessible via `faction::cults_construction_cancel_system`. Re-export it from `faction/mod.rs`.

3. **`artifacts/developer/src/game/world/mod.rs`**: All `faction::*` references (lines 72-128) continue to work through `faction/mod.rs` re-exports. The `pub mod faction;` declaration on line 8 automatically resolves to `faction/mod.rs` when the directory exists.

### mod.rs re-export structure

The `faction/mod.rs` file should declare all submodules and re-export every `pub` function. Pattern:
```rust
mod setup;
mod power;
mod production;
mod construction;
mod resources;
mod rally;
mod placement;
mod ejection;

pub use setup::*;
pub use power::compute_power_grid;  // only re-export pub functions, not pub(super) helpers
pub use production::*;
pub use construction::*;
pub use resources::*;
pub use rally::*;
pub use placement::*;
pub use ejection::*;
```

Alternatively, explicitly list each re-exported function to be precise. Either approach works as long as every function currently accessible via `faction::function_name` remains accessible.

### System registration locations

All system registrations are in two places and must NOT change:

1. **`artifacts/developer/src/game/world/mod.rs`** (FactionPlugin, lines 60-131) — registers 30+ faction systems across `OnEnter`, `Update`, and `FixedUpdate`. All use `faction::system_name` paths.

2. **`artifacts/developer/src/game/combat/mod.rs`** (CombatPlugin, line 43) — registers `cults_construction_cancel_system` with `.before()` ordering constraint.

### Test-to-submodule mapping

The single `#[cfg(test)] mod tests` block (lines 2586-4387) must be split. Here is the mapping based on what each test exercises:

- **setup.rs tests**: `setup_player_resources_gdo_selected_spawns_gdo_and_syndicate` (3033), `setup_player_resources_cults_selected_spawns_cults_and_gdo` (3065), `setup_player_resources_colonists_selected_spawns_colonists_and_gdo` (3092), `setup_player_resources_syndicate_selected_spawns_syndicate_and_gdo` (3119), `cults_player_resources_default_values` (3146), `colonists_player_resources_default_values` (3154), `test_center_camera_on_start_*` (3209-3382)
- **rally.rs tests**: `headquarters_default_has_no_rally_point` (2593), `headquarters_rally_point_location_means_eject` (2599), `headquarters_no_rally_means_tunnel_network` (2610), `headquarters_rally_target_parent_tunnel_clears` (2622), `headquarters_rally_target_other_entity_does_not_clear` (2643), `rally_point_marker_stores_owner` (2664)
- **production.rs tests**: `headquarters_production_eject_decision_with_parent_tunnel_rally` (2671), `headquarters_production_eject_decision_with_enemy_rally` (2686), `supply_tower_placement_links_chopper` (2990), `rc_production_*` tests (3638-3733), `rc_recruit_*` tests (3747-3810), `armory_training_tick_*` tests (4087-4158), `armory_state_training_*` tests (4269-4295), `rally_attack_enemy_inserts_move_target_and_path` (4299). Helper functions: `spawn_production_rc` (3610), `setup_production_app` (3627).
- **resources.rs tests**: `depleted_patch_*` tests (2745-2828), `extraction_plate_mining_*` tests (2829-2989), `extraction_plate_power_grid_integration` (3164)
- **placement.rs tests**: `ghost_material_should_be_double_sided_and_additive` (2706), `flip_scale_axes_are_correct` (2723)
- **ejection.rs tests**: `tile_claim_map_*` (3406-3429), `rc_all_recruitable_terrain_*` (3432), `rc_half_recruitable_terrain_*` (3460), `rc_overlapping_areas_*` (3492), `rc_destruction_frees_tiles_*` (3537), `rc_near_map_edge_*` (3580), `aggregation_*` tests (3792-3865), `armory_exit_side_position_*` (4159), `armory_eject_*` tests (4175-4268). Helper functions: `spawn_test_tile` (3384), `spawn_test_rc` (3394).
- **construction.rs tests**: `cults_construction_*` tests (3898-4069), `generic_construction_hp_tick_skips_cults_buildings` (4051). Helper functions: `setup_construction_app` (3866), `spawn_cults_building_under_construction` (3875), `spawn_hidden_recruit` (3886).

### Import path changes in submodules

Since submodules are nested one level deeper than the original `faction.rs`:
- `super::types::*` becomes `super::super::types::*` (or preferably `crate::game::world::types::*`)
- `super::utils::*` becomes `super::super::utils::*` (or preferably `crate::game::world::utils::*`)
- `use super::*;` in test modules now refers to the submodule's scope, not the old faction.rs scope. Tests may need additional explicit imports for types from `crate::game::types::*` that were previously pulled in via the faction.rs wildcard import.

### Verification commands

After completing the split:
```bash
cargo check                    # Compilation check
cargo test --lib               # All library tests
cargo test --test qa           # Integration tests
```

## Dependencies

None. This is a standalone refactoring task with no dependencies on other planned tasks. The `pub mod faction;` declaration in `world/mod.rs` automatically resolves to `faction/mod.rs` when the directory exists, so no changes to mod.rs module declarations are needed.
