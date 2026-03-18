# Ticket: Bevy 0.17 Phase 4.6 — Shared/Testing Module Refactor

## Current State
The shared/testing module (`src/shared/` — 7 files) compiles after Phases 1-2 but may not follow Bevy 0.17 best practices for test harness, test app, and assertions.

## Desired State
All shared/testing code follows Bevy 0.17 idioms and best practices.

## Justification
The test infrastructure must be modernized before Phase 5 integration testing. Originated from forum topic `bevy_017_upgrade_and_refactor.md`. Depends on Phase 3 completion.

## Scope
- `src/shared/` — all 7 files
- Test harness, test app, assertions

## QA Steps
1. [auto] Run `cargo build` after refactoring — must succeed with zero errors and zero warnings
2. [human] Review shared/testing module for Bevy 0.17 test patterns

## Expected Experience
The game compiles cleanly after the shared module refactor. Test infrastructure is ready for Phase 5 integration testing.
