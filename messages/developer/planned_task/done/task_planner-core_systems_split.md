# core_systems_split

## Metadata
- **From**: task_planner
- **To**: developer

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

   - **indicators.rs** (~250 code + ~300 test lines): `emissive_from_color()`, `DesiredIndicator` struct, `CachedIndicatorMaterials` struct, `command_indicator_sync_system()`, `air_unit_separation_system()`. Currently lines 2262-2515.

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

## Technical Context

### Source File to Split
- **`artifacts/developer/src/game/units/systems/core.rs`** (6,054 lines) -- the single file being decomposed

### Exact Code Section Boundaries (verified by line inspection)

| Submodule | Code Lines | Description |
|---|---|---|
| utility.rs | 27-242 | `spawn_test_units` (27-205), `grid_position_sync_system` (210-220), `unit_selection_display` (222-238) |
| command_dispatch.rs | 244-952 | `CommandContext` struct (244-260), extracted entity-click handlers (309-640), extracted ground-click handlers (641-900), `right_click_move_command` main system (262-308 + 901-952) |
| rally.rs | 953-1104 | `set_rally_point_click_system` (953-1043), `schedule_deliveries_click_system` (1044-1101) |
| legacy_movement.rs | 1105-2163 | `unit_movement_system` (1105-1215), `unit_rotation_system` (1217-1268), `turn_rate_movement_system` (1270-1433), `fixed_turn_radius_movement_system` (1435-1660), `speed_turn_radius_movement_system` (1662-1840), `drag_movement_system` (1842-2012), `glider_movement_system` (2014-2163) |
| occupancy.rs | 2164-2261 | `rebuild_occupancy_map` (2164-2230), `collision_repath_system` (2232-2261) |
| indicators.rs | 2262-2515 | `emissive_from_color` (2262-2267), `DesiredIndicator` struct (2270-2277), `CachedIndicatorMaterials` struct (2281-2285), `command_indicator_sync_system` (2287-2460), `air_unit_separation_system` (2462-2515) |
| channel_locomotion.rs | 2516-2914 | `channel_turnrate_locomotion_system` (2516-2700), `channel_fallback_locomotion_system` (2702-2812), `channel_orientation_system` (2814-2878), `crushing_system` (2880-2914) |

### Test Section Boundaries (within lines 2916-6054)

Tests are in a single `#[cfg(test)] mod tests` block starting at line 2916. They must be distributed to the submodule that owns the code they exercise:

| Target Submodule | Test Line Ranges | Key Test Names/Sections |
|---|---|---|
| utility.rs | 2931-2978 | `grid_position_sync_*` tests (4 tests) |
| indicators.rs | 3060-3462 | emissive tests (3062-3097), indicator sync tests (3099-3273), air separation tests (3294-3462) |
| occupancy.rs | 3485-3552, 3963-4062 | memory leak fix / repath tests, `rebuild_occupancy_map` tests |
| command_dispatch.rs | 2978-3059, 3553-3962, 4063-4310, 5390-6054 | move-clears-attack (3553-3688), cursor/UI (3689-3802), dropoff target (3803-3962), enter command (4063-4310), attack right-click (5390-5549), try_issue_command (5551-5708), handler dispatch (5709-6054) |
| channel_locomotion.rs | 4312-4578, 4942-5050 | channel consumer tests (4312-4578), crushing tests (4942-5050) |
| legacy_movement.rs | 4579-4941, 5051-5389 | FixedTurnRadius (4579-4741), SpeedTurnRadius (4742-4941), drag (5051-5217), glider (5218-5389) |
| rally.rs | (none found -- no dedicated rally tests in test block) |

### Shared Test Helpers

Lines 2922-2930 define `spawn_unit_at()` which is used by multiple test submodules. Place it in utility.rs tests and have other test modules reference it via `use super::utility::tests::spawn_unit_at` or duplicate it where needed. Other shared test helpers:
- `create_indicator_test_world()` (line 3100) -- indicators.rs only
- `create_separation_test_world()` (line 3294) -- indicators.rs only  
- `spawn_air_unit_at()` (line 3276) -- indicators.rs only
- `test_turnrate_params()` (line 4315) -- channel_locomotion.rs only
- `spawn_channel_turnrate_unit()` (line 4325) -- channel_locomotion.rs only
- `spawn_crusher()` / `spawn_crushable()` (lines ~4930-4964) -- channel_locomotion.rs only
- `create_dropoff_test_world()` (line 3806) -- command_dispatch.rs only

### Import Block (lines 1-18)

The full import block at the top of core.rs must be distributed. Each submodule only needs the imports its code actually uses. Key imports by submodule:

- **All submodules**: `use bevy::prelude::*;`, `use crate::types::*;`, `use crate::game::units::types::*;`
- **command_dispatch.rs**: Most imports -- UI types, combat types, game types (TunnelState, BarracksState, etc.), ArmoryState, AgentCarryState, faction helpers, world types (SDS, SCP), units utils (world_to_grid, can_enter_tunnel, issue_or_queue_command, etc.)
- **rally.rs**: UI types (CursorTarget, CursorOverUi, ObjectInterfaceState, StructureMenuState), game types (SupplyTowerState, RallyTarget, RallyPointMarker), faction helpers
- **legacy_movement.rs**: Movement param types (TurnRate, FixedTurnRadius, SpeedTurnRadius, Drag, Glider), combat types (Silhouette), OccupancyMap
- **channel_locomotion.rs**: Movement param types, combat types (Armor, Silhouette), LocomotionChannel, OrientationChannel, OccupancyMap
- **occupancy.rs**: ObjectInstance, StructureInstance, GridPosition, OccupancyMap, Silhouette, DomainEnum
- **indicators.rs**: CommandIndicator types, combat types (AttackState, AttackTarget), UI types (ObjectInterfaceState)
- **utility.rs**: GridPosition, ObjectInstance, Unit, Selected, spawn helpers

### Cross-Submodule References

These items are referenced across submodule boundaries and MUST be re-exported from mod.rs:

1. **`WAYPOINT_ARRIVAL_THRESHOLD`** (defined in legacy_movement.rs, used only there -- no cross-ref needed)
2. **`CommandContext`** (defined in command_dispatch.rs, only used within command_dispatch.rs -- no cross-ref)
3. **`emissive_from_color`** (private fn in indicators.rs, used only there -- no cross-ref)
4. **`DesiredIndicator`** (private struct in indicators.rs -- no cross-ref)
5. **`CachedIndicatorMaterials`** (pub(crate) in indicators.rs -- re-export if needed, but only used internally)

No cross-submodule function calls exist between the 7 code sections. Each section is self-contained. The mod.rs only needs to re-export the PUBLIC items so that `units/mod.rs` can reference `systems::core::rebuild_occupancy_map` etc.

### Public Items to Re-export from mod.rs

All systems registered in `units/mod.rs` (lines 17-91) via `systems::core::*` paths:
- `rebuild_occupancy_map`, `collision_repath_system` (from occupancy)
- `unit_movement_system`, `unit_rotation_system`, `turn_rate_movement_system`, `fixed_turn_radius_movement_system`, `speed_turn_radius_movement_system`, `drag_movement_system`, `glider_movement_system` (from legacy_movement)
- `channel_turnrate_locomotion_system`, `channel_fallback_locomotion_system`, `channel_orientation_system`, `crushing_system` (from channel_locomotion)
- `grid_position_sync_system`, `unit_selection_display`, `spawn_test_units` (from utility)
- `right_click_move_command`, `CommandContext` (from command_dispatch)
- `set_rally_point_click_system`, `schedule_deliveries_click_system` (from rally)
- `command_indicator_sync_system`, `air_unit_separation_system` (from indicators)

### Plugin Registration (no changes needed)

`artifacts/developer/src/game/units/mod.rs` references all systems via `systems::core::*` paths (lines 17-91). Since `pub mod core;` resolves to `core/mod.rs` and mod.rs re-exports everything, no path changes are needed. Verify by reading lines 15-92 of mod.rs.

### External References (only one outside units/)

`artifacts/developer/src/game/combat/systems/behaviors.rs` line 11 imports from `crate::game::combat::systems::core` (the COMBAT core, not units core). This is a different module and is NOT affected by this refactor.

### Test Infrastructure

- Tests use `use super::*;` which in the new structure will import from the submodule scope
- Test-only imports like `use bevy::ecs::system::RunSystemOnce;` and `use crate::game::combat::types::{AttackTarget, AttackPhase};` need to be added to each test module that uses them
- Some tests call functions from other submodules (e.g., command_dispatch tests may call `grid_position_sync_system`). These will work via the re-exports in mod.rs if tests `use super::*;` (which imports from the submodule, not mod.rs). Alternative: add `use crate::game::units::systems::core::*;` in test modules that need cross-submodule access.

### Verification Commands

```bash
cd artifacts/developer && cargo check 2>&1
cd artifacts/developer && cargo test --lib 2>&1
cd artifacts/developer && cargo test --test qa 2>&1
```

Ensure no file exceeds 2,300 lines. Largest expected: legacy_movement.rs at ~2,260 lines.

## Dependencies

None. This is a standalone structural refactoring with no behavioral changes and no dependency on other tasks. It should be done before any new feature work touches core.rs to avoid merge conflicts.
