# 0091 — GDO build area is square

- **Date**: 2026-11-24T03:45:00-05:00
- **Answers**: Q0026 (point 1; points 2–4 still open)
- **Docs**: `factions/gdo/structures.md > GDOBuildArea` (shape)

## What changed and why
The user chose A (square). Each building adds its footprint, grown by its BuildRadiusExtension on every side, with the diagonals included. The starting DC (4x4, 12) gives a 28×28 square. The reason: the game's areas are square on the square grid, like the Tunnel Area and the Recruitment Area.

## Implementation impact
- Matches the Director's proposal.

## Verification
On an empty map, the starting DC's build area overlay is exactly 28×28 tiles centred on the DC.
