# 0085 — A finished unit waiting inside is refunded if the building dies

- **Date**: 2026-11-24T03:00:00-05:00
- **Answers**: — (closes the TBD in 0084)
- **Docs**: `factions/gdo/structures.md > Barracks > Destroyed while producing`; `core/entities.md > Structure Type > Exit Doorway` (point 3 clarified)

## What changed and why
A unit that has finished production but can't come out, because the exit doorway is occupied, is **fully inside** the building. Its production shows 100%, and it doesn't exist on the map yet. If the building is destroyed, that unit is **refunded** with the rest of the queue (user). The unit already standing in the doorway is a normal unit and is unaffected.

## Implementation impact
- Treat the waiting finished unit as a queue entry for the destruction refund.
- The production progress display stays at 100% while the unit waits.

## Verification
1. Block a Barracks doorway with a unit stuck in it (e.g. build a structure flush against side B).
2. Let the next Peacekeeper reach 100%, then destroy the Barracks. Its cost is refunded.
3. The Peacekeeper that was in the doorway remains on the map.
