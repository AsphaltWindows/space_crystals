# 0156 — Destination inside a remembered obstacle stays as resolved

- **Date**: 2026-11-24T19:45:00-05:00
- **Answers**: Q0033 (point 1)
- **Docs**: `systems/controls/behaviors.md > PathKnowledge`

## What changed and why
Option A (current build). The destination resolved at order time (nearest reachable spot by the player's knowledge) is final; seeing the obstacle gone later doesn't extend the move to the clicked point.

## Implementation impact
None (matches M2-E).

## Verification
Click inside a remembered, actually destroyed enemy building: the unit stops at the near edge even after it sees the ground is empty.
