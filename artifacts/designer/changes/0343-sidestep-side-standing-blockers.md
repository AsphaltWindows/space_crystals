# 0343 — Sidestep side at a standing blocker: the side it's on; right first only dead centre

- **Date**: 2026-11-26T21:50:00-05:00
- **Answers**: Q0129
- **Docs**: `systems/units.md > Ground Collision` (Moving units steer around blockers: Which side)

## What changed and why
User chose **(a), as built**:
- At a **standing** blocker, a unit goes round on the side of the blocker it is already on. "Right first" decides only dead centre.
- **Moving** blockers are unchanged: head-on both step right, otherwise the way it leans.

## Implementation impact
None. The doc now matches the build.

## Verification
As in Q0129's measurements.
