# 0129 — Other orders keep the load

- **Date**: 2026-11-24T16:05:00-05:00
- **Answers**: Q0037 (point 3)
- **Docs**: `factions/colonists/units.md > Other orders keep the load` (new)

## What changed and why
Option A. A carrying Drone given a non-gathering order (Move, Stop, …) keeps its load and delivers it at its next drop-off (next Gather trip per 0128, or a Drop Off order).

## Implementation impact
Load is never cleared by order changes (only by delivery or by deploy completion, 0114).

## Verification
Move a loaded Drone away, then right-click a patch: it skips mining there and delivers the old load (0128).
