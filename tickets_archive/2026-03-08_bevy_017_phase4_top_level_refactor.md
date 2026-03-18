# Ticket: Bevy 0.17 Phase 4.8 — Top-Level App Setup Refactor

## Current State
The top-level files (`src/main.rs`, `src/lib.rs`, `src/game/mod.rs`, `src/game/utils.rs`) compile after Phases 1-2 but may not follow Bevy 0.17 best practices for app setup and plugin registration.

## Desired State
All top-level app setup code follows Bevy 0.17 idioms and best practices.

## Justification
App setup and plugin registration are foundational — they must be idiomatic before the module refactors are meaningful. Originated from forum topic `bevy_017_upgrade_and_refactor.md`. Depends on Phase 3 completion.

## Scope
- `src/main.rs`, `src/lib.rs`, `src/game/mod.rs`, `src/game/utils.rs`
- App setup, plugin registration

## QA Steps
1. [auto] Run `cargo build` after refactoring — must succeed with zero errors and zero warnings
2. [human] Review top-level files for Bevy 0.17 App builder and plugin patterns

## Expected Experience
The game compiles cleanly after the top-level refactor. App boots and runs identically.
