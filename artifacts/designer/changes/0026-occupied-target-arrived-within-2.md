# 0026 — Occupied move target: arrived if stuck within 2 grid units behind non-moving units

- **Date**: 2026-11-23T18:51:23-05:00
- **Answers**: Q0008 (part 3; Q0008 now fully answered by 0024–0026)
- **Docs**: `systems/controls/behaviors.md > MovingToLocation > Occupied target location` (new)

## What changed and why
User chose option A with a larger radius. A unit that is stuck **within 2 grid units** (the Director proposed 1.5) of its destination, blocked by units that aren't moving, counts as **arrived**. It stops and completes without re-routing. This keeps converging groups from jostling at the back of the crowd. 2 grid units is a placeholder.

## Implementation impact
Same as the Director's default, but the radius is 2 grid units, not 1.5. It replaces the interim "arrived wherever it stops behind its group" stand-in mentioned in Q0012.

## Verification
- Converge 12 Peacekeepers on one point (click inside their box): the outer units stop within about 2 tiles of the point straight away, with no re-route attempts.
- A unit ordered to a point 3 tiles behind a wall of Hold Position units keeps trying (re-routes) and eventually gives up instead of counting as arrived.
