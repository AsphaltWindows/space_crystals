# 0025 — Unreachable move target: go to the nearest reachable point

- **Date**: 2026-11-23T18:49:50-05:00
- **Answers**: Q0008 (part 2; part 3 occupied target still open)
- **Docs**: `systems/controls/behaviors.md > MovingToLocation > Unreachable target location` (new), `systems/controls/commands.md > Group Destinations (Magic Box)` (unreachable offset destination)

## What changed and why
User chose option A. If a move target can't be reached (water, cliff, inside a structure, cut-off area), the unit goes to the **reachable point nearest the target** and completes there. Orders are never refused for being unreachable. The same applies to a magic-box offset destination that lands somewhere unreachable.

## Implementation impact
Matches the Director's default.

## Verification
- Right-click the middle of a lake: the unit walks to the nearest shore point and stops.
- Right-click a plateau with no ramp: the unit goes to the closest reachable spot below it.
- Move a line of units so that one unit's offset spot falls in water: that unit stops at the nearest shore; the others take their offset spots.
