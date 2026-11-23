# entity-migration-syndicate-buildings

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-declarative-entity-definitions.md

## Task

Migrate both Syndicate structures into per-entity definition files.

**Create 2 files:**
- src/game/entities/syndicate/buildings/tunnel.rs
- src/game/entities/syndicate/buildings/headquarters.rs

For each, implement EntityDefinition + StructureDefinition traits.

**Tunnel is the most complex entity.** It has:
- Tiered stats (tier 1-3 with different HP, capacity)
- Construction variants (under_construction with ConstructionHP)
- Multiple interface states (TunnelIdle, TunnelExpandMenu, TunnelEjectMenu, TunnelAwaitingPlacement)
- Tunnel-specific constants (upgrade costs, tier capacity, transit tier filtering)
- The SpawnParams.under_construction flag drives conditional component composition

Migrate from:
- Structure stats from structures.rs syndicate_structure_stats module
- Construction costs (tunnel_construction_cost scaling)
- Command panel slots from grid_layout.rs (all Tunnel interface states)
- Spawn logic from spawn_tunnel(), spawn_tunnel_under_construction(), spawn_headquarters() in utils.rs
- ObjectType data from objects.rs

Register both in build_registry(). Keep old functions as thin wrappers during transition.

**Tests per entity file:**
- Stats match current syndicate_structure_stats values
- Tunnel tier stats are correct for all tiers
- Command slots match current grid_layout.rs entries for all Tunnel states
- HQ production costs match current values
- Spawn produces same components (including under_construction variant for Tunnel)

**Reference:** Architecture doc Phase 4 (Syndicate Entities)
