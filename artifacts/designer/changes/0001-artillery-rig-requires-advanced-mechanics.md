# 0001 — Artillery Rig requires Advanced Mechanics

- **Date**: 2026-11-23T12:32:23-05:00
- **Answers**: —
- **Docs**: `factions/gdo/units.md > Artillery Rig` (description + new TechPrerequisite line), `factions/gdo/structures.md > Factory > Produces`, `factions/gdo/structures.md > Advanced Mechanics`

## What changed and why
The Artillery Rig, produced at the Factory, now has a unit prerequisite: the player must own at least one Advanced Mechanics. Before, it was available as soon as a Factory existed. Advanced Mechanics now unlocks two Factory units (Prospector and Artillery Rig), which fills out its role as the "more advanced vehicles" building.

## Implementation impact
- Factory production: the Artillery Rig is gated by the standard unit-prerequisite rule (`core/tech_and_research.md > Tech Prerequisites > Unit Prerequisites`), exactly like the Prospector.
- Factory units with no extra prerequisite: Raptor and Twister.
- All Artillery Rig numbers are still TBD.

## Verification
- With a Factory and no Advanced Mechanics, the Artillery Rig is shown as unavailable (prerequisite not met), while Raptor and Twister can be queued.
- After an Advanced Mechanics is completed, the Artillery Rig (and Prospector) can be queued.
- Losing every Advanced Mechanics makes them unavailable again, per the standard unit-prerequisite rule.
