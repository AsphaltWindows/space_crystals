# 0130 — Interrupted mining collects nothing

- **Date**: 2026-11-24T16:10:00-05:00
- **Answers**: Q0037 (point 4)
- **Docs**: `factions/colonists/units.md > Interrupted mining` (new)

## What changed and why
Option A. Interrupting a Drone before MiningDuration (32 frames) completes yields no partial load; the patch frees at once.

## Implementation impact
No partial-load arithmetic. Occupancy released on interruption.

## Verification
Order a mining Drone away at frame 20: it carries nothing; another waiting Drone starts mining that patch immediately.
