# 0065 — Invalid target click: ignored, targeting stays, with feedback

- **Date**: 2026-11-23T20:37:18-05:00
- **Answers**: Q0016 (point 2; points 3 and 4 still open)
- **Docs**: `systems/controls/selection_and_hud.md > AwaitingTarget[CommandType]` (Invalid target clicks)

## What changed and why
Option C. A left-click on a target the pending command can't use is ignored and targeting stays active (as now), **plus** short feedback: an "invalid target" message and/or sound. The exact form is TBD and will tie into the notification system later; use a simple placeholder.

## Implementation impact
Add placeholder feedback to the current behaviour.

## Verification
- Press Patrol, click an enemy unit: a short "invalid target" message or sound, still in Patrol targeting. Click ground: the patrol is issued.
