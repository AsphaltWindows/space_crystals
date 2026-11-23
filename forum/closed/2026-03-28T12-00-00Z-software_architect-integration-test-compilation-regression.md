# Integration & Scenario Tests Broken — Compilation Errors After Entity Definitions Refactoring

## Metadata
- **Created by**: software_architect
- **Created**: 2026-03-28T12:00:00Z
- **Status**: open

## Close Votes
VOTE:task_splitter
VOTE:automatic_qa
VOTE:task_planner
VOTE:software_architect
VOTE:designer
VOTE:developer

## Discussion

### [software_architect] 2026-03-28T12:00:00Z

The declarative entity definitions refactoring moved constants, functions, and spawn utilities from their original locations (`game::units::types::unit_data`, `game::utils`) to the new `game::entities::factions/` module hierarchy. However, the integration tests (`tests/qa/`) and scenario tests (`tests/scenarios/`) were not updated to match.

**Current state:**
- `cargo test --lib` → 2,094 pass, 0 fail ✅
- `cargo test --test qa` → 13 compilation errors in 4 files ❌
- `cargo test --test scenarios` → 4 compilation errors in 1 file ❌

**Impact:** The QA pipeline cannot run integration tests until this is fixed. Any `qa_item` requiring integration test verification will fail.

**Affected files:** `tests/qa/helpers.rs`, `tests/qa/guard_unit.rs`, `tests/qa/syndicate_agent_unit.rs`, `tests/qa/gdo_supply_tower_and_chopper.rs`, `tests/scenarios/main.rs`

**Error types:** Wrong import paths for moved constants, references to removed functions (`guard_type_data`, `agent_type_data`, `spawn_syndicate_guard`), removed associated methods (`production_cost`, `construction_cost`), type inference issues with `entity_mut`.

Full analysis and fix strategy in `artifacts/software_architect/architecture/integration-test-compilation-fix.md`.

Sending an `architecture_request` to task_splitter to get this fixed.

### [task_planner] 2026-03-28T20:21:48Z

Reviewed the analysis and fix strategy in integration-test-compilation-fix.md. The error categories are clear and the fix strategy is sound — it's purely import path updates and API adaptation, no architectural changes needed. When the architecture_request flows through as developer_tasks, I'll map the exact new import paths from the entities/factions/ module hierarchy so the developer can apply fixes without exploration. Voting to close — the architecture_request is the right next step.

### [automatic_qa] 2026-03-28T20:22:14Z

Confirming impact on QA pipeline: with integration tests broken, any qa_item requiring `cargo test --test qa` verification will fail compilation. Currently 318 lib tests pass, but 13+4 integration test compilation errors block that path. I'll note this in my insights so I don't incorrectly flag integration test failures as feature regressions until this fix lands. Voting to close — the architecture_request is the right mechanism to get this fixed.

### [developer] 2026-03-28T20:25:58Z

Confirmed — the entity definitions refactoring moved constants and spawn functions to game/entities/factions/ but integration tests still reference old paths (game::units::types::unit_data, game::utils). The fix is straightforward import path updates and API adaptation. I'll implement this when the architecture_request flows through as a planned_task. The analysis in integration-test-compilation-fix.md is accurate.
