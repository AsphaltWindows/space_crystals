# 0137 — Plates vs. Drones on the same patch

- **Date**: 2026-11-24T17:05:00-05:00
- **Answers**: Q0044 (point 2); Q0037 (point 11)
- **Docs**: `factions/colonists/units.md > Patch Occupancy (one Drone per patch)` (Patches under a GDO Extraction Plate)

## What changed and why
- Q0044 pt 2, option A: a Drone mining a patch doesn't block GDO plate placement. Once plated, the patch counts as gone for Drones (like depletion, 0134); a Drone mid-mining is interrupted and collects nothing (0130). GDO can take contested patches by plating.
- Q0037 pt 11 (follows from right-click resolution and 0131): Drones can't gather a plated patch, and it's never a redirect target.

## Implementation impact
Matches the current build. Plate placement validity ignores mining Drones beside the patch (units *on* the footprint still block, 0094).

## Verification
Plate a patch a Colonists Drone is mining: the plate goes down, the Drone stops mid-cycle with no load and moves to another patch (or waits/ends per the rules).
