# 0390 — Building ghosts show entrance and exit arrows

- **Date**: 2026-11-27T12:20:00-05:00
- **Answers**: —
- **Docs**: `core/entities.md > Structure Type > Entrance and exit` (Arrows on the ghost); `systems/controls/hud.md > PointerDisplayType > Resolution Rules > AwaitingPlacement`

## What changed and why
User: "it would be nice to have arrows for entrances and exits for the building ghosts".
- While placing, a ghost shows an arrow at each doorway:
  - **pointing in** at an entrance;
  - **pointing out** at an exit;
  - **two-way** at a single door used both ways (Garrison, Bunker).
- The arrows sit at the doorway's position, and turn and flip with the ghost.
- This applies to every faction's buildings that have entrances or exits, e.g. Cults pool, tech and research buildings, and the GDO Barracks, Factory and Airfield exits.

## Implementation impact
- Client only: an arrow overlay on the ghost, from each building type's EntranceSide and ExitSide (or door side).
- The art is a placeholder until the art pass.

## Verification
- Place a Cults Armory: an inward arrow shows on side B and an outward arrow on side C. Rotating the ghost turns the arrows with it.
- A Garrison ghost shows one two-way arrow.
