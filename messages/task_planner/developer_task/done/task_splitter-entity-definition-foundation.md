# entity-definition-foundation

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-declarative-entity-definitions.md

## Task

Create the foundation for the declarative entity definition system. This is the infrastructure that all subsequent entity migration tasks depend on.

**New files to create:**

1. **src/game/entities/mod.rs** — Contains:
   - `EntityDefinition` trait (base trait: `const OBJECT: ObjectEnum`, `fn object_type() -> ObjectType`, `fn base_stats() -> BaseStats`, `fn cost() -> Option<StructureCost>`, `fn command_slots() -> &'static [CommandSlotDef]`, `fn spawn(commands: &mut EntityCommands, params: SpawnParams)`)
   - `UnitDefinition: EntityDefinition` trait (`fn unit_stats() -> UnitStats`, `fn attack_data() -> Option<AttackAttributesData>`, `fn movement_params() -> MovementParams`)
   - `StructureDefinition: EntityDefinition` trait (`fn structure_stats() -> StructureStats`)
   - Shared types: `BaseStats` (max_hp, point_armor, full_armor, faction), `UnitStats` (unit_base, silhouette, control_cost), `StructureStats` (power_value, build_radius, symmetry), `CommandSlotDef` (state, row, col, action, condition), `SpawnParams` (position, owner, player, rotation, under_construction), `EntityKind` enum (Unit/Structure/Resource)
   - `EntityRegistry` resource struct with `entries: HashMap<ObjectEnum, RegistryEntry>`
   - `RegistryEntry` struct storing: object_type, base_stats, cost, command_slots, spawn_fn pointer, EntityKind
   - `register_unit::<T: UnitDefinition>()` and `register_structure::<T: StructureDefinition>()` methods on EntityRegistry
   - Lookup methods: `get(&self, object: &ObjectEnum) -> Option<&RegistryEntry>`

2. **src/game/entities/registry.rs** — Contains:
   - `build_registry() -> EntityRegistry` function (initially empty, will be populated as entities are migrated)
   - Bevy plugin setup to insert EntityRegistry as a Resource at startup

3. **Module structure directories** (with mod.rs files):
   - src/game/entities/gdo/mod.rs, gdo/units/mod.rs, gdo/buildings/mod.rs
   - src/game/entities/syndicate/mod.rs, syndicate/units/mod.rs, syndicate/buildings/mod.rs
   - src/game/entities/cults/mod.rs, cults/units/mod.rs, cults/buildings/mod.rs
   - src/game/entities/neutral/mod.rs

4. Wire the entities module into `src/game/mod.rs`

**Tests:**
- Registry operations: register, lookup, no duplicate registrations
- Empty registry returns None for lookups
- EntityKind variants are correct for unit vs structure registrations

**Reference:** Architecture doc at artifacts/software_architect/architecture/declarative-entity-definitions.md (Trait Design Option B, EntityRegistry section)
