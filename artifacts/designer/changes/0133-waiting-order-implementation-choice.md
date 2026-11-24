# 0133 — Which waiting Drone mines next: implementation's choice

- **Date**: 2026-11-24T16:30:00-05:00
- **Answers**: Q0037 (point 6; with 0132, done)
- **Docs**: `factions/colonists/units.md > Patch Occupancy (one Drone per patch)` (Waiting and re-checking, item 4)

## What changed and why
The user doesn't mind which waiting Drone gets a freed patch: use the cheapest deterministic rule.

## Implementation impact
Director's choice (e.g. fixed entity order). Must stay deterministic.

## Verification
Same setup replayed twice gives the same Drone mining next.
