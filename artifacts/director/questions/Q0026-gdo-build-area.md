# Q0026: GDO build area: shape and what happens when buildings are lost

- **Date**: 2026-11-23T22:18:49-05:00
- **Blocking**: no. M2 builds the proposals. Please answer point 1 before the test map's layouts are finalised.
- **Docs**:
  - `factions/gdo/structures.md > GDOBuildArea`;
  - `factions/gdo/structures.md > DeploymentCenter` (its description calls it the "build radius");
  - BuildRadiusExtension of each GDO building;
  - `factions/gdo/economy.md > Opening Economy Baseline` ("8 patches in GDO build range").

## Question
1. **Shape.**
   - The area "expands outward from the placed building by that building's BuildRadiusExtension in all directions". It could be square (every tile within that many tiles of the footprint, diagonals included) or round (by distance from the footprint).
   - **Proposal:** square. With the DC's 12, the starting area is 28 × 28 tiles.
   - **Alternative:** round, which is closer to "radius".
2. **Losing buildings.**
   - Does the area shrink when a building, or the DC itself, is destroyed?
   - **Proposal:** the area is always the union of the current buildings' areas, so it shrinks.
   - **Alternative:** it only ever grows.
3. **No Deployment Center.**
   - The area is "seeded by each Deployment Center's and each Outpost's BuildRadiusExtension".
   - **Proposal:** with the DC gone, the other buildings' areas still count, so an EF can still place plates.
4. **What the overlay shows.**
   - The overlay shows "all valid build cells".
   - **Proposal:** it shows the build area only. Whether a given tile is valid (slope, fog, occupied) shows on the ghost.

## Context
The build area is built in M2. It decides where buildings and plates can go and how many patches a starting base reaches. The reference base assumes 8.

## Options
- **Point 1:** A, square; B, round.
- **Point 2:** A, follows the current buildings; B, only grows.
- **Point 4:** A, the area only; B, the area minus tiles that are invalid right now.
