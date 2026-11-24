# 0135 — No drop-off: nearest Egress Node, or idle with the load

- **Date**: 2026-11-24T16:45:00-05:00
- **Answers**: Q0037 (point 8)
- **Docs**: `factions/colonists/units.md > Assigned Drop-off` (Drop-off gone)

## What changed and why
The Director's proposal. At the start of the return trip, a missing AssignedDropOff is replaced by the nearest own Egress Node; with no Egress Node, the Drone idles holding its load.

## Implementation impact
Check happens when the return trip starts (not continuously).

## Verification
Destroy the node a mining group delivers to: on their next return they go to the nearest remaining node. Destroy every node: Drones idle with loads; warp in a node and right-click it to deliver.
