# fix_stale_test_values

## Metadata
- **From**: task_splitter
- **To**: task_planner

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

### Files
- src/game/types/structures.rs -- fix ST_BUILD_FRAMES constant (line 433) and its unit test (line 2055)
- tests/qa/gdo_supply_tower_and_chopper.rs -- verify test passes
- tests/qa/tunnel_expansions_and_starting_condition.rs -- fix ejection test expectation

### Verification
- cargo test --test qa gdo_supply_tower_and_chopper -- dc_construction_cost_supply_tower passes
- cargo test --test qa tunnel_expansions_and_starting_condition -- hq_production_ejects test passes
- cargo test --lib structures -- no regressions on unit tests
