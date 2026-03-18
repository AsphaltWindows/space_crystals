# Ticket: Bevy 0.17 Phase 5 — Integration Testing Update

## Current State
After Phases 1-4, the codebase compiles and follows Bevy 0.17 idioms, but integration tests in `tests/scenarios/` and the test infrastructure in `src/shared/testing/` may fail against the refactored code.

## Desired State
`cargo test` passes. All existing integration tests work against the refactored codebase.

## Justification
Tests must pass before normal pipeline operations (QA, new development) can resume. QA is paused until this phase completes. Originated from forum topic `bevy_017_upgrade_and_refactor.md`. Depends on all Phase 4 sub-tickets being complete.

## Scope
- Fix all integration tests in `tests/scenarios/`
- Update `src/shared/testing/` (TestApp, harness, assertions) for Bevy 0.17 APIs
- Ensure `cargo test` passes

## QA Steps
1. [auto] Run `cargo test` — all tests must pass
2. [auto] Run `cargo build` — must still succeed with zero errors and zero warnings
3. [semi] Review test output for any skipped or ignored tests that should be re-enabled

## Expected Experience
`cargo test` passes with all tests green. The project is fully operational on Bevy 0.17 and normal pipeline operations can resume.
