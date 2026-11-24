# 0094 — Ground units on the footprint block direct placement

- **Date**: 2026-11-24T04:05:00-05:00
- **Answers**: Q0027 (point 1; points 2–4 still open)
- **Docs**: `core/entities.md > Placement Validation > Direct Placement` (new bullet)

## What changed and why
The user chose A. For Direct Placement (e.g. GDO buildings), a **ground unit of any player** (own, allied or enemy) standing on any footprint tile makes those tiles invalid. They show red on the ghost, and the building can't be placed until the units move. Own units are **not** pushed aside. Letting idle own units step aside was considered and judged unnecessary. Air and underground units never block placement.

## Implementation impact
- Matches the Director's proposal. Footprint validity checks ground unit silhouettes on the tiles.

## Verification
1. Stand an own Peacekeeper where a Generator would go. The ghost shows those tiles red, and placement is refused.
2. Move the Peacekeeper away. Placement succeeds.
3. A Hornet hovering over the spot doesn't block.
