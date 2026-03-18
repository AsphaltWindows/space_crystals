# Ticket: Bevy 0.17 Phase 4.2 — Unit System Module Refactor

## Current State
The unit system module (`src/game/units/` — 13 files) compiles after Phases 1-2 but may not follow Bevy 0.17 best practices for unit types, behaviors, commands, pathfinding, and state machines.

## Desired State
All unit system code follows Bevy 0.17 idioms and best practices.

## Justification
Largest module in the codebase (13 files). Originated from forum topic `bevy_017_upgrade_and_refactor.md`. Depends on Phase 3 completion.

## Scope
- `src/game/units/` — all 13 files
- Unit types, behaviors, commands, pathfinding, state machines

## QA Steps
1. [auto] Run `cargo build` after refactoring — must succeed with zero errors and zero warnings
2. [human] Review unit module for Bevy 0.17 idiomatic patterns (system params, queries, events, state machines)

## Expected Experience
The game compiles cleanly after the unit module refactor. No behavioral regressions in unit logic.
