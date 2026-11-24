# 0049 — Shift + box adds to the selection; empty-ground clicks

- **Date**: 2026-11-23T19:49:20-05:00
- **Answers**: Q0013 (point 4; points 2, 5, 7 still open)
- **Docs**: `systems/controls/selection_and_hud.md > BoxSelection > Shift + box (add)`, `> Empty clicks and boxes` (new)

## What changed and why
- **Shift + box adds** the box result (after the priority tiers) to the current selection, which **always stays selected**. It never removes, unlike Shift-click, which toggles. This **replaces** the current "Shift-box ignores modifiers" behaviour.
- Box contents that can't be added under the Selection Constraints (non-owned objects into an own selection) aren't added, and the selection is unchanged.
- Kept as built: a plain empty click or empty box clears; Shift/Ctrl-click on empty ground does nothing; Ctrl wins on Ctrl+Shift.

## Implementation impact
Change Shift+box from replace to add-only. Still open: when the **current** selection is a single enemy or neutral object and the player Shift-boxes own units (asked the user; until answered, keep the enemy selected, per "current selection remains selected").

## Verification
- Select Peacekeepers on the left, Shift-box Raptors on the right: both groups are selected.
- Shift-box over already-selected units: nothing is deselected.
- With own units selected, Shift-box only enemies: the selection is unchanged.
