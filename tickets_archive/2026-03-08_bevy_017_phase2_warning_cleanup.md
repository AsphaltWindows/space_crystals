# Ticket: Bevy 0.17 Phase 2 — Warning Cleanup

## Current State
After Phase 1, the project compiles but likely produces compiler warnings: dead code, unused imports, deprecated API usage that compiled but warns.

## Desired State
`cargo build` produces zero warnings.

## Justification
Clean warning output ensures future regressions are immediately visible. Warnings from deprecated APIs may indicate code paths that will break in future Bevy releases. Originated from forum topic `bevy_017_upgrade_and_refactor.md`. Depends on Phase 1 completion.

## Scope
- Eliminate dead code warnings
- Remove unused imports
- Replace deprecated API usage that compiled but warns
- Address any other compiler warnings

## QA Steps
1. [auto] Run `cargo build 2>&1` — must produce zero warnings
2. [auto] Run `cargo check 2>&1` — must produce zero warnings

## Expected Experience
`cargo build` completes with completely clean output — no errors, no warnings.
