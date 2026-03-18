# Ticket: Bevy 0.17 Phase 4.4 — UI System Module Refactor

## Current State
The UI system module (`src/ui/` — 6 files) compiles after Phases 1-2 but may not follow Bevy 0.17 best practices for HUD, command panel, and menus.

## Desired State
All UI system code follows Bevy 0.17 idioms and best practices.

## Justification
Bevy 0.17 has significant UI changes. Originated from forum topic `bevy_017_upgrade_and_refactor.md`. Depends on Phase 3 completion.

## Scope
- `src/ui/` — all 6 files
- HUD, command panel, menus

## QA Steps
1. [auto] Run `cargo build` after refactoring — must succeed with zero errors and zero warnings
2. [human] Review UI module for Bevy 0.17 UI idioms (Node, styling, layout patterns)

## Expected Experience
The game compiles cleanly after the UI module refactor. No behavioral regressions in UI rendering or interaction.
