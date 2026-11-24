# 0044 — Underground units can't surface onto occupied spots

- **Date**: 2026-11-23T19:41:33-05:00
- **Answers**: Q0011 (part 4, surfacing follow-up; air units still to confirm)
- **Docs**: `systems/units.md > UnitCollision > Underground Collision > Surfacing`

## What changed and why
User decision: surfacing is **not allowed** where the unit would overlap an above-ground unit. I extended this to structures and terrain impassable for its surface form (obvious cases, flagged to the user). The unit stays underground until surfaced somewhere clear. There's no automatic waiting and no relocation.

## Implementation impact
Later domain (not M1). The surface action validates the footprint and is refused if blocked.

## Verification
(Later.) Order a drill unit to surface under a standing Peacekeeper: refused, it stays underground. Move it a little: it surfaces.
