# entity-migration-neutral

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-declarative-entity-definitions.md

## Task

Migrate both neutral resource objects into per-entity definition files.

**Create 2 files:**
- src/game/entities/neutral/space_crystals_patch.rs
- src/game/entities/neutral/supply_delivery_station.rs

These are simpler entities — they are indestructible, neutral, non-combat resource nodes. Implement EntityDefinition trait (not UnitDefinition or StructureDefinition — they may need an EntityKind::Resource variant or a minimal trait impl).

Migrate from:
- ObjectType data from objects.rs (indestructible, neutral, SightRange=0)
- SpaceCrystalPatch component data (remaining_amount, initial_amount, has_plate)
- SupplyDeliveryStation component data (delivery_size, delivery_interval)
- Spawn logic from spawn_space_crystal_patch(), spawn_supply_delivery_station() in utils.rs
- Any command panel slots (likely none for neutral resources)

Register both in build_registry(). Keep old functions as thin wrappers during transition.

**Tests per entity file:**
- Stats match current values (indestructible, neutral)
- Spawn produces same components (SpaceCrystalPatch/SupplyDeliveryStation components with correct fields)

**Reference:** Architecture doc Phase 5, neutral/ directory
