# Q0005 — CommandPanel: standard slot positions don't match the 4×3 grid

- **Date**: 2026-11-23T11:55:00-05:00
- **Blocking**: no — proceeding on assumption: the hotkey letters are authoritative (Z = Back/Cancel menu, X = Cancel Production/Upgrade, C = Set Rally Point) and slot positions follow the grid, so C is the third slot of the bottom row and V (bottom-right) has no standard function.
- **Docs**: `systems/controls/selection_and_hud.md > CommandPanel`, `> Standard Slot Assignments`

## Question
The grid is 4 columns × 3 rows, with the bottom row `Z X C V`. But Standard Slot Assignments calls X "bottom-center" and C "bottom-right", which only fits a 3-column grid. Which is intended: standard functions on Z/X/C (positions 1–3 of the bottom row) or on bottom-left / bottom-center / bottom-right? And does V have a standard role?

## Context
Found while writing the client architecture brief. The command panel is built in M1.

## Options
- **A**: keep the hotkeys (Z, X, C) and fix the position words. V stays free.
- **B**: keep the positions. Back = Z, Cancel = X or C, and Set Rally Point moves to V (bottom-right).
