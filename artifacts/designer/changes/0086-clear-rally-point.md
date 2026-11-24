# 0086 — Rally point on the building itself clears it

- **Date**: 2026-11-24T03:05:00-05:00
- **Answers**: Q0024 (point 5; points 6–7 still open)
- **Docs**: `systems/controls/selection_and_hud.md > Standard Slot Assignments` (C: Set Rally Point) and `> Production Building Default Right-Click`

## What changed and why
The user chose A. Setting a production building's rally point **on the building itself** clears it (RallyPoint = None). This works both through C + click and through a right-click on the building. It applies to every unit-producing structure. No extra panel command is needed.

## Implementation impact
- RallyPoint is set to None when the rally target is the producing building.
- With no rally point, units walk straight out of the exit doorway (0080) and then have no command.

## Verification
1. Set a Barracks rally point on the ground. Then select the Barracks and right-click the Barracks itself. The rally marker disappears.
2. The next Peacekeeper walks just out of side B and stops.
3. The same works with C, then a click on the Barracks.
