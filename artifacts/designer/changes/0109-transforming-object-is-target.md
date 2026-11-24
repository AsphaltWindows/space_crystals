# 0109 — Deploy / Warp In: the object is the target structure from the first frame

- **Date**: 2026-11-24T12:30:00-05:00
- **Answers**: Q0039 (point 1; points 2–8 still open)
- **Docs**: `factions/colonists/economy.md > Beacon Deployment > A Beacon from the first frame` (new); `factions/colonists/economy.md > Warp In > The target building from the first frame` (new)

## What changed and why
Option A. When a Drone starts deploying, it immediately becomes a Beacon under deployment: a structure on its 2x2 footprint (claimed at once; units can't enter; can't move; treated as a structure by targeting/auto-targeting/placement). It is the same object (stays selected and in control groups). Cancel turns it back into the Drone at the centre of the footprint.

Warp In works the same way: the object is the target building under Warp In from the first frame, with its full footprint claimed at once; cancel shrinks it back to the Beacon on the Beacon's original footprint.

## Implementation impact
Matches plan decision T15 (one object whose kind changes at the start). No completion-time space re-check is needed.

## Verification
- Order a Drone to deploy; at the first deploy frame, try to walk a unit onto the 2x2: it's blocked. The selection still shows the object.
- Enemy auto-targeting treats it as a structure during the 3 s.
- Cancel: a Drone appears at the footprint centre.
- Warp In a 3x3 (or larger) building: the whole footprint is solid from the first frame; cancel restores the 2x2 Beacon.
