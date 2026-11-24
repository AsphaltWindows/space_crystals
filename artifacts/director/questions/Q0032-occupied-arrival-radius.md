# Q0032: Arrival radius behind an occupied destination: 2 or 3 grid units?

- **Date**: 2026-11-23T22:53:40-05:00
- **Blocking**: no. M1 keeps 2 grid units.
- **Docs**: `systems/controls/behaviors.md > MovingToLocation > Occupied target location`; change 0026.

## Question
0026 says a unit that is stuck within 2 grid units of its destination, blocked by units that aren't moving, counts as arrived. Should that radius be 3 grid units instead?

## Context
- 0026 set the radius to 2 grid units; the Director had proposed 1.5.
- **The problem.** In M1-G3's crowd tests, blocks of large units still lose a few members, for example a 5×5 block of Guards (36 su each) ordered through another group.
  - Crossing the other group reorders the block.
  - The last Guards to arrive then find their places walled in by their own group's earlier arrivals. Those don't make way (0027).
  - The late Guards stop just beyond 2 grid units and give up as Failed.
- **What M1-G3 already fixed.** Re-routes now go around the group's earlier arrivals. Failures across a sweep of 192 crowd layouts fell from 1.4% to 0.7%. What's left are places enclosed on every side, which no route can reach.
- **An experiment with 3 grid units.** In the same sweep, Guard losses fell from 43 to 17, and Peacekeeper-against-Peacekeeper losses from 12 to 0.

## Options
- **A**: keep 2 grid units. A few large units in big crowd crossings end as Failed, a few tiles short.
- **B**: 3 grid units. Fewer failures, but a unit may stop up to 3 grid units from its point when that area is crowded.
- **C**: scale the radius with the unit's size, for example 2 grid units plus the unit's own width.
