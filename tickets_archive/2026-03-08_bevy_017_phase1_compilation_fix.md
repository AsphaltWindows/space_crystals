# Ticket: Bevy 0.17 Phase 1 — Compilation Fix

## Current State
The project has been upgraded to Bevy 0.17 in `Cargo.toml`, but the codebase (~30,000 lines across 68 Rust source files) was written against an older Bevy version. `cargo build` fails with numerous compilation errors due to breaking API changes.

## Desired State
`cargo build` succeeds with zero errors. No new features or refactoring — purely mechanical API migration to get the project compiling.

## Justification
This is the critical-path blocker for all other work. The project cannot compile, which blocks development, QA, and testing. Originated from forum topic `bevy_017_upgrade_and_refactor.md` (operator directive on behalf of user). All other work is deprioritized until the migration is complete.

## Scope
- Fix all Bevy 0.17 breaking API changes (renamed types, changed function signatures, removed/replaced APIs)
- Update system function signatures (query syntax, resource access patterns, event handling)
- Fix component derive macros and bundle changes
- Update plugin registration and app builder patterns
- Fix any asset loading API changes

## QA Steps
1. [auto] Run `cargo build` — must succeed with zero errors
2. [auto] Run `cargo check` — must succeed with zero errors

## Expected Experience
`cargo build` completes successfully. The game binary is produced. No compilation errors remain. Warnings are acceptable at this stage — they are addressed in Phase 2.
