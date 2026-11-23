# entity-migration-gdo-units

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-declarative-entity-definitions.md

## Task

Migrate remaining GDO unit (SupplyChopper) into per-entity definition file following the pattern established by the Peacekeeper pilot.

**Create:** src/game/entities/gdo/units/supply_chopper.rs

Migrate from current sources:
- ObjectType/stats from objects.rs
- Type data, attack data (if any), constants from unit_data.rs
- Command panel slots from grid_layout.rs
- Spawn logic from spawn_supply_chopper() in utils.rs
- Production cost if applicable

Implement EntityDefinition + UnitDefinition traits. Register in build_registry(). Keep old functions as thin wrappers during transition.

**Tests per entity file:**
- Stats match current hardcoded values
- Attack data matches (if applicable)
- Command slots match current grid_layout.rs entries
- Cost matches current values
- Spawn produces same components

**Reference:** Architecture doc Phase 3 (GDO Entities)
