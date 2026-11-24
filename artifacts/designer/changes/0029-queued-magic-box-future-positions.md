# 0029 — Shift-queued group orders build the magic box from planned positions

- **Date**: 2026-11-23T18:56:56-05:00
- **Answers**: Q0015 (point 3; Q0015 now fully answered by 0027–0029)
- **Docs**: `systems/controls/commands.md > Group Destinations (Magic Box)` (queued orders bullet + fallback)

## What changed and why
User chose option B. For a Shift-queued order, each unit's box position and offset come from the **destination of its last queued movement order**, i.e. where it will stand when the new order starts, not where it stands now. Queuing "to X, then back home" therefore returns the group in formation. After a queued converge, the future box is tiny, so the next queued leg moves the blob on as a blob.

**Fallback (my resolution of an edge case, flagged to the user):** a unit whose queue has no move ending at a known location (empty queue, or its last queued order targets an object) uses its current position.

## Implementation impact
- **Replaces** the Director's assumption A (current positions for queued orders).
- Needs per-unit "planned end position" = destination of the last queued location-targeted move (Move, Attack Move, Patrol endpoint?).
- Patrol as the previous queued order: Patrol never completes, so a queued order after a Patrol won't execute anyway. Use the fallback.

## Verification
- Line up 4 units horizontally. Shift-right-click far north, then Shift-right-click back at the starting spot: the line goes north and returns as the same line, not a blob.
- Shift-right-click inside the box (converge), then Shift-right-click far away: the blob moves there as a blob.
