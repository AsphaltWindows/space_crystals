# harden_assertions_and_cleanup

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-test_suite_cleanup.md

## Task

Harden 33 soft assertions across 11 integration test files, remove dead test code, fix deprecated API usage, and add proper ignore annotations.

### Soft Assertion Hardening (33 sites across 11 files)

Replace all `if let Some(component) = world.get::<T>(entity)` assertion guards with `let component = world.get::<T>(entity).expect("Entity should have T component");` or `.unwrap()`. This ensures tests fail loudly when a component is missing instead of silently passing.

File-by-file count:
- fix_units_moving_while_attacking.rs: 7 instances
- tunnel_object_interface_state.rs: 6 instances
- gdo_supply_tower_and_chopper.rs: 4 instances
- combat_behaviors.rs: 4 instances
- autonomous_targeting.rs: 3 instances
- construction_hp_rule.rs: 2 instances
- tunnel_area_and_construction_rules.rs: 2 instances
- movement_behaviors.rs: 2 instances
- attack_phases.rs: 1 instance
- agent_resource_gathering_commands_and_behaviors.rs: 1 instance
- tunnel_expansions_and_starting_condition.rs: 1 instance

After hardening, run the full integration suite. If any newly-hardened assertions fail, that reveals a real bug -- fix the test setup or report the issue (do not revert to soft assertions).

### Dead Code Removal

1. Remove `create_dropoff_test_world` function at src/game/units/systems/core.rs:3806 (unused test helper)
2. Remove `create_right_click_entity_world` function at src/game/units/systems/core.rs:4080 (unused test helper)
3. Remove unused imports across integration test files (check for compiler warnings and fix)
4. Replace deprecated `world.iter_entities()` with appropriate query in tests/qa/fix_memory_leak_oom_freeze.rs:14

### Ignore Annotations

Add proper reason strings to all ignored tests:
- The 6 escape-key tests: change `#[ignore]` to `#[ignore = "Escape key not processed in headless mode"]` if not already annotated
- step_8_attack_ground_command: add `#[ignore = "reason"]` with appropriate reason
- step_9_reverse_command: add `#[ignore = "reason"]` with appropriate reason

### Files
- tests/qa/ -- 11 files for soft assertion hardening (listed above)
- tests/qa/fix_memory_leak_oom_freeze.rs -- deprecated API
- src/game/units/systems/core.rs -- remove 2 dead test helpers
- tests/qa/basic_combat_unit_interface_state.rs -- ignore annotations (if unannotated ignores exist here)
- tests/qa/tunnel_object_interface_state.rs -- ignore annotations (if escape-key ignores exist here)
- tests/qa/agent_object_interface_state.rs -- ignore annotations (if escape-key ignores exist here)

### Verification
- cargo test --test qa -- full suite passes with no new failures
- cargo test --lib -- no regressions, no dead code warnings from removed helpers
- cargo test --test qa 2>&1 | grep warning -- warning count should decrease
