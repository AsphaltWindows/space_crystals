# Integration & Scenario Test Compilation Fix

## Status: PROPOSED
## Date: 2026-03-28

## Problem

The integration tests (`tests/qa/`) and scenario tests (`tests/scenarios/`) no longer compile due to API changes from the declarative entity definitions refactoring. Unit tests (`cargo test --lib`) pass (2,094 tests), but `cargo test --test qa` and `cargo test --test scenarios` fail with compilation errors.

This means the QA pipeline cannot verify any features that rely on integration tests.

## Root Cause

The entity definitions refactoring moved functions, constants, and types from their original locations to the new `src/game/entities/factions/` module hierarchy. The integration and scenario tests still reference the old import paths.

## Specific Errors

### Integration Tests (`tests/qa/`) — 13 compilation errors in 4 files

**Affected files:**
- `tests/qa/helpers.rs`
- `tests/qa/guard_unit.rs`
- `tests/qa/syndicate_agent_unit.rs`
- `tests/qa/gdo_supply_tower_and_chopper.rs`

**Error categories:**

1. **Moved constants** — Constants relocated from `game::units::types::unit_data` to entity-specific modules:
   - `AGENT_TUNNEL_BUILD_FRAMES` → now in `game::entities::factions::syndicate::units::agent`
   - `GUARD_CONTROL_COST`, `GUARD_TUNNEL_SPACE_COST`, `GUARD_RUGGED_BONUS` → now in `game::entities::factions::syndicate::units::guard`
   - `AGENT_CONTROL_COST` → now in `game::entities::factions::syndicate::units::agent`

2. **Removed functions** — Data functions that no longer exist (replaced by entity-specific patterns):
   - `guard_type_data()` — no longer exists
   - `guard_attack_data()` — no longer exists
   - `agent_type_data()` — no longer exists
   - `spawn_syndicate_guard()` in `game::utils` — no longer exists

3. **Removed associated functions** — Structure state methods removed:
   - `BarracksState::production_cost()` — no longer exists
   - `HeadquartersState::production_cost()` — no longer exists
   - `SupplyTowerState::production_cost()` — no longer exists
   - `DeploymentCenterState::construction_cost()` — no longer exists

### Scenario Tests (`tests/scenarios/main.rs`) — 4 compilation errors

1. **Moved spawn functions**: `spawn_peacekeeper`, `spawn_tunnel`, `spawn_headquarters` no longer in `game::utils` — likely moved to `game::entities::utils` or entity-specific modules
2. **Type inference**: `entity_mut().insert(Selected)` needs type annotation (possibly Bevy API change)

## Fix Strategy

### For moved constants/functions:
Update import paths in test files to reference the new module locations. The constants are public in their new locations.

### For removed functions (`*_type_data`, `*_attack_data`, `spawn_*`):
1. Check if equivalent functionality exists in the new entity modules
2. If the data is now embedded in spawn logic, extract the test-relevant assertions to use the new patterns
3. For `spawn_syndicate_guard`: use the TestHarness `spawn_unit_at_grid()` method or the new entity-specific spawn function

### For removed associated functions (`production_cost`, `construction_cost`):
1. Find where cost data now lives (likely in entity definition modules or a registry)
2. Update tests to reference the new cost source

### For type inference (`entity_mut`):
Add explicit type annotations where Bevy's `entity_mut()` return type can't be inferred.

## Scope

- `tests/qa/helpers.rs` — update imports
- `tests/qa/guard_unit.rs` — update imports, fix removed function references
- `tests/qa/syndicate_agent_unit.rs` — update imports, fix removed function references
- `tests/qa/gdo_supply_tower_and_chopper.rs` — update imports, fix removed function references
- `tests/scenarios/main.rs` — update imports, fix type inference

## Verification

1. `cargo test --test qa` compiles and all tests pass (or match previous pass/fail state)
2. `cargo test --test scenarios` compiles
3. `cargo test --lib` still passes (no regressions)
4. No new test files need to be created — this is purely fixing existing tests to match the refactored API
