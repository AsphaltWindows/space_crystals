# 0128 — Gather while carrying a load

- **Date**: 2026-11-24T16:00:00-05:00
- **Answers**: Q0037 (point 2)
- **Docs**: `factions/colonists/units.md > Gather while carrying a load` (new)

## What changed and why
User's rule (not the proposed "deliver first"). A loaded Drone ordered to Gather goes straight to the new patch/refinery, skips mining on arrival (doesn't occupy the patch), and returns to deliver as if it had just mined the load there; then it continues the normal loop.

## Implementation impact
Arrival at the pick-up point with a non-empty load = treat as pick-up complete. No occupancy is taken.

## Verification
Right-click a far patch with a loaded Drone: it drives to the patch, turns around without mining, delivers, then mines that patch on the next trip. A Drone mining that patch is not displaced.
