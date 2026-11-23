# 0006 — Slopes don't affect movement speed

- **Date**: 2026-11-23T13:20:55-05:00
- **Answers**: —
- **Docs**: `core/map.md > Elevation > Flat and sloped tiles`

## What changed and why
The user decided there's no slowdown or speedup on slopes for now, going up or down. Elevation matters through ElevationModifier and cliffs only. This may be revisited after playtesting.

## Implementation impact
Confirms the default stated in 0004: movement speed and pathfinding costs ignore terrain height. Nothing to change.

## Verification
- A unit takes the same time to cross a hill as an equally long stretch of flat ground.
