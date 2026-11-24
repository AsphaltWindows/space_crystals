# 0170 — Drop-off destroyed mid-trip: switch at once

- **Date**: 2026-11-24T21:35:00-05:00
- **Answers**: Q0046
- **Docs**: `factions/colonists/units.md > Assigned Drop-off` (Destroyed mid-trip)

## What changed and why
Option A (as built). A Drone whose drop-off is destroyed while it heads there switches immediately to the nearest own Egress Node, or idles holding its load if none. 0135's "check at the start of the return trip (not continuously)" was designer wording, not a user rule; it's superseded for this case. Detection method is the Director's choice; the user wants no per-frame cost if avoidable (e.g. react to the destruction event).

## Implementation impact
Behaviour as built; consider an event-driven reassignment instead of a per-tick existence check.

## Verification
Destroy the node while Drones are returning: they turn toward the nearest other node immediately.
