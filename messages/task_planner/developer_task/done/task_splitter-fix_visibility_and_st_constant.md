# fix_visibility_and_st_constant

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-core_systems_split_r2.md

## Task

Fix 4 failing QA integration tests: 3 auto-targeting tests that fail because spawned enemies lack VisibilityStateEnum::Visible, and 1 constant mismatch test.

### Category A: Auto-targeting not engaging (3 tests)

**Root cause**: base_auto_target_system (combat/systems/core.rs:474) queries potential_targets with VisibilityStateEnum. The is_valid_target() function (combat/utils.rs:278) requires target_visibility == VisibilityStateEnum::Visible. But spawn_peacekeeper (game/utils.rs:399) does NOT insert VisibilityStateEnum on spawned units. So enemy units in tests have no VisibilityStateEnum component and fail the filter.

**Fix**: Add VisibilityStateEnum::Visible to the spawn bundle in ALL unit spawn functions that are used in tests:
- spawn_peacekeeper (game/utils.rs ~line 451)
- spawn_syndicate_agent (game/utils.rs)
- spawn_syndicate_guard (game/utils.rs)
- spawn_supply_chopper (game/utils.rs)

Search for all spawn functions that create entities with ObjectInstance and ensure they all include VisibilityStateEnum::Visible. This is correct for gameplay too — newly spawned units should be Visible and the fog system will update visibility state each frame based on FogOfWarMap.

Alternatively, if modifying spawn functions is too broad, fix the test harness spawn_unit_at_grid to insert VisibilityStateEnum::Visible after spawning. But fixing spawn functions is the cleaner approach since it matches gameplay behavior.

**Tests to fix**:
- autonomous_targeting::step_1_auto_targeting_engages_enemy
- autonomous_targeting::step_6_infantry_auto_targeting  
- combat_behaviors::step_10_hold_position_engages_nearby

### Category B: ST_BUILD_FRAMES constant (1 test)

**Root cause**: ST_BUILD_FRAMES in src/game/types/structures.rs:433 is 160 but should be 240 (15 seconds at 16 FPS). The test expects 240. DeploymentCenterState::construction_cost already returns 240 for SupplyTower.

**Fix**:
1. Change ST_BUILD_FRAMES from 160 to 240 in structures.rs:433
2. Update the unit test at structures.rs:2055 that asserts ST_BUILD_FRAMES == 160 to assert 240
3. Search for any other references to ST_BUILD_FRAMES and verify consistency

**Test to fix**: gdo_supply_tower_and_chopper::supply_tower_constants

### Verification

All 4 tests must pass:
- cargo test --test qa autonomous_targeting::step_1
- cargo test --test qa autonomous_targeting::step_6
- cargo test --test qa combat_behaviors::step_10
- cargo test --test qa gdo_supply_tower_and_chopper::supply_tower_constants

Also verify no regressions:
- cargo test --lib
- cargo test --test qa
