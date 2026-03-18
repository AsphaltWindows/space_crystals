# Feature Update: Syndicate Objects

**Date**: 2026-03-06
**Feature file**: `features/syndicate_objects.md` (new)
**Design sources**: `design/syndicate_objects.md`, `design/factions.md`

## Summary

Created new feature specification for Syndicate faction objects, covering the Tunnel Network system — the Syndicate's core base-building mechanic.

## Specifications Added

### Tunnel Network
- Collective of all player-owned Tunnels
- Free intra-network travel for units
- Entry/exit gated by Tunnel tier vs unit base category
- Transit tier mapping: Infantry (Tier 1+), Vehicles (Tier 2+), Air (Tier 3+)

### Tunnel (Structure)
- 4x4, AAAA symmetry, destructible, groupable
- 3 upgrade tiers with increasing Tunnel Area radius (3/4/5)
- Each tier unlocks higher-tier buildings and unit transit

### Tunnel Area
- Square underground build zone centered on Tunnel
- Spatial grid placement matching GDO surface building mechanics
- Non-overlap rule using current (not maximum) tier area
- Upgrade blocked if enlarged area would overlap

### Construction Rules
- One operation at a time (build expansion OR upgrade)
- Cost scaling: additional Tunnels cost more, upgrades cost more based on existing count
- All costs in Supplies

## Also Modified
- `features/factions_and_resources.md`: Updated note about Syndicate formalization status

## Open Items Flagged
- HP/armor/sight per tier, Tunnel Space per tier, cost values, expansion roster, Agent unit, defensive structures, detection mechanics all remain TBD per design source
