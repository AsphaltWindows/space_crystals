# Q0031: A Move or Follow aimed at a selected unit itself

- **Date**: 2026-11-23T22:53:40-05:00
- **Blocking**: no. Until this is answered, the unit follows itself forever: its order never ends.
- **Docs**: `systems/controls/commands.md` (Move, Follow, right-click on a friendly unit); changes 0032 and 0062.

## Question
The player right-clicks one of the units in their own selection, or presses Move and clicks one of them. The other selected units follow it as usual. What does the clicked unit itself do?

## Context
M1-G3 found that a unit ordered to Move onto itself follows itself and never finishes. The design doesn't cover a unit being the target of its own order.

## Options
- **A** (proposed): the clicked unit is left out of the order and keeps its current orders. This matches how units that can't carry out an order are left out (0075).
- **B**: the clicked unit stops. Its order ends at once, as if it had arrived.
- **C**: a click on one of the selected units counts as a ground click at that spot. Everyone moves there, and the clicked unit takes part in the magic box.
