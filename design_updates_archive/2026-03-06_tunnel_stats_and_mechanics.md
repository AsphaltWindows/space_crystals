# Design Update: Tunnel Stats & Mechanics

**Date**: 2026-03-06
**Files modified**: `design/syndicate_objects.md`, `design/entities.md`, `design/gdo_objects.md`

## Changes

### Tunnel (syndicate_objects.md)

- **SymmetryType**: Changed from AAAA to ABCD. Each side has a distinct function:
  - Side A: Unit entrance/exit
  - Side B: Crystal drop-off (1 Agent at a time)
  - Side C: Supply drop-off (1 Agent at a time)
  - Side D: Back wall (no function)
- **Stats added**: HP 600/800/1000 by tier, PointArmor 1, FullArmor 16, SightRange 5, Tunnel Space 20/30/40 by tier
- **Cost formulas formalized**:
  - Construction: cost = current Tunnels owned (Supplies)
  - Upgrade to T2: 2 + 2 x (T2+ Tunnels owned)
  - Upgrade to T3: 3 + 3 x (T3 Tunnels owned)
  - Higher-tier Tunnels count toward lower-tier cost scaling
- **Construction time**: 480 frames (30 seconds), Agent must be present
- **Starting condition**: Player starts with 1 Tier 1 Tunnel + 1 pre-built Headquarters
- **Drop-off rule**: Per-side bottleneck — crystal and supply drop-offs operate independently

### Tunnel Expansions (syndicate_objects.md)

- **New section**: Tunnel Expansions defined as underground buildings within Tunnel Area
- **Headquarters**: T1 expansion, produces Agents and Guards, not unique (can build multiples)

### Building Placement Flipping (entities.md, gdo_objects.md)

- **System-wide change**: All structure placement now supports rotation AND flipping (horizontal/vertical axis)
- Structure Instance gains FlipHorizontal and FlipVertical boolean properties
- AwaitingPlacement visual feedback updated: F flips horizontally, Shift+F flips vertically
