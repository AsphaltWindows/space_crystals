# 0002 — CommandPanel standard slots are identified by hotkey letter (Z/X/C); V is free

- **Date**: 2026-11-23T12:32:23-05:00
- **Answers**: Q0005
- **Docs**: `systems/controls/selection_and_hud.md > Standard Slot Assignments`

## What changed and why
The docs already answer Q0005. The grid is 4x3 (`Grid Layout`), and every panel definition in the faction docs puts Set Rally Point on **C** (e.g. `factions/gdo/structures.md > Barracks`, `factions/syndicate/structures.md`, `factions/cults/structures.md`). The position words "bottom-center / bottom-right" were leftovers from an older 3-column layout. The hotkey letters are authoritative. The wording is now clarified:
- Z = bottom row, 1st slot (bottom-left): Back / Cancel menu
- X = bottom row, 2nd slot: Cancel Production / Cancel Upgrade
- C = bottom row, 3rd slot: Set Rally Point
- V = bottom row, 4th slot (bottom-right): no standard function, free for object-specific commands

## Implementation impact
None beyond the Director's stated assumption (option A), which matches the design.

## Verification
- Select a Barracks: Set Rally Point is in the third slot of the bottom row (hotkey C), and the bottom-right slot (V) is empty unless the object defines something there.
