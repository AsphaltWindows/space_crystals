# fix_auto_targeting_and_stop_tests

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-test_suite_cleanup.md

## Task

Fix 5 failing integration tests: 3 auto-targeting tests (Category A) and 2 stop command tests (Category B).

### Category A: Auto-targeting not engaging (3 tests)

**Root cause**: base_auto_target_system requires targets to pass is_valid_target(), which checks VisibilityStateEnum. The test harness spawn_unit_at_grid (src/shared/testing/harness.rs:47) does not set VisibilityStateEnum::Visible on enemy units, so targets are filtered out.

**Fix**: After spawning enemy units in tests, set VisibilityStateEnum::Visible on them. The best approach is to fix spawn_unit_at_grid itself to always insert VisibilityStateEnum::Visible on spawned units (or at least for the enemy player's units from the spawning player's perspective). Alternatively, fix the individual tests to manually insert the component.

Check if the fog of war map also needs to be set -- the test harness has set_all_tiles_visible() (harness.rs:385) which sets all tiles visible on the fog map. The tests may already call this, but the per-entity VisibilityStateEnum might also need to be set.

**Tests to fix**:
- autonomous_targeting::step_1_auto_targeting_engages_enemy
- autonomous_targeting::step_6_infantry_auto_targeting
- combat_behaviors::step_10_hold_position_engages_nearby

### Category B: Stop command behavior mismatch (2 tests)

**Test 1**: basic_combat_unit_interface_state::step_3_stop_immediate_command
- Expects UnitCommand::Stop after issuing Stop, but gets Idle
- Root cause: behavior_completion_system runs during step(). Unit was stationary (never moved), so Stop completes instantly -> Idle
- Fix: Change test expectation to assert Idle (correct behavior for Stop on stationary unit)

**Test 2**: movement_behaviors::step_8_stop_command
- Issues Move (5 frames), then Stop (10 frames). Velocity still ~1.0 after 10 frames
- Root cause: deceleration rate is too slow for 10 frames, or system ordering issue
- Fix: Either increase frame count (try 30-50 frames) or investigate deceleration rate and adjust test expectation

### Files
- src/shared/testing/harness.rs -- may need to add VisibilityStateEnum::Visible to spawn_unit_at_grid
- tests/qa/autonomous_targeting.rs -- verify 2 tests pass
- tests/qa/combat_behaviors.rs -- verify step_10 passes
- tests/qa/basic_combat_unit_interface_state.rs -- fix step_3 expectation
- tests/qa/movement_behaviors.rs -- fix step_8 timing or expectation

### Verification
- cargo test --test qa autonomous_targeting -- all tests pass
- cargo test --test qa combat_behaviors -- step_10 passes
- cargo test --test qa basic_combat_unit_interface_state -- step_3 passes
- cargo test --test qa movement_behaviors -- step_8 passes
