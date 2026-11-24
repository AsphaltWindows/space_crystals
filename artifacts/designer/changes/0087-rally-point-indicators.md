# 0087 — Rally point indicators

- **Date**: 2026-11-24T03:10:00-05:00
- **Answers**: Q0024 (point 6; point 7 still open)
- **Docs**: `systems/controls/selection_and_hud.md > CommandIndicators` (three new rows, plus a rally point note)

## What changed and why
The Director's proposal is accepted, with a line added. While a producing building is selected, its rally point is shown:
- a green Location marker for a ground rally point;
- a green Object marker on an own or neutral object;
- a red Object marker on an enemy;
- a **line** from the building to the rally point.

The user calls this fine for now.

## Implementation impact
- New indicator rows. They follow the existing CommandIndicators visibility rule: shown only while the building is in the Selection.
- No indicator when RallyPoint is None.

## Verification
1. Set a Barracks rally point on the ground and select the Barracks. A green ground marker and a line from the Barracks to it are shown.
2. Rally on an enemy. The marker is red, around the enemy.
3. Deselect the Barracks. Both the marker and the line disappear.
