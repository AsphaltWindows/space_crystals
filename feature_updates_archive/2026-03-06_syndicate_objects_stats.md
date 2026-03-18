# Feature Update: Syndicate Objects - Stats & Mechanics

**Date**: 2026-03-06
**Feature file**: `features/syndicate_objects.md`
**Design sources**: `design/syndicate_objects.md`
**Design update**: `design_updates/2026-03-06_tunnel_stats_and_mechanics.md`

## Modifications

### Tunnel Structure
- **SymmetryType**: Changed from AAAA to ABCD. Each side has a distinct function (A: unit entrance/exit, B: crystal drop-off, C: supply drop-off, D: back wall).
- **SightRange**: Set to 5 (was TBD).
- **Stats by tier now specified**:
  - HP: 600/800/1000
  - PointArmor: 1 (all tiers)
  - FullArmor: 16 (all tiers)
  - Tunnel Space: 20/30/40
- **Drop-off rule**: Per-side bottleneck — one Agent per side at a time. Crystal and supply drop-offs can occur simultaneously on different sides.

### Cost Scaling (was TBD, now formalized)
- **Construction**: cost = current Tunnels owned (Supplies). Linear scaling, starts at 0 for first Tunnel.
- **Upgrade to T2**: 2 + 2 x (T2+ Tunnels owned). Higher-tier counts toward lower-tier scaling.
- **Upgrade to T3**: 3 + 3 x (T3 Tunnels owned).

### Construction Time
- 480 frames (30 seconds). Agent must be present for duration.

### Starting Condition
- Player starts with 1 Tier 1 Tunnel + 1 pre-built Headquarters expansion.

### Tunnel Expansions (new section)
- Underground buildings within Tunnel Area. Invisible without detection, walkable by surface units.
- All Syndicate units produced by expansions. Rally point determines emerge vs. stay in network.
- **Headquarters**: T1 expansion, produces Agents and Guards, not unique (can build multiples).

### Open Questions Updated
- Removed resolved items (HP/armor/sight, Tunnel Space, cost formulas).
- Added: Tunnel upgrade duration, Headquarters size/HP/cost, Guard unit spec.
