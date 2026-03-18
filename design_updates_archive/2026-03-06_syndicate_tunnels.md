# Design Update: Syndicate Tunnel Mechanics

**Date**: 2026-03-06
**Files changed**: `design/syndicate_objects.md` (new)

## Summary

Formalized the core Syndicate building mechanic: Tunnels and the Tunnel Network. This establishes the faction's unique base-building identity — visible surface entry points with invisible underground expansion areas.

## Decisions

### Tunnel Network
- Defined as the collective of all Tunnels a player owns
- Units travel freely within the network but can only enter/exit through Tunnels of appropriate tier

### Tunnel Structure
- 4x4 AAAA surface structure, visible to enemies
- 3 upgrade tiers with increasing Tunnel Area radius
- Groupable, destructible

### Tunnel Tiers and Transit
- Tier 1: Infantry transit, radius 3 (10x10 area)
- Tier 2: Vehicle transit + below, radius 4 (12x12 area)
- Tier 3: Air transit + below, radius 5 (14x14 area)
- Tunnel tier gates both what buildings can be built in its area AND what units can transit through it

### Tunnel Area
- Square underground grid zone centered on the Tunnel
- Underground expansions placed spatially on the grid (like GDO surface buildings)
- Invisible to enemies without detection
- Non-overlap rule: Tunnel Areas cannot overlap; check uses current tier area; upgrades blocked if they would cause overlap

### Construction Rules
- One operation at a time: upgrade OR construct expansion, never both
- Cost scaling: additional Tunnels cost more based on total owned; upgrades cost more based on number at target tier; both cost Supplies

## Still Open
- Tunnel HP, armor, sight per tier
- Tunnel Space provided per tier
- Specific cost/timing values
- Underground expansion building roster
- Agent worker unit
- Syndicate defensive structures
- Detection mechanics for underground buildings
