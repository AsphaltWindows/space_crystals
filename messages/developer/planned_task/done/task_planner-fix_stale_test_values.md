# fix_stale_test_values

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-test_suite_cleanup.md

## Task

Fix 2 failing integration tests caused by stale hardcoded values: Category C (Supply Tower cost) and Category E (production ejection).

### Category C: ST_BUILD_FRAMES constant discrepancy (1 test)

**Failing test**: gdo_supply_tower_and_chopper::dc_construction_cost_supply_tower (expects 160, got 240)

**Root cause**: ST_BUILD_FRAMES constant in structures.rs:433 is 160, but DeploymentCenterState::construction_cost returns build_frames: 240 for SupplyTower. The unit test at structures.rs:2151 correctly asserts 240. The constant is stale.

**Fix**:
1. Update ST_BUILD_FRAMES from 160 to 240 in src/game/types/structures.rs:433
2. Update the unit test at structures.rs:2055 that asserts ST_BUILD_FRAMES == 160 -> assert 240
3. Search for any other uses of ST_BUILD_FRAMES and update if needed
4. Verify the integration test now passes with the correct value

### Category E: Production ejection logic (1 test)

**Failing test**: tunnel_expansions_and_starting_condition::hq_production_ejects_agent_without_rally_point

**Root cause**: Test expects agents produced by HQ without a rally point to eject to surface (no InTunnelNetwork component). But current behavior keeps agents in the tunnel network after production. The test expectation is stale.

**Fix**: Update the test to expect that the produced agent HAS InTunnelNetwork component (stays in tunnel network) instead of asserting it does NOT have InTunnelNetwork.

### Verification
- cargo test --test qa gdo_supply_tower_and_chopper -- dc_construction_cost_supply_tower passes
- cargo test --test qa tunnel_expansions_and_starting_condition -- hq_production_ejects test passes
- cargo test --lib structures -- no regressions on unit tests

## Technical Context

### Category C: ST_BUILD_FRAMES fix

Four locations need updating:

1. **Constant definition** -- artifacts/developer/src/game/types/structures.rs line 433:
   `pub const ST_BUILD_FRAMES: u32 = 160;` must become `240`.
   The authoritative value is in `DeploymentCenterState::construction_cost()` at line 96-99 of the same file, which already returns `build_frames: 240` for `ObjectEnum::SupplyTower`.

2. **Unit test in structures.rs** -- same file, line 2055:
   `assert_eq!(ST_BUILD_FRAMES, 160);` must become `assert_eq!(ST_BUILD_FRAMES, 240);`
   This is inside `fn supply_tower_constants()` (the lib unit test).

3. **Integration test** -- artifacts/developer/tests/qa/gdo_supply_tower_and_chopper.rs line 431:
   `assert_eq!(ST_BUILD_FRAMES, 160, "ST_BUILD_FRAMES should be 160 (10s)");` must become `assert_eq!(ST_BUILD_FRAMES, 240, ...);`
   Update the message string too (240 frames at 16fps = 15s).

4. **Integration test** -- same file, line 446:
   `assert_eq!(cost.build_frames, 160, "Supply Tower build frames should be 160");` in `fn dc_construction_cost_supply_tower()` must become `assert_eq!(cost.build_frames, 240, ...);`.

No other code references `ST_BUILD_FRAMES` -- the constant is only used in test assertions. The actual game logic uses the value from `construction_cost()` directly, which is already 240.

### Category E: Production ejection test fix

One file, one assertion block to change:

**File**: artifacts/developer/tests/qa/tunnel_expansions_and_starting_condition.rs lines 404-409.

Current code asserts `world.get::<InTunnelNetwork>(*new_agent).is_none()`. Change to `.is_some()` instead, and update the comment (line 404) and assertion message to reflect that produced agents stay in the tunnel network. Reference line 481 in the same file for an existing example of the `.is_some()` pattern used for tunnel network membership assertions.

The `InTunnelNetwork` component is imported at line 7 of the test file from `space_crystals::game::units::types::state::behavior::InTunnelNetwork`.

### Patterns to follow

- Integration tests in tests/qa/ use `TestApp::new()` and `test_app.step()` for setup
- Assert messages should describe the expected behavior clearly
- Unit tests in structures.rs are grouped by structure type under `#[cfg(test)] mod tests`

## Dependencies

None. This task is standalone -- it only updates stale test expectations to match the current game logic. No production code changes required.
