# 0411 — A box still selects one building, groupable ones included

- **Date**: 2026-11-26T16:15:00-05:00
- **Answers**: Q0151 part 2 (Q0151 is now fully answered: 0407, 0409, 0411)
- **Docs**: `systems/controls/selection.md > BoxSelection`; `factions/cults/structures/recruitment_hall.md > Groupable`

## What changed and why
The user chose (a): the box keeps its tiers, and a box with no own units selects **one** own building, even when the buildings are groupable. Several Halls are selected with Shift-click, Ctrl-click or double-click (now on buildings, 0410), or Shift + box.

## Implementation impact
None for the box (as built). 0395's Verification step "Box-select 2 Halls" should read "Double-click a Hall with 2 on screen (or Shift-click the second)".

## Verification
- Box over 2 Halls: one Hall selected.
- Double-click one of them: both selected, one group; set a rally point: both use it.
