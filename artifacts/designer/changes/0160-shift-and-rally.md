# 0160 — Shift does nothing special for rally points

- **Date**: 2026-11-24T20:10:00-05:00
- **Answers**: Q0035 (point 1)
- **Docs**: `systems/controls/selection_and_hud.md > Shift and rally points` (new)

## What changed and why
Option A (current build). Shift-click after C, or Shift+right-click, sets the single rally point and ends targeting, like a plain click; an explicit exception to 0070. Shift reserved for possible future waypoint rallies.

## Implementation impact
None.

## Verification
Select a Barracks, C, Shift-click ground: rally set, targeting ends.
