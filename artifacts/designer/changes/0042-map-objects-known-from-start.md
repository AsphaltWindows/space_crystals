# 0042 — Path knowledge: map-placed patches and stations are known from the start

- **Date**: 2026-11-23T19:40:26-05:00
- **Answers**: Q0011 (confirms the 0040 assumption; part 4, later domains, pending a clarification)
- **Docs**: `systems/controls/behaviors.md > PathKnowledge`

## What changed and why
User confirmed: crystal patches and Supply Delivery Stations are known from the start for pathing, like terrain. A patch depleted out of view stays an obstacle in the player's path knowledge until seen gone.

## Implementation impact
Matches the assumption in 0040.

## Verification
- A unit ordered across unexplored ground routes around crystal fields it has never seen.
- A patch mined out in the fog is still routed around until a unit sees its tile.
