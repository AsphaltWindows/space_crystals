# Ticket: Bevy 0.17 Phase 4.7 — Game Types Module Refactor

## Current State
The game types module (`src/game/types/` — 6 files) compiles after Phases 1-2 but may not follow Bevy 0.17 best practices for object, structure, and faction type definitions.

## Desired State
All game types code follows Bevy 0.17 idioms and best practices.

## Justification
Originated from forum topic `bevy_017_upgrade_and_refactor.md`. Depends on Phase 3 completion.

## Scope
- `src/game/types/` — all 6 files
- Objects, structures, factions type definitions

## QA Steps
1. [auto] Run `cargo build` after refactoring — must succeed with zero errors and zero warnings
2. [human] Review game types module for Bevy 0.17 component/resource derive patterns

## Expected Experience
The game compiles cleanly after the game types module refactor. No behavioral regressions in type definitions.
