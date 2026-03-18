# Ticket: Bevy 0.17 Phase 4.5 — Simulation Layer Module Refactor

## Current State
The simulation layer (`src/simulation/` — 6 files) compiles after Phases 1-2 but may not follow Bevy 0.17 best practices for diagnostics and performance instrumentation.

## Desired State
All simulation layer code follows Bevy 0.17 idioms and best practices.

## Justification
Originated from forum topic `bevy_017_upgrade_and_refactor.md`. Depends on Phase 3 completion.

## Scope
- `src/simulation/` — all 6 files
- Diagnostics, performance instrumentation

## QA Steps
1. [auto] Run `cargo build` after refactoring — must succeed with zero errors and zero warnings
2. [human] Review simulation module for Bevy 0.17 idiomatic patterns

## Expected Experience
The game compiles cleanly after the simulation module refactor. No behavioral regressions in diagnostics/instrumentation.
