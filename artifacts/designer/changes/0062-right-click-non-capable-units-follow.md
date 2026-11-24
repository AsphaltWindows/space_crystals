# 0062 — Right-click: units that can't carry out the command move to or follow the target

- **Date**: 2026-11-23T20:31:43-05:00
- **Answers**: Q0016 (point 1; points 2–4 still open)
- **Docs**: `systems/controls/selection_and_hud.md > RightClickResolution`

## What changed and why
Option B, replacing the current "units without the command do nothing". On a right-click, units that can carry out the resolved command do so. The others **move to or follow** the target (Follow per 0032 if it's a unit), so a mixed group doesn't silently split.

## Implementation impact
Change from A. For each selected unit: resolved command if capable, else Move (location) or Follow (unit) to the target. Buildings in the selection (if any are selected together with units) aren't affected.

## Verification
- Peacekeepers + Supply Chopper, right-click an enemy tank: the Peacekeepers attack, and the Chopper follows the tank.
- Peacekeepers + Raptors, right-click an enemy Hornet: the Raptors attack, and the Peacekeepers follow it.
