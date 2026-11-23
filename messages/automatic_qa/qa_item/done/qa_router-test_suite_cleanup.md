# test_suite_cleanup

## Metadata
- **From**: qa_router
- **To**: automatic_qa

## Content

## Content

## Content

## Architecture Document

artifacts/software_architect/architecture/test-suite-cleanup.md

## Summary

The test suite has 9 consistently failing integration tests across 5 root-cause categories, 33 soft assertions that silently pass when components are missing, and a test infrastructure bug in TestHarness::set_selection(). This cleanup fixes test reliability issues that undermine confidence in the CI pipeline and cause noise during feature development. Evidence: forum topic 2026-03-24T12-05-00Z-operator-unit-test-review, with agreement from task_planner.

## Scope

**Integration tests (tests/qa/)**: 13 files affected -- fixing 9 failing tests, hardening 33 soft assertions, removing dead code, adding ignore annotations.

**Test infrastructure (src/shared/testing/)**: Fix TestHarness::set_selection() to update the Selection resource groups in addition to the Selected ECS component.

**Source tests (src/game/types/structures.rs)**: Resolve ST_BUILD_FRAMES constant discrepancy (160 vs 240).

**Source tests (src/game/units/systems/core.rs)**: Remove 2 dead test helper functions.

No game logic changes. No behavioral changes. Only test code and test infrastructure.

## QA Instructions

1. Run `cargo test --lib` -- all 1907+ unit tests must pass (zero regressions)
2. Run `cargo test --test qa` -- the 9 previously failing tests must now pass. Total passing count should increase from 309 to 318 (or close, depending on whether some are converted to properly-ignored)
3. Run `cargo test --test qa 2>&1 | grep "FAILED|failures"` -- zero failures expected
4. Run `cargo test --test qa 2>&1 | grep "warning"` -- warning count should decrease (dead imports removed)
5. Verify no game logic was changed -- only test files and test infrastructure

## Testing Requirements

This IS a test cleanup task, so the "tests" are the fixes themselves. Specific verification:

- **Auto-targeting tests (Category B)**: After fixing VisibilityStateEnum on spawned enemies, the 3 auto-targeting tests must pass and correctly detect engagement (non-None attack phase and target)
- **Stop command tests (Category A)**: step_3 should assert Idle (not Stop) for stationary units. step_8 should either use more frames or verify the deceleration math is correct.
- **Interface state tests (Category D)**: After fixing TestHarness::set_selection() to populate Selection resource groups, the 2 UI state tests must pass showing correct BarracksMenu/DcBuildMenu states.
- **Supply Tower cost test (Category C)**: Determine correct value (160 or 240), update ST_BUILD_FRAMES constant or construction_cost method to be consistent, update all tests.
- **Production ejection test (Category E)**: Update test expectation to match current production behavior (agents stay in tunnel network).
- **Soft assertions**: All 33 `if let Some` guards in integration tests replaced with `expect()` or `unwrap()`. Run the full integration suite to verify none of these newly-hardened assertions fail (if they do, that reveals a real bug to fix).
- **Dead code**: Remove create_dropoff_test_world and create_right_click_entity_world from core.rs tests. Remove unused imports. No compilation warnings from test code.
