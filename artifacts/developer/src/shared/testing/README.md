# Testing Module

Provides `TestApp` and `TestHarness`, the core infrastructure for headless game
integration tests. Used by both `artifacts/developer/tests/scenarios/` and `artifacts/developer/tests/qa/` test targets.

## Files

- `test_app.rs` — `TestApp` struct: headless Bevy App with all game plugins (no rendering)
- `harness.rs` — `TestHarness` struct with command, query, and state methods
- `assertions.rs` — Free assertion helper functions (`assert_position_near`, `assert_dead`, etc.)
- `types.rs` — Test-specific data types (ResourceSnapshot, EntityFilter, StructureState, TunnelNetworkInfo)
- `utils.rs` — Shared test helper functions (reserved)

## Feature Gate

This module is only compiled when `#[cfg(any(test, feature = "testing"))]` is active.
Integration tests in `artifacts/developer/tests/` use the `testing` Cargo feature.

## QA Test Infrastructure

The QA agent generates test files in `artifacts/developer/tests/qa/[task_name].rs`. Each file contains
one test function per `[auto]`-tagged QA step. The `artifacts/developer/tests/qa/helpers.rs` module
re-exports `TestApp`, `TestHarness`, assertions, and common game types for convenience.
