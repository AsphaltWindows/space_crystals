# integration-test-compilation-fix

## Metadata
- **From**: software_architect
- **To**: task_splitter

## Content

## Architecture Document

artifacts/software_architect/architecture/integration-test-compilation-fix.md

## Summary

The declarative entity definitions refactoring moved constants, functions, and spawn utilities to new module paths but did not update the integration tests (tests/qa/) and scenario tests (tests/scenarios/). Both test targets fail to compile (13 errors in qa, 4 errors in scenarios). Unit tests pass fine (2,094 pass). This blocks the QA pipeline from running integration test verification.

## Scope

5 test files need import path updates and function reference fixes:
- tests/qa/helpers.rs
- tests/qa/guard_unit.rs
- tests/qa/syndicate_agent_unit.rs
- tests/qa/gdo_supply_tower_and_chopper.rs
- tests/scenarios/main.rs

No source code changes needed — only test files.

## QA Instructions

- Run cargo test --test qa and verify it compiles and all previously-passing tests still pass
- Run cargo test --test scenarios and verify it compiles
- Run cargo test --lib and verify no regressions (should be 2,094+ pass, 0 fail)

## Testing Requirements

- No new tests needed — this is fixing existing test compilation
- All 318+ previously-passing integration tests must still pass after the fix
- The scenario test movement_runs_after_ui_hud_ordering may still fail (pre-existing issue) — that is acceptable
- Verify cargo test --test qa shows 0 compilation errors and 0 test failures
- Verify cargo test --test scenarios compiles successfully
