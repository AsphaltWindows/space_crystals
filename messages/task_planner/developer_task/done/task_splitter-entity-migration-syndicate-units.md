# entity-migration-syndicate-units

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-declarative-entity-definitions.md

## Task

Migrate both Syndicate units into per-entity definition files.

**Create 2 files:**
- src/game/entities/syndicate/units/agent.rs
- src/game/entities/syndicate/units/guard.rs

For each, implement EntityDefinition + UnitDefinition traits. Migrate from:
- agent_type_data(), agent_attack_data(), guard_type_data(), guard_attack_data() in unit_data.rs
- Agent-specific constants: AGENT_MINING_DURATION, AGENT_CRYSTAL_CARRY, AGENT_CONTROL_COST, AGENT_TUNNEL_SPACE_COST, etc.
- Guard-specific constants: GUARD_CONTROL_COST, GUARD_TUNNEL_SPACE_COST
- Command panel slots from grid_layout.rs (Agent has multiple interface states: Default, AgentMenu variants)
- Spawn logic from spawn_syndicate_agent(), spawn_syndicate_guard() in utils.rs
- Production costs from HeadquartersState::production_cost()

**Note:** Agent is more complex than most entities — it has gathering, tunnel-building, and carrying capabilities with associated constants. Ensure all Agent-specific constants are consolidated in the entity file.

Register both in build_registry(). Keep old functions as thin wrappers during transition.

**Tests per entity file:**
- Stats match current type_data values
- Attack data matches current attack_data values
- All entity-specific constants preserved with correct values
- Command slots match current grid_layout.rs entries
- Cost matches HeadquartersState::production_cost() values
- Spawn produces same components

**Reference:** Architecture doc Phase 4 (Syndicate Entities)
