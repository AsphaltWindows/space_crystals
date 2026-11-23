# declarative-entity-definitions

## Metadata
- **From**: software_architect
- **To**: task_splitter

## Content

## Architecture Document

artifacts/software_architect/architecture/declarative-entity-definitions.md

## Summary

Consolidate scattered entity data (stats, costs, command panel layouts, spawn logic) into per-entity Rust files organized by faction, using a trait-based definition pattern (EntityDefinition/UnitDefinition/StructureDefinition) with an EntityRegistry resource. This addresses the #1 codebase maintainability pain point: adding or modifying any entity currently requires touching 4-6 files across 5,948 lines of scattered declarative data. The per-entity file structure distributes this into ~20 focused files (~200-300 lines each), making new entity additions a single-file task.

Team consensus was reached in forum topic 2026-03-25T12-00-00Z-operator-data-driven-entity-definitions.md. All agents (developer, task_splitter, task_planner, designer, automatic_qa, software_architect) agreed on: declarative Rust structs (not YAML), per-entity files by faction, trait hierarchy with explicit registry, phased migration.

## Scope

**New code:**
- src/game/entities/mod.rs -- trait definitions (EntityDefinition, UnitDefinition, StructureDefinition), shared types (BaseStats, UnitStats, StructureStats, CommandSlotDef, SpawnParams, EntityKind), EntityRegistry resource
- src/game/entities/registry.rs -- build_registry() with explicit registration calls
- src/game/entities/gdo/units/peacekeeper.rs (and all other per-entity files ~20 total)
- Per-faction mod.rs files for module organization

**Modified code (consumers migrated to registry):**
- ui/command_panel/grid_layout.rs -- replace match arms with registry lookup
- game/types/structures.rs -- production/construction costs read from registry
- game/utils.rs -- spawn functions delegate to registry or move into entity files
- game/units/types/unit_data.rs -- type_data/attack_data functions replaced by entity definitions

**Removed code (after migration complete):**
- Emptied match arms and per-entity functions from utils.rs, unit_data.rs, objects.rs, structures.rs, grid_layout.rs

**NOT in scope:** Behavior systems, state machine logic, combat systems, pathfinding, command availability logic, production queue logic (these stay in their current modules and reference the registry).

## QA Instructions

1. cargo check must pass at every phase boundary (foundation, pilot entity, each faction migration, consumer wiring, cleanup)
2. cargo test --lib must pass with all existing unit tests (1,920+)
3. cargo test --test qa must pass with all integration tests (318+)
4. Behavioral equivalence: every entity must spawn with identical ECS components, stats, and command panel layouts as before migration
5. Verify EntityRegistry contains entries for ALL ObjectEnum variants (no missing entities)
6. Verify command panel grid layout produces identical button arrangements for all ObjectInterfaceState values
7. Verify production/construction costs return identical values for all producible entities

## Testing Requirements

### Unit Tests (src/game/entities/)

**Registry tests (entities/mod.rs or registry.rs #[cfg(test)]):**
- All ObjectEnum variants have a registered entry in build_registry()
- No duplicate ObjectEnum registrations (register same variant twice -> panic or error)
- Registry lookup returns correct BaseStats for each entity (spot-check HP, armor, faction)
- Registry lookup returns correct cost for producible entities
- Registry lookup returns None cost for starting structures and resources
- Unit entries have UnitStats (unit_base, control_cost), Structure entries have StructureStats (power, symmetry)
- Spawn function pointer in registry is callable (spawns entity without panic)

**Per-entity definition tests (each entity file #[cfg(test)]):**
- Stats match design doc values: HP, point_armor, full_armor, faction
- Attack data valid where expected: damage > 0, range > 0, cooldowns > 0
- Command slots have no duplicate (row, col) pairs for the same ObjectInterfaceState
- Cost values match current hardcoded values (regression check)
- Entity-specific constants preserved (e.g., AGENT_MINING_DURATION, AGENT_CRYSTAL_CARRY unchanged)

### Integration Tests (tests/qa/)

**Spawn equivalence tests:**
- For each entity type: spawn via registry, verify entity has all expected components (Transform, Health, ObjectEnum, faction components, attack components where applicable)
- Compare spawned component values against current spawn function outputs

**Command panel equivalence tests:**
- For each ObjectInterfaceState: verify registry-based grid lookup returns same CommandButtonAction at each (row, col) as the current get_grid_slot_action() match

**Production cost equivalence tests:**
- For each producible entity: verify registry cost lookup matches current construction_cost()/production_cost() values

### Regression Tests

- Existing unit tests (1,920+) must continue passing. During migration, old functions can be thin wrappers delegating to entity definitions. After cleanup, tests update import paths.
- Existing integration tests (318+) must continue passing without modification during the coexistence period.
- The scenario test suite (27 passing) must remain stable.
