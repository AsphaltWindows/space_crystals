# 0144 — Robo Portal exit side

- **Date**: 2026-11-24T18:05:00-05:00
- **Answers**: Q0041 (points 3 and 4)
- **Docs**: `factions/colonists/structures.md > Robo Portal` (Which side)

## What changed and why
Option A. Robo Portal uses the Bio Portal's rule across all four sides: closest side to the RallyPoint; none/blocked → random unblocked side (deterministic). Closest = distance from each side's midpoint to the rally point (object centre); ties top, right, bottom, left. Same measure for the Bio Portal.

## Implementation impact
As proposed.

## Verification
Rally a Robo Portal to the east: Drones exit east. Block the east side: they exit from another, unblocked side. No rally: varied sides.
