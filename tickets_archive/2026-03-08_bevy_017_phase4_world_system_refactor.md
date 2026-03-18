# Ticket: Bevy 0.17 Phase 4.3 — World System Module Refactor

## Current State
The world system module (`src/game/world/` — 6 files) compiles after Phases 1-2 but may not follow Bevy 0.17 best practices for map, factions, resources, and world utilities.

## Desired State
All world system code follows Bevy 0.17 idioms and best practices.

## Justification
Originated from forum topic `bevy_017_upgrade_and_refactor.md`. Depends on Phase 3 completion.

## Scope
- `src/game/world/` — all 6 files
- Map, factions, resources, world utilities

## QA Steps
1. [auto] Run `cargo build` after refactoring — must succeed with zero errors and zero warnings
2. [human] Review world module for Bevy 0.17 idiomatic patterns

## Expected Experience
The game compiles cleanly after the world module refactor. No behavioral regressions in world/map logic.
