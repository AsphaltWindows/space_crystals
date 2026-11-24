# Q0050: The ActiveGroup when an unfinished group's last building completes

- **Date**: 2026-11-24T16:58:55-05:00
- **Blocking**: no. M3-U is built with the current behaviour below.
- **Docs**: `systems/controls/selection_and_hud.md > SelectionGroup` (Unfinished buildings form their own group; 0185), `> ActiveGroup`.

## Question
Under 0185, a building under construction or Warp In forms its own SelectionGroup, and joins the finished group of its type when it completes.
- If that unfinished group is the ActiveGroup and its **last** building completes, the group disappears. Which group becomes active?
- And if the ActiveGroup is another group, does a completion ever change it?

**Current behaviour:**
- The ActiveGroup stays as long as it exists, so other completions don't change it.
- If a completion empties the active unfinished group, the group the building joins (its finished group) becomes active, and the command panel resets to that group's DefaultState. Any targeting in progress for the unfinished group ends.

## Context
This came up while building 0185 (M3-U). The docs cover selection changes made by the player, and by targeting. They don't cover a group that disappears because its members changed type.

## Options
- A, as now: the finished group the building joined becomes active.
- B: the first group in the SelectionPanel's order becomes active.
- C: the same rule as when the ActiveGroup's last unit dies, if the docs define one.
