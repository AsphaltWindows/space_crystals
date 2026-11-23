# Q0013: Selection edge cases the docs don't cover

- **Date**: 2026-11-23T18:21:00-05:00
- **Blocking**: no. Selection is built with the behaviour listed under each point. Each one is a single named rule in the code.
- **Docs**: `systems/controls/selection_and_hud.md` (Selection, Constraints, BoxSelection priority tiers, MapClickSelection, ControlGroups, SelectionPanel, InfoPanel)

## Question
Please confirm or correct each of these:
1. **Ungroupable objects and ActiveGroup.** ActiveGroup is typed as an object type. Objects that aren't groupable need a group each. Current behaviour: each such object is its own group, and Tab cycles through them.
2. **Allies in box selection.** There are no teams yet, so the priority tiers don't place allied objects. Current behaviour: allies are treated like enemies, so they're never mixed into an own-units selection.
3. **Modified clicks on enemy or neutral objects.** Shift-click, Ctrl-click and double-click on them are unspecified. Current behaviour: the object is selected alone, as with a plain click.
4. **Clicks and boxes on empty ground.** Current behaviour:
   - A plain click or an empty box clears the selection.
   - A Shift- or Ctrl-click on empty ground does nothing.
   - A box ignores modifiers: Shift-box doesn't add to the selection.
   - Ctrl wins over Shift on a Ctrl+Shift click.
5. **InfoPanel with 2+ objects.** Current behaviour: it shows the ActiveGroup's type, how many are in that group and the total selected.
6. **SelectionPanel wording.** Its text says "2+ units" while its table says "2+ objects". Current behaviour: "objects", so it applies to resources too.
7. **Control groups and other players' objects.** Current behaviour: an object you don't own can't be added to a control group.

## Context
These came up while building selection (M1). None blocks anything, but each is something the player sees. Where the docs are silent, the implementation copied StarCraft II, except for point 4's Shift-box.

## Options
For each point: the current behaviour (A), or a different rule you give. For point 4, the main alternative (B) is that Shift-box **adds** the boxed units to the current selection, as in StarCraft II.
