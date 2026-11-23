# entity-def-pilot-peacekeeper

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-declarative-entity-definitions.md

## Task

Migrate Peacekeeper as the pilot entity to validate the EntityDefinition/UnitDefinition trait pattern end-to-end.

**Create:** src/game/entities/gdo/units/peacekeeper.rs

This file should contain:
- `pub struct Peacekeeper;`
- `impl EntityDefinition for Peacekeeper` — OBJECT, object_type(), base_stats(), cost(), command_slots(), spawn()
- `impl UnitDefinition for Peacekeeper` — unit_stats(), attack_data(), movement_params()
- All values migrated from: `peacekeeper_type_data()` in unit_data.rs, `peacekeeper_attack_data()` in unit_data.rs, `PEACEKEEPER_CONTROL_COST` constant, command panel slots from grid_layout.rs, production cost from BarracksState::production_cost(), spawn logic from `spawn_peacekeeper()` in utils.rs
- Entity-specific constants: PEACEKEEPER_CONTROL_COST, PEACEKEEPER_RUGGED_BONUS
- Per-entity tests in `#[cfg(test)]` module verifying stats, attack data, command slots, cost

**Register** Peacekeeper in registry.rs `build_registry()`

**Keep old functions as thin wrappers** that delegate to the new entity definition (for backward compatibility during migration). E.g., `peacekeeper_type_data()` can call `Peacekeeper::base_stats()` internally.

**This task validates the pattern.** If the trait design needs adjustment, adapt it here before other entities follow.

**Tests:**
- Stats match current peacekeeper_type_data() values (HP=50, armor 1/1, LightInfantry, GDO)
- Attack data matches current peacekeeper_attack_data() values
- Command slots match current grid_layout.rs entries for Peacekeeper
- Cost matches BarracksState::production_cost(Peacekeeper) (50 SC, 80 frames)
- Registry lookup for ObjectEnum::Peacekeeper returns correct entry
- Spawning via entity definition produces same components as current spawn_peacekeeper()

**Reference:** Architecture doc Phase 2 (Pilot Entity)
