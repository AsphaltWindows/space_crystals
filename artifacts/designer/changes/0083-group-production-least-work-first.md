# 0083 — Group production with limited resources: least work queued pays first

- **Date**: 2026-11-24T02:45:00-05:00
- **Answers**: Q0024 (point 3; points 4–7 still open)
- **Docs**: `systems/controls/selection_and_hud.md > SelectionGroup > No smart-casting > Group production with limited resources` (new)

## What changed and why
When a production command goes to several selected production buildings (no smart-casting: every building receives it) and the player can't pay for all of them:
- **As many as possible** queue the unit.
- Buildings are served **least work queued first**. The Designer's reading of "work" is the total remaining production time in the queue, counting the unit currently in production.
- Ties are broken in a fixed, deterministic order.
- The others don't queue, and the player gets insufficient-resources feedback.

The user's reasoning: available resources get spent, and they go to the idlest buildings first. This applies to every faction and every cost, including crystals, GDO Unit Control and anything else.

## Implementation impact
- This replaces the "fixed order" in the Director's proposal.
- Order of evaluation per press: sort the receiving buildings by remaining queued production time, ascending, then queue in that order until a cost check fails. Each building's check includes everything the earlier buildings in this press have already spent.
- **Tie-break:** any deterministic order, e.g. entity id. The simulation must stay deterministic.
- **Placeholder interpretation:** "least work" is measured as remaining production time, not queue length. If the user prefers queue length, only the sort key changes.

## Verification
1. Select 3 Barracks:
   - Barracks X: an empty queue.
   - Barracks Y: 1 Peacekeeper nearly finished.
   - Barracks Z: 3 Peacekeepers queued.

   With crystals for exactly 2 Peacekeepers, press Q. X and Y queue one each, and Z doesn't.
2. With enough crystals for all, every selected Barracks queues one.
