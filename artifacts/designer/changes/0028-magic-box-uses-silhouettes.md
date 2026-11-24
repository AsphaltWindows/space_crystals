# 0028 — Magic box is drawn around unit silhouettes, not centres

- **Date**: 2026-11-23T18:53:22-05:00
- **Answers**: Q0015 (point 2; point 3 still open)
- **Docs**: `systems/controls/commands.md > Group Destinations (Magic Box)`

## What changed and why
Option A, as the Director assumed. The box bounds the units' **full Silhouettes**, so a straight line of units has a real box and clicking on or between them converges the group. The group centre used for offsets is the centre of that box.

## Implementation impact
Matches the Director's assumption. 0021's Verification ("right-click in the middle of the same group: they converge") now holds for a line too.

## Verification
- Line up units horizontally and right-click between two of them: the group converges.
- Right-click a few tiles above the line: the line moves there, keeping its shape.
