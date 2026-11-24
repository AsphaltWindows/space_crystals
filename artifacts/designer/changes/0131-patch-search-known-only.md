# 0131 — Patch search: known, uncovered patches only

- **Date**: 2026-11-24T16:15:00-05:00
- **Answers**: Q0037 (point 5; with 0123, done)
- **Docs**: `factions/colonists/units.md > Patch Occupancy (one Drone per patch)` (Search radius)

## What changed and why
Option A. Within the Drone's search radius (around itself, per-unit stat, 6 GU placeholder; 0123), only patches the player knows (visible or remembered) and not covered by a structure qualify. No information leak from fog.

## Implementation impact
Search reads the player's map memory; plates' patches are excluded (also relevant to Q0037 pt 11, still open).

## Verification
A patch within 6 GU that the player has never seen is never chosen; a remembered one is. A plated patch is never a redirect target.
