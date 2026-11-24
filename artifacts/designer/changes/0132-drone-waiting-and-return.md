# 0132 — Drone waiting, re-checking, and returning to the last mined patch

- **Date**: 2026-11-24T16:25:00-05:00
- **Answers**: Q0037 (point 6, partly: where; the queue order among waiting Drones is still open)
- **Docs**: `factions/colonists/units.md > Patch Occupancy (one Drone per patch)` (Waiting and re-checking; Returning to the last mined patch)

## What changed and why
User's rules:
1. At an occupied patch with no free patch in range, the Drone waits **right at the patch it tried to mine** (within MiningReach) and mines it when it frees.
2. While waiting it **re-checks periodically** (16 frames placeholder) for a nearby freed patch and goes there if one appears.
3. If that patch is occupied again on arrival, it starts over (step 1) from there.
4. After each delivery a Drone returns to the patch where it **last successfully mined** (its gather target follows where it actually mined).

## Implementation impact
- Waiting Drones are not static: they may leave for a freed patch; several may race for it, and losers restart step 1 at that patch.
- Gather target updates to the actually-mined patch after each successful mining.
- Designer's reading, flag if wrong: "step 1 from there" = the Drone treats the patch it raced to as the one it tried (redirect again, or wait at that patch).

## Verification
- Overcrowd one patch: extra Drones wait beside it; when a neighbouring patch frees, a waiting Drone moves over within ~1 s.
- A Drone redirected from patch A to patch B mines B, delivers, and then returns to B, not A.
