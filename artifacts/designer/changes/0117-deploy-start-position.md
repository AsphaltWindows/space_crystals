# 0117 — Deploy: where the Drone starts

- **Date**: 2026-11-24T13:50:00-05:00
- **Answers**: Q0040 (point 1; other points open)
- **Docs**: `factions/colonists/economy.md > Beacon Deployment > Deploy Command`

## What changed and why
The Director's proposal. The Drone moves to the centre of the footprint and deploys on arrival. If blocked en route while overlapping the footprint, it deploys where it stands; if it stops without overlapping, the order fails and it idles. The Beacon is placed on the ghost's footprint regardless of the Drone's exact position. The user noted the blocked case rarely matters (a blocker on the footprint invalidates the site anyway).

## Implementation impact
Arrival = centre reached, or stopped-while-overlapping. Placement validation on arrival as before (Worker-Built Structures).

## Verification
- Deploy on open ground: the Drone drives to the middle of the ghost and deploys; the Beacon matches the ghost.
- Block the approach with a unit just outside the footprint while the Drone's edge overlaps it: it deploys.
- Wall the Drone off before it reaches the footprint: order fails, Drone idles.
