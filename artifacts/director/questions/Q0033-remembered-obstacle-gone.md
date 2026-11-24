# Q0033: Path knowledge: when a remembered obstacle turns out to be gone

- **Date**: 2026-11-24T00:22:56-05:00
- **Blocking**: no. M2-E is built with the current behaviour listed under each point.
- **Docs**: `systems/controls/behaviors.md > PathKnowledge`; changes 0030 (group destinations resolved when the order is given), 0040, 0041, 0042.

## Question
Units now plan their paths only from what their player knows. A building or patch the player has seen stays a wall in their plans until they see it gone, even if it was destroyed or mined out meanwhile. Three cases where that wall turns out to be gone are open:

1. **A destination inside or behind a remembered obstacle.** Per 0030, the destination is settled when the order is given, from the player's knowledge. A click inside a remembered building's footprint therefore becomes the nearest reachable spot on the near side.
   - Later the unit sees the building is gone. Should it carry on to the point that was clicked?
   - **Current behaviour:** no. It stops at the near side.
2. **A shortcut that opens up.** A unit is walking around a remembered wall and sees that the wall is gone.
   - Should it re-plan and take the shorter way through?
   - **Current behaviour:** it keeps its planned route. It only cuts a corner if a later waypoint comes into its line of sight.
3. **Structures placed on the map.** "Map-placed objects are known from the start" is read as resources only (patches and stations). This matters only for Custom maps, which are deferred.
   - Should a structure pre-placed on a map also be known from the start?
   - **Current behaviour:** it is known once seen.

## Context
M2-E (path knowledge, 0040–0042) is built and merged. Re-routing on discovering a new obstacle works: a unit walking at a gap that turns out to be walled re-routes within a few ticks. These three points are the opposite case, where knowledge goes from blocked to open, which the docs don't cover.

## Options
- **Point 1:**
  - A (current): the destination stays as resolved.
  - B: the unit keeps the clicked point and continues there once it sees the way open.
- **Point 2:**
  - A (current): no re-plan.
  - B: re-plan when a remembered obstacle on its route is seen gone. This costs a path search per affected unit, which is cheap.
- **Point 3:**
  - A (current): resources only.
  - B: all map-placed objects.
