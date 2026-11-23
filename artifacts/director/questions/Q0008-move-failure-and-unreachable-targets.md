# Q0008: When a move fails, and moves to unreachable or occupied targets

- **Date**: 2026-11-23T15:46:30-05:00
- **Blocking**: no. Proceeding on the defaults below, each held in a named placeholder.
- **Docs**: `systems/controls/behaviors.md > MovingToLocation`, `> MovingToObject`, `> Failure`

## Question
1. "Repeatedly fails to make progress": what counts as failing, and after how long does the unit give up?
2. A move target can't be reached when it's on a cliff, in water, inside a structure, or in an area cut off from the unit. What happens?
3. A move target can be reachable but occupied by units that won't move. What happens then?

## Context
The behaviours say a move stops and completes when it repeatedly fails to make progress, but give no numbers. They also don't cover targets that can't be reached, or that are physically occupied.

## Options
**1. Giving up.** Default:
- A unit is stuck after 1.5 s (24 ticks) without getting closer.
- It then looks for another route, at most 3 times, waiting 0.5 s, 1 s and 2 s between tries.
- If that still fails, the order completes, about 7 s in the worst case.
- The alternative is shorter or longer limits.

**2. Unreachable target.**
- **A (default)**: go to the reachable point nearest the target and complete there.
- **B**: refuse the order (no movement), maybe with feedback.

**3. Occupied target.**
- **A (default)**: when a unit is stuck within 1.5 tiles of its target behind units that aren't moving, it counts as arrived.
- **B**: it keeps trying until the "giving up" rule ends the order.
