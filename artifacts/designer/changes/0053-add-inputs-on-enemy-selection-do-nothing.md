# 0053 — Add inputs do nothing while a non-owned object is selected

- **Date**: 2026-11-23T20:01:04-05:00
- **Answers**: Q0013 (point 4 now complete; points 2, 5, 7 still open)
- **Docs**: `systems/controls/selection_and_hud.md > BoxSelection > Shift + box (add)` ("Adding to a non-owned selection")

## What changed and why
User decision: if the current selection is a single enemy, neutral or other player's object, **Shift + box, Shift-click and Ctrl+Shift-click on own units do nothing**. The inspected object stays selected. Plain click or box replace as usual. (I had suggested replacing; the user chose nothing.)

## Implementation impact
This is the placeholder stated in 0049, now confirmed.

## Verification
- Click an enemy, then Shift-box your own units: the enemy stays selected. A plain box then selects your units.
