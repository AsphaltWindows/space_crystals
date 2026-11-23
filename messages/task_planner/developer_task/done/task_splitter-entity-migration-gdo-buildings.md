# entity-migration-gdo-buildings

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-declarative-entity-definitions.md

## Task

Migrate all 6 GDO structures into per-entity definition files following the established pattern.

**Create 6 files:**
- src/game/entities/gdo/buildings/deployment_center.rs
- src/game/entities/gdo/buildings/power_plant.rs
- src/game/entities/gdo/buildings/barracks.rs
- src/game/entities/gdo/buildings/extraction_facility.rs
- src/game/entities/gdo/buildings/extraction_plate.rs
- src/game/entities/gdo/buildings/supply_tower.rs

For each, implement EntityDefinition + StructureDefinition traits. Migrate from:
- Structure stats from structures.rs gdo_structure_stats module (HP, armor, power, symmetry, build_radius)
- Construction/production costs from DeploymentCenterState::construction_cost() and BarracksState::production_cost()
- Command panel slots from grid_layout.rs
- Spawn logic from respective spawn_*() functions in utils.rs
- ObjectType data from objects.rs

Register all 6 in build_registry(). Keep old functions as thin wrappers during transition.

**Tests per entity file:**
- Stats match current gdo_structure_stats values
- Construction/production costs match current match arms
- Command slots match current grid_layout.rs entries
- Spawn produces same components

**Reference:** Architecture doc Phase 3 (GDO Entities)
