# 0140 — Right-click on the node with an empty Drone = Move

- **Date**: 2026-11-24T17:25:00-05:00
- **Answers**: Q0044 (point 3)
- **Docs**: `factions/colonists/units.md > Drop Off order` (Empty Drones)

## What changed and why
Follows from the existing right-click resolution (Drop Off only when carrying) and per-unit resolution (0063); confirmed by the user. An empty Drone right-clicking its own Egress Node gets Move to the node and stops there. Replaces the current build's "goes back to its patch".

## Implementation impact
Remove the "empty Drop Off returns to patch" path; there is no empty Drop Off order.

## Verification
Mixed group (some loaded, some empty) right-clicks the node: loaded ones deliver and idle; empty ones walk to the node and stop.
