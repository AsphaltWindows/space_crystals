# 0440 — Idle wheeled, tracked and surfaced drill vehicles don't yield; hover vehicles do

- **Date**: 2026-11-26T20:25:00-05:00
- **Answers**: Q0172 part 1 (Q0172 is now fully answered: 0439, 0440)
- **Docs**: `systems/units.md > UnitCollision > Ground Collision` ("Vehicles")

## What changed and why
The user chose (a): the "idle vehicle doesn't yield" rule covers **WheeledVehicle, TrackedVehicle and the DrillUnit above ground** (treads and wheels can't step sideways). The **HoverVehicle still yields**, as it can move in any direction. Standing vehicles also don't take the idle push-apart nudge.

## Implementation impact
None: matches the assumption for M5-V3.

## Verification
- An idle Hand of Heaven in a lane: own infantry moving through go round it; it doesn't budge.
- An idle Colonists Drone in a lane: it steps aside for own units.
