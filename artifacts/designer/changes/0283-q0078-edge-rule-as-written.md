# 0283 — Tunnel near the map edge: the rule as written (verification in 0242 was a slip)

- **Date**: 2026-11-26T08:30:00-05:00
- **Answers**: Q0078
- **Docs**: `factions/syndicate/tunnels.md > Tunnel Area Non-Overlap Rule` (unchanged)

## What changed and why
Option **A**. The docs already answer it: every cell of the Tunnel Area must be on the map and Drillable, so a Tunnel needs as many tiles to the edge as its area's radius.
- **Tier 1:** a footprint 2 tiles from the edge is valid, and 1 tile is red.
- **Upgrades:** 3 tiles for Tier 2, 4 for Tier 3.

0242's verification line ("2 tiles from the map edge is red") was a slip.

## Implementation impact
None; as built.

## Verification
As built.
