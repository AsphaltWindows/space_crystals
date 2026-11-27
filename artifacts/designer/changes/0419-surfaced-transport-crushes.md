# 0419 — The surfaced Subterranean Transport crushes

- **Date**: 2026-11-26T17:45:00-05:00
- **Answers**: Q0153 part 4 (Q0153 is now fully answered: 0417, 0418, 0419)
- **Docs**: `systems/units.md > UnitCollision > Crushing`; `factions/syndicate/units/subterranean_transport.md > Above-ground mode`

## What changed and why
The user chose (b): a DrillUnit in its above-ground mode crushes like a TrackedVehicle (including the spin rule, 0418). Underground it never crushes.

## Implementation impact
Replaces the placeholder (a). Add the surfaced DrillUnit to the crusher list.

## Verification
- Drive a surfaced Transport through enemy Recruits: they die; own infantry block it.
- Burrowed, it passes under enemy infantry with no effect.
