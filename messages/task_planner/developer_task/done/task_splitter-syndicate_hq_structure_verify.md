# syndicate-hq-structure-verify

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-syndicate-headquarters-structure.md

## Task

**Verification-only task**: The Headquarters underground expansion structure is already fully implemented. All stats, construction details, production catalog, and starting condition match the design spec. Verify the following are correct and aligned with `artifacts/designer/design/syndicate_objects.md`:

1. **ObjectEnum::Headquarters** in `objects.rs`: size (2,2), destructible, groupable=false
2. **Constants** in `structures.rs` (syndicate_structure_stats): HQ_MAX_HP=400, HQ_POINT_ARMOR=1, HQ_FULL_ARMOR=4, HQ_SC_COST=200, HQ_BUILD_FRAMES=400
3. **spawn_headquarters()** in `utils.rs`: ObjectInstance::destructible with HQ_MAX_HP, DomainEnum::Underground, HeadquartersState::default(), TunnelExpansionMarker
4. **HeadquartersState** in `structures.rs`: rally_point, build_queue (max 5), current_build, current_build_progress, production_cost() for Agent (100 SC/160f) and Guard (125 SC/120f)
5. **Starting condition** in `faction.rs`: spawn_headquarters called during Syndicate setup (pre-built in starting tunnel)
6. **Tunnel ExpandMenu integration**: HQ appears in expand menu, costs 200 SC, builds in 400 frames
7. **Tests**: All existing tests pass (HQ stats, production costs, spawn)

If any discrepancies are found, fix them. Otherwise confirm all is correct.
