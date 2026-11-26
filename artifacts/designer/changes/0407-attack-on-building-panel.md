# 0407 — A building in focus shows the selection's Attack on its free A (from the docs)

- **Date**: 2026-11-26T15:46:00-05:00
- **Answers**: Q0151 part 3 (parts 1 and 2 follow separately)
- **Docs**: `systems/controls/interface.md > CommandPanel > Standard Slot Assignments` (0399, unchanged)

## What changed and why
0399 (user, 2026-11-27) says A: Attack is shown "whenever at least one selected group can attack, **whichever group is in focus**". A building group in focus is one of those groups, so when its A slot is free its panel shows the selection's Attack, issued to the units as on a unit panel (option a, as built). No doc change.

## Implementation impact
None.

## Verification
Select a building whose A slot is free plus some units that can attack, Tab to the building: A: Attack is shown, and A + click sends the units to attack.
