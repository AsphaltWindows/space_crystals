# 0447 — Correction to 0446: the default rally point can be cleared

- **Date**: 2026-11-26T21:40:00-05:00
- **Answers**: — (corrects 0446)
- **Docs**: `core/entities.md > Entrance and exit > Exit Doorway` (point 7)

## What changed and why
0446 wrongly said there is no command to clear a rally point. There is (user, 2026-11-24; `systems/controls/interface.md > CommandPanel`): right-clicking the building itself, or C and clicking it, sets RallyPoint = None. The default vehicle rally point can be cleared that way; then the doorway stall is the player's to handle.

## Implementation impact
None beyond 0446; the existing clear rule applies to the default rally point.

## Verification
Select a new Chop Shop and right-click it: its default rally point disappears; a second Seeker then stalls behind the first, as before 0446.
