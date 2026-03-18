# Feature Update: Syndicate Objects - Agent Unit & Headquarters Update

**Date**: 2026-03-06
**Feature file**: `features/syndicate_objects.md`
**Design sources**: `design/syndicate_objects.md`, `design/combat.md`
**Triggered by**: `design_updates/2026-03-06_agent_unit_and_melee_subtype.md`

## Modifications

### Added: Agent Unit (full specification)
- Syndicate's cyborg worker unit (HeavyInfantry, 36x36 silhouette)
- Stats: 75 HP, 1/1 armor, sight 5, speed 6 su/frame, TunnelSpaceCost 2
- Attack: FullyConnected Melee, Ground, SingleTarget, 6 damage
- Gathering: 50 SC per load (48f mine/drop-off at Side B), 1 Supply per trip (48f pick-up/drop-off at Side C)
- Building: Constructs Tunnels and defensive structures on surface (must be present)
- Produced by Headquarters: 100 SC, 160 frames

### Updated: Headquarters
- Removed Guard from production roster — HQ now only produces Agents
- Added production cost/time (100 SC, 160 frames)
- Guard will be produced by a separate T1 underground expansion (TBD)

### Updated: Open Questions
- Resolved: Agent worker unit specification (now fully specified)
- Added: Guard-producing expansion (TBD), Agent gathering commands/behaviors integration
- Retained: HQ size/HP/armor/build cost, defensive structures, detection, tunnel destruction, upgrade duration

### Added: Production Chain Summary
- Headquarters --> Agent

### Updated: Dependencies
- Added `combat_system` (Agent's FullyConnected Melee attack)
- Added Space Crystals to factions_and_resources dependency note
