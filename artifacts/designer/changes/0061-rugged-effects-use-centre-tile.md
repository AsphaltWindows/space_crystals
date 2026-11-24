# 0061 — Rugged effects use the tile under the unit's centre

- **Date**: 2026-11-23T20:30:33-05:00
- **Answers**: Q0014 (point 3; Q0014 now fully answered by 0059–0061)
- **Docs**: `systems/units.md > LightInfantry > RuggedTerrainDefenseBonus` (which tile counts)

## What changed and why
Option A. A unit counts as on Rugged terrain when the tile under its **centre** is Rugged. This applies to the infantry defense bonus and to the HoverVehicle double drag (0039). Passability (0039) is separate: no overlap at all for units that can't enter Rugged.

## Implementation impact
Matches the current behaviour.

## Verification
- A Drone slows down only once its centre crosses into a Rugged tile.
