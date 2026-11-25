# 0239 — Underground expansions need Drillable tiles

- **Date**: 2026-11-25T09:50:00-05:00
- **Answers**: Q0062 point 3a, 3b, 3e (3c, 3d and points 4–5 pending)
- **Docs**: `factions/syndicate/tunnels.md > Tunnel ObjectInterfaceState > AwaitingPlacement (Expansion)`

## What changed and why
User: keep the Drillable requirement (option A). Every tile under an expansion must be Drillable: Plane, Rugged and Cliff yes; Mountain and Water no. Own and neutral underground structures, and the Tunnel's own footprint, block it (as proposed).

## Implementation impact
As proposed for M4-F.

## Verification
An expansion ghost over a Water tile inside the Tunnel Area is red.
