# 0134 — Depleted patch: walking Drones redirect on arrival

- **Date**: 2026-11-24T16:40:00-05:00
- **Answers**: Q0037 (point 7); Q0044 (point 5)
- **Docs**: `factions/colonists/units.md > Patch Occupancy (one Drone per patch)` (A patch that runs out)

## What changed and why
A depleted patch counts as occupied for the redirect rule. Waiting Drones search immediately. Drones walking to it continue to its location and search **on arrival** (Q0044 pt 5: B, replacing the current "redirect at once"), because the search radius is centred on the Drone: searching from near the node could miss the field and end the Gather wrongly. No patch found → Gather ends (0123: deliver, then idle/next queued).

## Implementation impact
Changes the current build's immediate redirect. The destination for a walking Drone stays the vanished patch's location.

## Verification
Deplete a patch while a Drone is leaving the node toward it: it walks all the way to the empty spot, then moves to a neighbouring patch. Deplete the last patch: Drones arrive, find nothing, deliver any load and idle.
