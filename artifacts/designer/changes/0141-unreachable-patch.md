# 0141 — A patch the Drone can't reach counts as occupied

- **Date**: 2026-11-24T17:30:00-05:00
- **Answers**: Q0044 (point 4; Q0044 done)
- **Docs**: `factions/colonists/units.md > Patch Occupancy (one Drone per patch)` (A patch it can't reach)

## What changed and why
Option C. An unreachable patch is handled like an occupied one: redirect to another reachable free patch in range; otherwise wait as close as possible and re-check at the waiting interval (16 frames placeholder, 0132); continue when the way clears. Never gives up permanently.

## Implementation impact
Replaces "retry forever" with the occupied-patch path; reachability check feeds the redirect search (unreachable patches are skipped).

## Verification
Wall off a patch with idle tanks: the Drone switches to another free patch; if none, it waits near the wall and resumes once a tank moves.
