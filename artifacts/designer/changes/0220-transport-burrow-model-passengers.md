# 0220 — Subterranean Transport: burrow/surface follow vehicle deploy; passengers die with it

- **Date**: 2026-11-25T06:15:00-05:00
- **Answers**: —
- **Docs**: `factions/syndicate/units.md > Subterranean Transport` (Burrowing and surfacing work like vehicle deploy; Destroyed with passengers)

## What changed and why
User:
- Burrow and Surface follow the vehicle deploy model (0211, 0213, 0214):
  - each takes time and is a queueable order;
  - it can't be cancelled once started;
  - the mode changes at completion;
  - the last order given meanwhile is held, and impossible ones are dropped.
- If the transport is destroyed, all infantry inside die.

## Implementation impact
Concept only; times and capacity TBD.

## Verification
—
