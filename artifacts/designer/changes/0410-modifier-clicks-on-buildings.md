# 0410 — Shift-, Ctrl- and double-click work on own buildings

- **Date**: 2026-11-26T16:10:00-05:00
- **Answers**: — (raised by the user while discussing Q0151 part 2; that part is still open)
- **Docs**: `systems/controls/selection.md > MapClickSelection` (table and "Own buildings too"); `> BoxSelection > Shift + box (add)` and `> Empty clicks and boxes` (wording "own objects")

## What changed and why
The user found that Shift-click wouldn't add a second Recruitment Hall to a selected one (Shift + box did). The docs limited the modifier clicks to own **units**. The user chose (a): they now work on **all own buildings** too, one rule for everything.
- **Shift-click** own building: toggle it in the selection.
- **Ctrl-click / double-click** own building: select every own building of that type on screen.
- **Ctrl+Shift-click** own building: add every own building of that type on screen (never removes).
- Ungroupable buildings each get their own SelectionGroup (existing Constraints), so these are still useful for Tab-cycling.
- "Of that type": an unfinished building counts as another type than finished ones (existing SelectionGroup rule), so Ctrl-click on a finished Hall takes only finished Halls.
- Mixing units and buildings is allowed (existing Constraints: all-own selections have no limit on mixing). Non-owned rules unchanged.

## Implementation impact
New: the three modifier clicks accept own buildings. Plain left-click and the box rules are unchanged (Q0151 part 2, box-selecting several Halls, is still open).

## Verification
- Select a Hall, Shift-click a second Hall: both are selected, one group; Shift-click it again: removed.
- Double-click a Hall with three on screen (one under construction): the two finished ones are selected.
- Select some Recruits, Shift-click a Storage: Recruits and Storage selected, two groups.
- Ctrl-click an Armory (ungroupable) with two on screen: both selected, each its own group; Tab cycles them.
