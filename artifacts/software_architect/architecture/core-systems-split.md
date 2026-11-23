# Unit Systems Core Split

## Date: 2026-03-24
## Status: Architecture request sent
## Priority: 2 (parallel with command panel split)
## Parent: codebase-refactoring-priorities.md

## Problem Statement

`src/game/units/systems/core.rs` is 6,054 lines and contains at least 6 unrelated responsibility groups: command dispatch, legacy movement systems, channel-based locomotion, occupancy/collision, visual indicators, and rally/delivery click systems. The P1 refactor (right-click-command-dispatch) extracted handler functions but they remained in the same file, so the total size did not decrease materially.

## Current Structure Analysis

### 1. Test Scaffolding (lines 27-209, ~180 lines)
- `spawn_test_units()` - spawns test units on the map

### 2. Grid/Selection Utilities (lines 210-242, ~30 lines)
- `grid_position_sync_system()` - syncs Transform to GridPosition
- `unit_selection_display()` - updates selection visual indicators

### 3. Command Dispatch (lines 244-952, ~710 lines)
- `CommandContext` struct
- `try_issue_command()`, `try_issue_command_no_attack_reset()`
- All `handle_*` extracted handler functions (attack, dropoff, enter, pickup, attach, chopper, agent, recruit, guard right-click handlers)
- All ground-click handler functions (move, patrol, attack, attack_move, attack_ground_location, reverse)
- `ground_pathfind_and_issue()` helper
- `right_click_move_command()` - the main dispatch system

### 4. Rally/Delivery Click Systems (lines 953-1104, ~150 lines)
- `set_rally_point_click_system()`
- `schedule_deliveries_click_system()`

### 5. Legacy Movement Pipeline (lines 1105-2163, ~1,060 lines)
- `unit_movement_system()` - fallback movement for non-parameterized units
- `unit_rotation_system()` - fallback rotation
- `turn_rate_movement_system()` - TurnRate movement (~165 lines)
- `fixed_turn_radius_movement_system()` - FixedTurnRadius movement (~225 lines)
- `speed_turn_radius_movement_system()` - SpeedTurnRadius movement (~180 lines)
- `drag_movement_system()` - Drag movement (~170 lines)
- `glider_movement_system()` - Glider movement (~180 lines)

### 6. Occupancy and Collision (lines 2164-2261, ~100 lines)
- `rebuild_occupancy_map()`
- `collision_repath_system()`

### 7. Visual/Indicator Systems (lines 2262-2515, ~250 lines)
- `emissive_from_color()` utility
- `command_indicator_sync_system()` (~160 lines)
- `air_unit_separation_system()` (~55 lines)

### 8. Channel-Based Locomotion (lines 2516-2915, ~400 lines)
- `channel_turnrate_locomotion_system()` (~185 lines)
- `channel_fallback_locomotion_system()` (~110 lines)
- `channel_orientation_system()` (~65 lines)
- `crushing_system()` (~40 lines)

### 9. Tests (lines 2916-6054, ~3,140 lines)

## Proposed Refactoring

Convert `systems/core.rs` into a `systems/core/` directory module, splitting by responsibility:

### New Directory Structure

```
src/game/units/systems/core/
    mod.rs              -- re-exports all public systems
    command_dispatch.rs -- CommandContext, try_issue_command, right_click_move_command, all handle_* functions
    rally.rs            -- set_rally_point_click_system, schedule_deliveries_click_system
    legacy_movement.rs  -- unit_movement_system, unit_rotation_system, and the 5 parameterized movement systems
    channel_locomotion.rs -- channel_turnrate_locomotion_system, channel_fallback_locomotion_system, channel_orientation_system, crushing_system
    occupancy.rs        -- rebuild_occupancy_map, collision_repath_system
    indicators.rs       -- emissive_from_color, command_indicator_sync_system, air_unit_separation_system
    utility.rs          -- grid_position_sync_system, unit_selection_display, spawn_test_units
```

### Key Decisions

1. **Command dispatch stays together**: The P1 refactor already extracted handler functions from `right_click_move_command`. These extracted handlers should stay in the same file as the dispatcher since they are tightly coupled (share `CommandContext`, shared patterns). This file will be ~710 lines, which is reasonable.

2. **Movement systems split by pipeline**: Legacy movement (MoveTarget/Path-driven) goes in `legacy_movement.rs`. Channel-based locomotion (LocomotionChannel/OrientationChannel-driven) goes in `channel_locomotion.rs`. This makes the dual-pipeline architecture explicit and prepares for the eventual Priority 4 unification.

3. **Tests distributed to submodules**: Each submodule gets its relevant tests. The ~3,140 lines of tests split roughly:
   - command_dispatch.rs tests: ~800 lines (command issuance, right-click scenarios)
   - legacy_movement.rs tests: ~1,200 lines (movement systems, turn rates)
   - channel_locomotion.rs tests: ~400 lines
   - occupancy.rs tests: ~200 lines
   - indicators.rs tests: ~300 lines
   - utility.rs tests: ~200 lines (grid_position_sync tests)

4. **No system signature changes**: All systems keep their exact signatures. The `mod.rs` re-exports everything that `units/mod.rs` and `units/plugin.rs` reference.

## Scope

### Files Created
- `src/game/units/systems/core/mod.rs`
- `src/game/units/systems/core/command_dispatch.rs`
- `src/game/units/systems/core/rally.rs`
- `src/game/units/systems/core/legacy_movement.rs`
- `src/game/units/systems/core/channel_locomotion.rs`
- `src/game/units/systems/core/occupancy.rs`
- `src/game/units/systems/core/indicators.rs`
- `src/game/units/systems/core/utility.rs`

### Files Deleted
- `src/game/units/systems/core.rs` (replaced by directory module)

### Files Modified
- `src/game/units/systems/mod.rs` -- `pub mod core` stays the same (Rust resolves to `core/mod.rs`)
- Any files that import from `systems::core` -- import paths unchanged due to re-exports

### Expected Line Counts (approximate, including tests)
- command_dispatch.rs: ~710 + ~800 tests = ~1,510
- legacy_movement.rs: ~1,060 + ~1,200 tests = ~2,260
- channel_locomotion.rs: ~400 + ~400 tests = ~800
- occupancy.rs: ~100 + ~200 tests = ~300
- indicators.rs: ~250 + ~300 tests = ~550
- rally.rs: ~150 + ~100 tests = ~250
- utility.rs: ~210 + ~200 tests = ~410
- mod.rs: ~40 lines

Note: `legacy_movement.rs` at ~2,260 lines is the largest. This is acceptable because it contains 7 self-contained movement system implementations that share no logic between them. If a future refactor unifies the dual movement pipeline (Priority 4), this file would shrink significantly. Splitting it further now (one file per movement type) would create 7 tiny files with identical structure, which is worse for navigability.

## Constraints

- Behavioral equivalence: identical behavior before and after
- All existing unit tests and integration tests must pass
- System registration paths must work via re-exports
- No new Bevy systems

## Success Metrics

- `core.rs` replaced by 8 files, with clear single-responsibility naming
- Command dispatch, movement, locomotion, and visual systems are in separate files
- The dual movement pipeline (legacy vs channel) is explicitly separated, documenting the architectural boundary
- Adding a new movement type requires editing only `legacy_movement.rs` or `channel_locomotion.rs`
