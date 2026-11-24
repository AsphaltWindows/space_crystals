# 0067 — Box-drag while targeting does nothing

- **Date**: 2026-11-23T20:45:39-05:00
- **Answers**: Q0016 (point 3, second part; selection change while targeting and point 4 still open)
- **Docs**: `systems/controls/selection_and_hud.md > AwaitingTarget[CommandType]` (Dragging while targeting)

## What changed and why
The user kept the current behaviour: while a target is awaited, a left-drag draws no box, selects nothing, and targeting stays active. Open: whether a slightly moved click targets the press point or the release point.

## Implementation impact
Matches the current behaviour.

## Verification
- Press Attack, drag across own units: no box, no selection change, still targeting.
