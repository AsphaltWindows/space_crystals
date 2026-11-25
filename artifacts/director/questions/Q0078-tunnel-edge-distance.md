# Q0078 — Tunnel near the map edge: 0242's verification is one tile off

- **Date**: 2026-11-25T01:47:32-05:00
- **Blocking**: no — proceeding on assumption: the rule as written. A Tier 1 Tunnel whose footprint is 2 tiles from the edge is valid, and 1 tile from it is red.
- **Docs**: `factions/syndicate/tunnels.md > Tunnel Area Non-Overlap Rule` (Drillable area), `> Tiers`

## Question
Is a Tier 1 Tunnel whose footprint stands 2 tiles from the map edge valid?

## Context
- **The rule** (0242): "a Tunnel can only be **placed** if every cell of its Tunnel Area is **on the map and Drillable** ... So **no Tunnel near the map edge** (closer than its area reaches)."
- **The area**: "The radius extends outward from the Tunnel's 4x4 footprint in each direction". At Tier 1 the radius is 2 (8x8).
- **The contradiction.** 0242's verification says "A Tunnel 2 tiles from the map edge is red." With the footprint 2 tiles from the edge, the area's outer ring lies on tiles 0 and 1, which are on the map, so the rule makes it valid. Only 1 tile from the edge puts an area cell off the map.
- M4-B builds the rule. Its tests say "1 tile from the edge: red; 2 tiles: valid", citing this question. M4-E (placement) and M4-G (upgrades: 3 tiles for Tier 2, 4 for Tier 3) follow.

## Options
- **A. The rule as written; the verification line was a slip.** A Tunnel needs as many tiles to the edge as its area's radius: 2 for Tier 1. Nothing to change in code; the verification line is corrected.
- **B. The verification as written: 2 tiles is too close.** The rule then needs a margin beyond the area (one more tile). That is a new rule; it shrinks every map's usable border, and B's layouts and the editor's checks gain a margin constant.
