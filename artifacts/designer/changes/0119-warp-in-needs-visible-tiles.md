# 0119 — Warp In needs Visible tiles

- **Date**: 2026-11-24T14:10:00-05:00
- **Answers**: Q0040 (point 3)
- **Docs**: `core/entities.md > Placement Validation > Warp In`

## What changed and why
Option B (differs from the Director's proposal). Like Direct Placement, every Warp In footprint tile must be Visible; non-visible tiles are red on the ghost. Normally irrelevant (Beacon sight 4 GU covers current footprints), but it stops the ghost from leaking hidden units in fog, and matches GDO.

## Implementation impact
Warp In ghost validity adds the Visible check; ghost colour never depends on unseen units.

## Verification
Normal Warp In near a Beacon: all green as before. If any footprint tile is not Visible (test setup), it's red regardless of what stands there.
