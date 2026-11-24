# Q0046: A Drone's drop-off is destroyed while it is on its way back

- **Date**: 2026-11-24T13:40:39-05:00
- **Blocking**: no. M3-H3 is built with option A below.
- **Docs**: `factions/colonists/units.md > Assigned Drop-off`; change 0135.

## Question
0135 says: "At the start of the return trip, a missing AssignedDropOff is replaced by the nearest own Egress Node. ... Check happens when the return trip starts (not continuously)." What should a Drone do if its drop-off is destroyed **after** the return trip has started, while the Drone is on its way back?

## Context
Building 0135 (M3-H3), the check at the start of the trip doesn't cover a node lost mid-trip. The current build re-checks every tick of the return, so the Drone turns to the nearest remaining node at once. That goes against 0135's "not continuously", so it needs your answer.

## Options
- **A, as built:** the Drone turns to the nearest remaining own Egress Node as soon as its node is gone. It idles holding its load if there is none.
- **B:** it carries on to the destroyed node's spot, and applies 0135's replacement rule when it arrives, as if a new return trip started there.
- **C:** it idles where it is, holding its load, until given an order.
