# 0394 — Entrance and exit arrows also show on a selected placed building

- **Date**: 2026-11-27T12:45:00-05:00
- **Answers**: —
- **Docs**: `core/entities.md > Structure Type > Entrance and exit` (Arrows on the ghost)

## What changed and why
User: yes, the door arrows from 0390 also show on a **placed building while it is selected**, not only on ghosts.

## Implementation impact
- Client: draw the same arrows for a selected building's doorways.

## Verification
- Select an Armory: an inward arrow shows on its entrance and an outward arrow on its exit.
