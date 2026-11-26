# 0353 — Vehicles: idle vehicles don't yield; infantry yield to them; reverse only on order; forward loop to points behind

- **Date**: 2026-11-26T23:10:00-05:00
- **Answers**: Q0119
- **Docs**: `systems/units.md > Ground Collision > Who yields` (Vehicles)

## What changed and why
User accepted the defaults:
1. An idle vehicle doesn't yield, and own idle infantry yield to vehicles.
2. Vehicles reverse only on W (Reverse).
3. A point inside the turning circle is reached by a forward loop.

## Implementation impact
As proposed.

## Verification
- A Jackal ordered just behind itself loops forward.
- Infantry idle in its path step aside.
- An idle Jackal bumped by infantry stays put.
