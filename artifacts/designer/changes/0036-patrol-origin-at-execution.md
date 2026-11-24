# 0036 — Patrol origin is the unit's position when the Patrol starts executing

- **Date**: 2026-11-23T19:20:03-05:00
- **Answers**: Q0010 (part 2; failed patrol leg still open)
- **Docs**: `systems/controls/commands.md > Patrol`, `systems/controls/behaviors.md > Patrolling` (Reads, PatrolOrigin)

## What changed and why
Option A. This resolves the docs conflict: the patrol origin is where the unit is when the Patrol **starts executing**. For a Shift-queued Patrol, that's where the previous order ended. "Move to X, then patrol to Y" therefore patrols X ↔ Y. In a group, each unit's origin is its own position at start, and its destination is its magic-box spot.

## Implementation impact
Matches the Director's default.

## Verification
- Shift-queue Move to X, then Patrol to Y: after reaching X the unit patrols between X and Y, never returning to its original spot.
