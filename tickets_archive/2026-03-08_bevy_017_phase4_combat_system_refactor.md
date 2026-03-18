# Ticket: Bevy 0.17 Phase 4.1 — Combat System Module Refactor

## Current State
The combat system module (`src/game/combat/` — 7 files) compiles after Phases 1-2 but may not follow Bevy 0.17 best practices for projectile systems, damage calculation, turrets, and attack systems.

## Desired State
All combat system code follows Bevy 0.17 idioms and best practices.

## Justification
Module-by-module refactoring ensures each subsystem is fully modernized. Originated from forum topic `bevy_017_upgrade_and_refactor.md`. Depends on Phase 3 completion.

## Scope
- `src/game/combat/` — all 7 files
- Projectile systems, damage calculation, turrets, attack systems

## QA Steps
1. [auto] Run `cargo build` after refactoring — must succeed with zero errors and zero warnings
2. [human] Review combat module for Bevy 0.17 idiomatic patterns (system params, queries, events)

## Expected Experience
The game compiles cleanly after the combat module refactor. No behavioral regressions in combat logic.
