# 0348 — Cults production buildings: leaving units go to the rally, exit in ready order; production buildings are Ungroupable

- **Date**: 2026-11-26T22:40:00-05:00
- **Answers**: Q0105 (points 3–5; 1–2 in 0332)
- **Docs**:
  - `factions/cults/structures/README.md` (Leaving; Not groupable);
  - `factions/cults/structures/{armory,cathedral,chop_shop,airstrip}.md`, `factions/cults/augmentation.md` (Groupable false).

## What changed and why
User:
3. Ejected and released units walk to the RallyPoint (default).
4. Units come out in the order they are ready (default).
5. **Cults production buildings aren't groupable.** The Armory, Altar, Cathedral, Chop Shop and Airstrip become Groupable false, so 0083's shared ordering doesn't apply to them.

Research buildings, the Shrine (already Ungroupable), Storage and Garrison are unchanged.

## Implementation impact
Selection grouping for those five.

## Verification
- Box-selecting two Armories gives one panel at a time (Tab between them).
