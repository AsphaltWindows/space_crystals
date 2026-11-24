# 0048 — Selection edge cases answered from the docs (Q0013 points 1, 3, 6)

- **Date**: 2026-11-23T19:44:49-05:00
- **Answers**: Q0013 (points 1, 3, 6; points 2, 4, 5, 7 go to the user)
- **Docs**: `systems/controls/selection_and_hud.md > Selection > ActiveGroup` (type clarified), `> SelectionPanel` (wording)

## What changed and why
These three were already settled by the docs; the wording is now clarified:
1. **Ungroupable objects:** `Selection > Constraints` and `SelectionGroup` say each Ungroupable instance is its own SelectionGroup. ActiveGroup is now typed as a **SelectionGroup** (not an ObjectEnum), so it can point at one specific Ungroupable instance. Tab cycles through them one by one. This matches your current behaviour.
3. **Modified clicks on enemy or neutral objects:** `Selection > Constraints`: any non-owned object in the selection forces a single-object selection. Shift-, Ctrl- or double-click on one therefore selects it alone, as a plain click does. This matches your current behaviour.
6. **SelectionPanel:** "objects" is right; the "units" wording was stale. One correction to your note: it can't apply to resources, because non-owned and neutral objects are always selected alone. A 2+ selection is always own units and/or structures.

## Implementation impact
No behaviour change, apart from not expecting resources in multi-selections.

## Verification
- Select two own Ungroupable buildings (e.g. via control group): Tab switches the command panel between them.
- With units selected, Shift-click an enemy: the selection becomes just that enemy.
