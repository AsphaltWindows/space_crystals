# entity-migration-cults-units

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-declarative-entity-definitions.md

## Task

Migrate all 3 Cults units into per-entity definition files.

**Create 3 files:**
- src/game/entities/cults/units/recruit.rs
- src/game/entities/cults/units/soldier.rs
- src/game/entities/cults/units/gunner.rs

For each, implement EntityDefinition + UnitDefinition traits. Migrate from:
- Type data and attack data functions in unit_data.rs (recruit_type_data, soldier_type_data, gunner_type_data, etc.)
- Unit-specific constants (control costs, any special attributes)
- Command panel slots from grid_layout.rs
- Spawn logic from respective spawn_*() functions in utils.rs
- Production costs from CultsArmory or RecruitmentCenter production_cost() if applicable

Register all 3 in build_registry(). Keep old functions as thin wrappers during transition.

**Tests per entity file:**
- Stats match current type_data values
- Attack data matches current attack_data values
- Entity-specific constants preserved
- Command slots match current grid_layout.rs entries
- Cost matches current production cost values
- Spawn produces same components

**Reference:** Architecture doc Phase 5 (Cults Entities)
