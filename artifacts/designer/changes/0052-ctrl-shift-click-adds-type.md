# 0052 — Ctrl+Shift-click adds all on-screen units of that type

- **Date**: 2026-11-23T20:00:17-05:00
- **Answers**: Q0013 (point 4, last part; still open: Shift-adding own units while an enemy is selected, and points 2, 5, 7)
- **Docs**: `systems/controls/selection_and_hud.md > MapClickSelection` (new row), `> BoxSelection > Empty clicks and boxes`

## What changed and why
User decision: **Ctrl+Shift-click an own unit adds all own units of that type on screen** to the current selection, never removing anything. This **replaces** the current "Ctrl wins" behaviour. On empty ground, Ctrl+Shift-click does nothing.

## Implementation impact
New add-type modifier combination.

## Verification
- Select Peacekeepers, Ctrl+Shift-click a Raptor: all on-screen Raptors are added and the Peacekeepers stay selected.
