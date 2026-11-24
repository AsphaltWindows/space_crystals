# 0148 — Shields display

- **Date**: 2026-11-24T18:45:00-05:00
- **Answers**: Q0042 (point 4)
- **Docs**: `systems/combat.md > Shields > Display` (new)

## What changed and why
The Director's proposal: a shield bar directly above the HP bar on the map; "Shields current/max" next to HP in the InfoPanel. Styling deferred to the HUD pass.

## Implementation impact
As proposed.

## Verification
Select a damaged Legionnaire: two bars above it (shields on top); the InfoPanel shows both values.
