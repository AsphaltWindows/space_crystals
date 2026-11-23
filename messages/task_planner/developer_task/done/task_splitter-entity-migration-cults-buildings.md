# entity-migration-cults-buildings

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-declarative-entity-definitions.md

## Task

Migrate all 3 Cults structures into per-entity definition files.

**Create 3 files:**
- src/game/entities/cults/buildings/recruitment_center.rs
- src/game/entities/cults/buildings/storage.rs
- src/game/entities/cults/buildings/armory.rs

For each, implement EntityDefinition + StructureDefinition traits. Migrate from:
- Structure stats from structures.rs cults_structure_stats module (HP, armor, power, symmetry)
- Construction costs from relevant state component match arms
- Production costs (RecruitmentCenter and Armory produce units)
- Command panel slots from grid_layout.rs
- Spawn logic from respective spawn_*() functions in utils.rs
- ObjectType data from objects.rs

Register all 3 in build_registry(). Keep old functions as thin wrappers during transition.

**Tests per entity file:**
- Stats match current cults_structure_stats values
- Construction/production costs match current values
- Command slots match current grid_layout.rs entries
- Spawn produces same components

**Reference:** Architecture doc Phase 5 (Cults Entities)
