# Ticket: Bevy 0.17 Phase 3 — ECS Architecture Refactor

## Current State
After Phases 1-2, the project compiles cleanly but the core ECS architecture may still use patterns from the older Bevy version rather than idiomatic Bevy 0.17 patterns.

## Desired State
The core ECS layer follows Bevy 0.17 best practices: system ordering/scheduling, component design, resource patterns, and plugin organization are all idiomatic.

## Justification
Migrating the ECS architecture to idiomatic Bevy 0.17 reduces future maintenance burden and takes advantage of engine improvements. The developer agent has access to a Bevy skill with best-practice guidance. Originated from forum topic `bevy_017_upgrade_and_refactor.md`. Depends on Phase 2 completion.

## Scope
- **System ordering and scheduling**: Review all system sets, ordering constraints, and run conditions for Bevy 0.17 scheduling APIs
- **Component design**: Audit component structs for idiomatic Bevy 0.17 patterns, migrate away from deprecated patterns
- **Resource patterns**: Ensure resources follow current best practices
- **Plugin organization**: Review plugin structure (`src/game/mod.rs`, `src/ui/mod.rs`, `src/simulation/mod.rs`) for proper Bevy 0.17 plugin patterns

## QA Steps
1. [auto] Run `cargo build` — must succeed with zero errors and zero warnings
2. [human] Review system ordering in `src/game/mod.rs`, `src/ui/mod.rs`, `src/simulation/mod.rs` — confirm they use Bevy 0.17 scheduling APIs
3. [human] Spot-check component definitions in `src/game/types/` — confirm they follow Bevy 0.17 derive patterns

## Expected Experience
The game compiles and runs identically to post-Phase-2. No behavioral changes. Code review reveals idiomatic Bevy 0.17 ECS patterns throughout the core architecture.
