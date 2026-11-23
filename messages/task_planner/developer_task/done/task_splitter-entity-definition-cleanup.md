# entity-definition-cleanup

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-declarative-entity-definitions.md

## Task

Remove all emptied legacy code that has been superseded by the EntityRegistry and per-entity definition files. This is the final cleanup task after all entities are migrated and all consumers are wired to the registry.

**Code to remove:**

1. **utils.rs** — Remove all per-entity spawn functions (spawn_peacekeeper, spawn_syndicate_agent, spawn_tunnel, etc.) and their thin wrappers. The file should only retain non-entity utility functions (if any).

2. **unit_data.rs** — Remove all per-unit type_data() and attack_data() functions (peacekeeper_type_data, agent_type_data, guard_type_data, etc.) and per-unit constants that have been moved to entity files. Remove thin wrappers.

3. **objects.rs** — Remove per-entity ObjectType/StructureType construction code that is now in entity files. Keep the ObjectEnum enum definition itself and any shared type definitions.

4. **structures.rs** — Remove the gdo_structure_stats, syndicate_structure_stats, cults_structure_stats submodules (stat constants moved to entity files). Remove construction_cost() and production_cost() match arms (now in registry). Keep shared types (StructureCost, production state components) that are still used by systems.

5. **grid_layout.rs** — Remove any remaining per-entity match arms (should already be gone from the grid-layout wiring task, but verify and clean up).

6. **Update all remaining import paths** throughout the codebase that still reference the old locations. Tests that imported peacekeeper_type_data() etc. should import from the entity definition files instead.

**Verification:**
- cargo check passes — no dead code warnings for removed items
- cargo test --lib passes — all 1,920+ unit tests still pass (with updated imports)
- cargo test --test qa passes — all 318+ integration tests pass
- No remaining references to removed functions anywhere in the codebase
- Files are significantly smaller: utils.rs, unit_data.rs, objects.rs, structures.rs should each lose hundreds of lines

**Reference:** Architecture doc Phase 7 (Cleanup)
