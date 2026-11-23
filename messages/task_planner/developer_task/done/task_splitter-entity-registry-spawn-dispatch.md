# entity-registry-spawn-dispatch

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-declarative-entity-definitions.md

## Task

Replace per-entity spawn function calls throughout the codebase with registry-based spawn dispatch.

**Current state:** Callers use specific functions like `spawn_peacekeeper()`, `spawn_syndicate_agent()`, `spawn_tunnel()`, etc. These are currently thin wrappers delegating to entity definitions (set up by migration tasks).

**After this task:** A single generic spawn path via EntityRegistry:
1. Add a `spawn(&self, object: &ObjectEnum, commands: &mut EntityCommands, params: SpawnParams)` method to EntityRegistry that dispatches to the registered spawn_fn
2. Update all spawn call sites (production systems in faction/, construction systems, initial game setup) to use `registry.spawn()` instead of per-entity functions
3. The thin wrapper functions in utils.rs and unit_data.rs can remain temporarily (removed in cleanup task) but all callers should go through the registry

**Call sites to update:**
- Production tick systems (barracks_production_tick, headquarters_production_tick, etc.) — spawn produced units via registry
- Construction completion (deployment_center construction) — spawn completed structures via registry
- Initial game setup / scenario spawning — use registry
- Any test helpers that spawn entities

**Tests:**
- All spawn call sites use registry dispatch
- Spawned entities have identical components to pre-refactor
- All existing spawn-related tests pass
- Production and construction systems work identically

**Reference:** Architecture doc Phase 6 (Wire Registry Consumers)
