# 0038 — Crystal patches and Supply stations block ground units

- **Date**: 2026-11-23T19:23:06-05:00
- **Answers**: Q0011 (part 1; rugged passability, unseen-structure pathing and later domains still open)
- **Docs**: `systems/units.md > UnitCollision > Ground Collision`

## What changed and why
Option A. Patch and Supply Delivery Station footprints are solid for ground units. Workers mine from beside patches (8 su reach), and crystal fields act as natural walls. A depleted patch disappears and opens its tile, so paths can change mid-game (map makers can use this deliberately). This is consistent with plated patches, which block anyway as structures.

## Implementation impact
Matches the Director's default. Pathfinding must handle a patch vanishing on depletion (the tile becomes passable).

## Verification
- Units path around a crystal field. Mine a patch out: units can now walk through its tile.
- Units can't walk across a Supply Delivery Station.
