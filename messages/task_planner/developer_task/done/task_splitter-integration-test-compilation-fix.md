# integration-test-compilation-fix

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-integration-test-compilation-fix.md

## Task

Fix compilation errors in integration tests (tests/qa/) and scenario tests (tests/scenarios/) caused by the declarative entity definitions refactoring. All errors are import path updates and API adaptation — no source code changes needed.

### Scope — 5 files

1. **tests/qa/helpers.rs** — Update imports: constants moved from `game::units::types::unit_data` to `game::entities::factions::syndicate::units::agent` (AGENT_TUNNEL_BUILD_FRAMES, AGENT_CONTROL_COST) and `game::entities::factions::syndicate::units::guard` (GUARD_CONTROL_COST, GUARD_TUNNEL_SPACE_COST, GUARD_RUGGED_BONUS).

2. **tests/qa/guard_unit.rs** — Update imports for moved constants. Fix references to removed functions: `guard_type_data()`, `guard_attack_data()`. Find equivalent data in the new entity modules or adapt tests to use the new patterns.

3. **tests/qa/syndicate_agent_unit.rs** — Update imports for moved constants. Fix references to removed function `agent_type_data()`. Adapt tests to use the new entity-specific patterns.

4. **tests/qa/gdo_supply_tower_and_chopper.rs** — Fix references to removed associated functions: `BarracksState::production_cost()`, `HeadquartersState::production_cost()`, `SupplyTowerState::production_cost()`, `DeploymentCenterState::construction_cost()`. Find where cost data now lives in the entity definition modules.

5. **tests/scenarios/main.rs** — Update imports for spawn functions (`spawn_peacekeeper`, `spawn_tunnel`, `spawn_headquarters`) moved from `game::utils` to `game::entities::utils` or entity-specific modules. Fix type inference issue with `entity_mut().insert(Selected)`.

### Verification

- `cargo test --test qa` compiles and all tests pass
- `cargo test --test scenarios` compiles
- `cargo test --lib` still passes (2,094+ pass, 0 fail)

### Architecture Reference

See `artifacts/software_architect/architecture/integration-test-compilation-fix.md` for full error analysis.
