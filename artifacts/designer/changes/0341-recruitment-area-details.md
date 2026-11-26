# 0341 — Recruitment Area: whole area on the map, capacity rounded down, ties split by nearest Hall, players compete, nothing until completed

- **Date**: 2026-11-26T21:35:00-05:00
- **Answers**: Q0099
- **Docs**: `factions/cults/economy.md > RecruitmentHall > RecruitmentArea`, `> TileClaiming` (Claiming details)

## What changed and why
User:
1. **The whole 10x10 area must lie on the map.** A Hall that would leave it can't be placed (option c).
2. **Capacity is rounded down everywhere** (option b): 6.6 becomes 6, in production and on the HUD.
3. **The Hall completed first wins.** For a tie in the same frame, the user asked for "some naive way to split the contested portions contiguously". The designer's rule: each contested tile goes to the Hall whose centre is nearer, and at exactly equal distance to the lower object id.
4. A Hall under construction does nothing until it is complete (default).
5. **Different players' Halls compete** for tiles.

## Implementation impact
- Placement validation for Halls (area on the map).
- Floor of the capacity.
- The tie rule: nearest centre.

## Verification
- Placing a Hall 3 tiles from the edge is refused.
- A Hall with 33 claimed tiles stops at 6 Recruits, and the HUD shows 6.
