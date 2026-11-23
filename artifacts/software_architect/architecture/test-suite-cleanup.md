# Test Suite Cleanup

## Problem Statement

The project has 1,907 unit tests and 326 integration tests (309 passing, 9 failing, 8 ignored). A review of the test suite reveals several categories of issues that reduce test reliability and increase maintenance burden.

## Findings

### 1. Failing Integration Tests (9 tests, 5 categories)

**Category A: Stop command behavior mismatch (2 tests)**
- `basic_combat_unit_interface_state::step_3_stop_immediate_command` -- expects `UnitCommand::Stop` after issuing Stop, but gets `Some(Idle)`. Root cause: `behavior_completion_system` runs during `step()`. Since the unit was stationary (just spawned, never moved), Stop completes instantly and transitions to Idle. The test expectation is wrong -- it should assert Idle, which means Stop was processed correctly.
- `movement_behaviors::step_8_stop_command` -- issues Move (5 frames), then Stop (10 frames). Velocity is still ~1.0 after those 10 frames. Root cause: either the deceleration rate is too slow for 10 frames, or there is a system ordering issue where locomotion updates run after stop behavior sets channels. Fix: increase frame count or investigate deceleration rate.

**Category B: Auto-targeting not engaging (3 tests)**
- `autonomous_targeting::step_1_auto_targeting_engages_enemy` -- spawns player unit at grid (20,20) and enemy at (21,20) with 30 frames. No auto-engagement occurs (`phase: None, target: None`).
- `autonomous_targeting::step_6_infantry_auto_targeting` -- same pattern.
- `combat_behaviors::step_10_hold_position_engages_nearby` -- similar: HoldPosition unit near enemy doesn't engage.

Root cause: `base_auto_target_system` requires target entities to pass `is_valid_target()`, which checks `VisibilityStateEnum`. The test harness `spawn_unit_at_grid` does not set `VisibilityStateEnum::Visible` on enemy units, so targets are filtered out by the visibility check. Fix: ensure enemy units spawned in tests have `VisibilityStateEnum::Visible`.

**Category C: Stale hardcoded values (1 test)**
- `gdo_supply_tower_and_chopper::dc_construction_cost_supply_tower` -- expects Supply Tower build_frames = 160, but the actual `DeploymentCenterState::construction_cost` returns 240. The constant `ST_BUILD_FRAMES` is 160, but the method uses 240. This is a discrepancy between the constant and the method, and the integration test uses the stale value. The unit test at `structures.rs:2151` correctly asserts 240.

**Category D: Interface state not updating -- TestHarness::set_selection() bug (2 tests)**
- `automated_qa_ui_state_queries::step_1_get_interface_state_barracks` -- selects a Barracks but interface state stays `Default` instead of `StructureMenu(BarracksMenu)`.
- `automated_qa_ui_state_queries::step_3_get_interface_state_dc_build_menu` -- same category.

Root cause: `TestHarness::set_selection()` only adds the `Selected` ECS component to entities -- it does NOT update the `Selection` resource's `groups` field. When `update_command_panel_state` runs during `test_app.step()`, it reads `selection.groups` to determine `active_type()`. Since groups is empty, `active_type()` returns `None`, the system resets to `Default`. Fix: `set_selection()` must also populate the `Selection` resource groups to match the selected entities.

**Category E: Production ejection logic (1 test)**
- `tunnel_expansions_and_starting_condition::hq_production_ejects_agent_without_rally_point` -- agent should eject to surface but ends up in tunnel network.

### 2. Soft Assertions (33 instances across 10 files)

Many integration tests use `if let Some(component) = world.get::<T>(entity)` to guard their assertions. If the component does not exist on the entity, the test silently passes without asserting anything. This is a significant test quality issue -- it means a test can appear green when the component it's testing was never spawned.

Worst offenders:
- `fix_units_moving_while_attacking.rs`: 7 instances
- `tunnel_object_interface_state.rs`: 6 instances
- `gdo_supply_tower_and_chopper.rs`: 4 instances
- `combat_behaviors.rs`: 4 instances
- `autonomous_targeting.rs`: 3 instances

**Fix**: Replace `if let Some(x) = world.get::<T>(entity)` with `let x = world.get::<T>(entity).expect("Entity should have T component");` or use `unwrap()` to make the test fail loudly when the component is missing.

### 3. Ignored Tests (8 tests)

Six tests are ignored because "Escape key not processed in headless mode." Two more in `basic_combat_unit_interface_state` are ignored without clear reason annotations. These tests cover escape-key navigation in tunnel and agent interface states.

**Fix**: The ignored tests with escape-key issues are a known limitation of headless testing (no HudPlugin). These should either:
- Be annotated with clear `#[ignore = "reason"]` messages (not just comments)
- Have tracking in this doc so they get addressed when the test infrastructure supports hotkey simulation

The two unannotated `#[ignore]` tests (`step_8_attack_ground_command`, `step_9_reverse_command`) should have reason annotations added.

### 4. Dead Code in Tests

- `create_dropoff_test_world` (core.rs:3806) -- unused test helper function
- `create_right_click_entity_world` (core.rs:4080) -- unused test helper function
- Multiple `#[allow(unused_imports)]` across integration test files (22 instances)
- 1 deprecated API usage: `world.iter_entities()` in `fix_memory_leak_oom_freeze.rs:14`

### 5. Constant-Mirroring Tests

Many tests in `structures.rs` simply assert that a constant equals a specific literal value (e.g., `assert_eq!(EP_POWER, -3)`). These tests:
- Do not test behavior or logic
- Break whenever a constant is intentionally changed (creating noise, not catching bugs)
- Add maintenance burden without meaningful coverage

However, some constant tests DO serve a purpose: tests like `dc_generates_enough_for_startup` verify relationships between constants (game balance invariants). These are valuable.

**Recommendation**: Keep relationship/invariant tests. Remove or convert pure constant-mirroring tests into documentation comments. This is low priority.

### 6. ST_BUILD_FRAMES Constant vs. construction_cost Discrepancy

The constant `ST_BUILD_FRAMES` in structures.rs is 160, but `DeploymentCenterState::construction_cost(&ObjectEnum::SupplyTower)` returns `build_frames: 240`. One of these is wrong. The unit test at line 2055 asserts `ST_BUILD_FRAMES == 160` (passes) and the unit test at line 2151 asserts `cost.build_frames == 240` (passes). Both pass but they disagree on what the Supply Tower build frames should be. This means `ST_BUILD_FRAMES` may be used elsewhere in the code with the wrong value.

## Proposed Cleanup Plan

### Phase 1: Fix failing integration tests (HIGH PRIORITY)

1. Fix the Supply Tower build_frames discrepancy -- determine correct value, update either the constant or the construction_cost method, update all tests
2. Fix stop-command tests -- either adjust timing expectations or fix the assertion logic (Stop->Idle transition)
3. Investigate auto-targeting test failures -- determine if spawn distance, frame count, or system regression is the cause
4. Fix or properly ignore the interface-state tests if they require HUD systems unavailable in headless mode
5. Fix the HQ production ejection test

### Phase 2: Harden soft assertions (MEDIUM PRIORITY)

Replace all `if let Some` guarded assertions in integration tests with `expect()` or `unwrap()` calls. This is a mechanical change across 33 sites in 10 files.

### Phase 3: Clean up dead code and warnings (LOW PRIORITY)

1. Remove dead test helper functions (`create_dropoff_test_world`, `create_right_click_entity_world`)
2. Remove unused imports
3. Replace deprecated `iter_entities()` with `world.query::<EntityRef>()`
4. Add proper `#[ignore = "reason"]` annotations to all ignored tests

### Phase 4: Test organization during file splits (ONGOING)

As the four large file splits proceed, ensure tests are co-located with their code in the new submodules. This is already noted in each split architecture document.

## Files Affected

### Integration tests (tests/qa/)
- `autonomous_targeting.rs` -- fix 2 failing tests + 3 soft assertions
- `basic_combat_unit_interface_state.rs` -- fix 1 failing test + add ignore annotations
- `combat_behaviors.rs` -- fix 1 failing test + 4 soft assertions
- `movement_behaviors.rs` -- fix 1 failing test + 2 soft assertions
- `gdo_supply_tower_and_chopper.rs` -- fix 1 failing test (stale value) + 4 soft assertions
- `automated_qa_ui_state_queries.rs` -- fix 2 failing tests or properly ignore
- `tunnel_expansions_and_starting_condition.rs` -- fix 1 failing test
- `fix_units_moving_while_attacking.rs` -- 7 soft assertions to harden
- `tunnel_object_interface_state.rs` -- 6 soft assertions to harden
- `construction_hp_rule.rs` -- 2 soft assertions to harden
- `tunnel_area_and_construction_rules.rs` -- 2 soft assertions to harden
- `attack_phases.rs` -- 1 soft assertion to harden
- `fix_memory_leak_oom_freeze.rs` -- deprecated API

### Source tests
- `src/game/types/structures.rs` -- ST_BUILD_FRAMES constant discrepancy
- `src/game/units/systems/core.rs` -- dead test helpers
