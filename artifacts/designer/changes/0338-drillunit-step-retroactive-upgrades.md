# 0338 — All Cults stat upgrades apply to existing units; the DrillUnit takes the 90° sidestep

- **Date**: 2026-11-26T21:05:00-05:00
- **Answers**: Q0128 (point 2)
- **Docs**:
  - `core/tech_and_research.md > Faction Exceptions` (the Cults retroactivity exception removed);
  - `factions/cults/structures/{gunworks,temple,machine_shop}.md`;
  - `systems/units.md > Ground Collision`.

## What changed and why
User:
- **"All stat upgrades apply to existing units."**
  - The Cults' "not retroactive" exception is removed entirely, following 0336, which did this for augmentation only.
  - The Cults follow shared rule 1 like everyone else.
  - Staffing by Recruits stays their one research exception.
- **The Subterranean Transport is tracked when above ground, so it steps sideways.** DrillUnits take 0323's 90° body-width step.

## Implementation impact
- No "trained after" bookkeeping for Cults upgrades.
- The DrillUnit's step data matches TrackedVehicle.

## Verification
- When a future Gunworks upgrade completes, Militants already on the field get it.
