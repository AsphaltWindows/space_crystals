# 0043 — Underground units don't collide with each other

- **Date**: 2026-11-23T19:41:05-05:00
- **Answers**: Q0011 (part 4, underground; air units still to confirm)
- **Docs**: `systems/units.md > UnitCollision > Underground Collision` (new)

## What changed and why
User decision, which **differs from the Director's default** (collide like ground units): underground units pass through each other freely.

## Implementation impact
Not needed for M1. Open: surfacing onto an occupied spot.

## Verification
(Later.) Two drill units underground crossing paths pass through each other.
