# 0107 — Economy reference check: node geometry

- **Date**: 2026-11-24T12:10:00-05:00
- **Answers**: Q0038 (point 2)
- **Docs**: `factions/gdo/economy.md > Economy Targets (GDO vs Colonists)` (Reference geometry paragraph); `factions/colonists/units.md > Patch Occupancy (one Drone per patch)` (example wording)

## What changed and why
Option A. The reference check's "Egress Node 2 grid units from the row" means: the node sits on the row's long side, centred along it, with a 2-grid-unit gap of open ground between the node's edge and the edge of the row's middle patches (edge to edge). Centre to centre was never meant (it leaves a 32 su gap, narrower than a Drone).

## Implementation impact
Oracle / test-scenario layout only: 8 adjacent patches in a row, 2x2 Egress Node centred on the long side, 128 su edge-to-edge gap. No gameplay rule changes.

## Verification
The Colonists oracle scenario places the node as described; Drone leg length between the node and the middle patches is ~2 grid units of travel.
