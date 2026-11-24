# 0068 — Targeting clicks count at the press point

- **Date**: 2026-11-23T20:46:27-05:00
- **Answers**: Q0016 (point 3 follow-up)
- **Docs**: `systems/controls/selection_and_hud.md > AwaitingTarget[CommandType]` (Dragging while targeting)

## What changed and why
User decision: while targeting, a click counts at the **press point**, even if the mouse drifts before release. Pressing on an enemy and releasing on the ground still issues Attack on that enemy, not Attack Move.

## Implementation impact
Resolve the target on mouse-down.

## Verification
- Press Attack, press the mouse on an enemy, drift onto the ground, release: Attack on that enemy.
