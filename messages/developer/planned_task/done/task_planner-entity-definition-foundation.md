# entity-definition-foundation

## Metadata
- **From**: task_planner
- **To**: developer

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

## Technical Context

### Files to Create

1. **`artifacts/developer/src/game/entities/mod.rs`** — Main module file with traits, shared types, and registry
2. **`artifacts/developer/src/game/entities/registry.rs`** — Registry builder + Bevy plugin
3. **Directory scaffold** (each with empty `mod.rs`):
   - `artifacts/developer/src/game/entities/gdo/mod.rs`
   - `artifacts/developer/src/game/entities/gdo/units/mod.rs`
   - `artifacts/developer/src/game/entities/gdo/buildings/mod.rs`
   - `artifacts/developer/src/game/entities/syndicate/mod.rs`
   - `artifacts/developer/src/game/entities/syndicate/units/mod.rs`
   - `artifacts/developer/src/game/entities/syndicate/buildings/mod.rs`
   - `artifacts/developer/src/game/entities/cults/mod.rs`
   - `artifacts/developer/src/game/entities/cults/units/mod.rs`
   - `artifacts/developer/src/game/entities/cults/buildings/mod.rs`
   - `artifacts/developer/src/game/entities/neutral/mod.rs`

### Files to Modify

4. **`artifacts/developer/src/game/mod.rs`** — Add `pub mod entities;` and re-export the plugin. Currently (line 1-10):
   ```rust
   pub mod types;
   pub mod utils;
   pub mod combat;
   pub mod units;
   pub mod world;
   // Re-export plugins for convenience
   pub use combat::{CombatPlugin, TurretPlugin, ProjectilePlugin};
   pub use units::{UnitsPlugin, CommandsPlugin};
   pub use world::{MapPlugin, ResourcesPlugin, FactionPlugin};
   ```
   Add: `pub mod entities;` and `pub use entities::EntitiesPlugin;`

5. **`artifacts/developer/src/main.rs`** (line 5, 32) — Import and add `EntitiesPlugin` to the plugin tuple on line 32:
   ```rust
   // line 5: add EntitiesPlugin to imports
   use space_crystals::game::{..., EntitiesPlugin};
   // line 32: add to plugin list
   .add_plugins((..., EntitiesPlugin, GamePlugin));
   ```

### Existing Types to Reference (DO NOT modify these, import them)

- **`ObjectEnum`** — `artifacts/developer/src/shared/types.rs:316-344`. The enum has 22 variants across GDO units/structures, Syndicate units/structures, Cults units/structures, and neutral resources. Used as the HashMap key for the registry. Derives Hash + Eq already.
- **`FactionEnum`** — `artifacts/developer/src/shared/types.rs:275-280`. Four variants: GlobalDefenseOrdinance, TheSyndicate, TheCults, Colonists.
- **`ObjectType`** — `artifacts/developer/src/game/types/objects.rs:14-20`. Existing struct with name, size, destructible, sight_range, groupable fields. Use this directly in the trait (not a new type).
- **`StructureCost`** — `artifacts/developer/src/game/types/structures.rs:49-53`. Has space_crystals: u32, build_frames: u32.
- **`AttackAttributesData`** — `artifacts/developer/src/game/units/types/unit_data.rs:64-88`. Attack stats struct with attack_type, damage, range, aim/firing/cooldown/reload durations.
- **`UnitTypeData`** — `artifacts/developer/src/game/units/types/unit_data.rs:12-21`. Stats struct with faction, silhouette, max_hp, armor, unit_base.
- **`UnitBaseEnum`** — `artifacts/developer/src/shared/types.rs:346+`. Movement archetype enum (LightInfantry, HeavyInfantry, etc.).
- **`MovementParams`** — `artifacts/developer/src/game/units/types/movement.rs:177-183`. Enum wrapping 5 movement model param structs (TurnRate, FixedTurnRadius, SpeedTurnRadius, Drag, Glider).
- **`SymmetryTypeEnum`** — `artifacts/developer/src/shared/types.rs`. Used for structure rotation symmetry.
- **`StructureRotation`** — `artifacts/developer/src/shared/types.rs`. Rotation variant for structures.
- **`Owner`** — `artifacts/developer/src/shared/types.rs`. Component identifying entity ownership (player number).
- **`CommandButtonAction`** — `artifacts/developer/src/ui/types.rs:254+`. Enum of all command panel button actions.
- **`ObjectInterfaceState`** — `artifacts/developer/src/ui/types.rs:151-162`. Enum with Default, AwaitingTarget, StructureMenu, AgentMenu, CultsRecruitMenu variants. Needed for `CommandSlotDef.state`.

### Trait Design Details (Option B from Architecture Doc)

**`EntityDefinition` base trait:**
```rust
pub trait EntityDefinition {
    const OBJECT: ObjectEnum;
    fn object_type() -> ObjectType;
    fn base_stats() -> BaseStats;
    fn cost() -> Option<StructureCost> { None }
    fn command_slots() -> &'static [CommandSlotDef] { &[] }
    fn spawn(commands: &mut EntityCommands, params: SpawnParams);
}
```

**`UnitDefinition` extends it:**
```rust
pub trait UnitDefinition: EntityDefinition {
    fn unit_stats() -> UnitStats;
    fn attack_data() -> Option<AttackAttributesData> { None }
    fn movement_params() -> MovementParams;
}
```

**`StructureDefinition` extends it:**
```rust
pub trait StructureDefinition: EntityDefinition {
    fn structure_stats() -> StructureStats;
}
```

### New Types to Define

**`BaseStats`**: Shared stats for all entities.
```rust
pub struct BaseStats {
    pub max_hp: u32,
    pub point_armor: u32,
    pub full_armor: u32,
    pub faction: FactionEnum,
}
```

**`UnitStats`**: Unit-specific stats.
```rust
pub struct UnitStats {
    pub unit_base: UnitBaseEnum,
    pub silhouette_width: u32,
    pub silhouette_height: u32,
    pub control_cost: u32,
}
```

**`StructureStats`**: Structure-specific stats.
```rust
pub struct StructureStats {
    pub power_value: i32,        // positive = generator, negative = consumer
    pub build_radius: u32,       // 0 if none
    pub symmetry: SymmetryTypeEnum,
}
```

**`CommandSlotDef`**: Maps an interface state + grid position to a command action.
```rust
pub struct CommandSlotDef {
    pub state: ObjectInterfaceState,
    pub row: u8,
    pub col: u8,
    pub action: CommandButtonAction,
}
```
Note: The architecture doc mentions a `condition: Option<SlotCondition>` field for conditional slots (e.g., cancel button only visible when constructing). You may add a `SlotCondition` enum or defer it — the foundation task can start with unconditional slots and add conditions in Phase 2. At minimum, leave a TODO comment.

**`SpawnParams`**: Parameters passed to spawn functions.
```rust
pub struct SpawnParams {
    pub position: Vec3,
    pub owner: Owner,
    pub player: u8,
    pub rotation: Option<StructureRotation>,
    pub under_construction: bool,
}
```

**`EntityKind`**: Discriminant stored in registry entries.
```rust
pub enum EntityKind {
    Unit {
        unit_stats: UnitStats,
        attack_data: Option<AttackAttributesData>,
        movement_params: MovementParams,
    },
    Structure {
        structure_stats: StructureStats,
    },
    Resource,
}
```

### EntityRegistry Implementation

```rust
#[derive(Resource)]
pub struct EntityRegistry {
    entries: HashMap<ObjectEnum, RegistryEntry>,
}

pub struct RegistryEntry {
    pub object_type: ObjectType,
    pub base_stats: BaseStats,
    pub cost: Option<StructureCost>,
    pub command_slots: &'static [CommandSlotDef],
    pub spawn_fn: fn(&mut EntityCommands, SpawnParams),
    pub kind: EntityKind,
}

impl EntityRegistry {
    pub fn new() -> Self { ... }
    pub fn register_unit<T: UnitDefinition>(&mut self) { ... }
    pub fn register_structure<T: StructureDefinition>(&mut self) { ... }
    pub fn get(&self, object: &ObjectEnum) -> Option<&RegistryEntry> { ... }
}
```

The `register_unit` method should:
1. Call `T::OBJECT` to get the ObjectEnum key
2. Assert no duplicate registration (panic with descriptive message)
3. Build a `RegistryEntry` by calling `T::object_type()`, `T::base_stats()`, `T::cost()`, `T::command_slots()`, `T::spawn` (as fn pointer), and construct `EntityKind::Unit { unit_stats: T::unit_stats(), attack_data: T::attack_data(), movement_params: T::movement_params() }`

Similarly for `register_structure`.

### Bevy Plugin Pattern

Follow the existing plugin pattern (see `artifacts/developer/src/game/combat/mod.rs:21-23`, `artifacts/developer/src/simulation/mod.rs:21-23`):
```rust
pub struct EntitiesPlugin;

impl Plugin for EntitiesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(build_registry());
    }
}
```

Place this in `registry.rs` and re-export from `entities/mod.rs`.

### Import Path Considerations

The crate uses `crate::types` for shared types (re-exported from `shared/types.rs` via `lib.rs`). Game-specific types use `crate::game::types::*`. UI types use `crate::ui::types::*`.

For the entity traits, you will need:
- `use crate::types::{ObjectEnum, FactionEnum, UnitBaseEnum, SymmetryTypeEnum, StructureRotation, Owner};`
- `use crate::game::types::objects::ObjectType;`
- `use crate::game::types::structures::StructureCost;`
- `use crate::game::units::types::unit_data::AttackAttributesData;`
- `use crate::game::units::types::movement::MovementParams;`
- `use crate::ui::types::{ObjectInterfaceState, CommandButtonAction};`
- `use bevy::prelude::*;` (for Resource derive, EntityCommands, Vec3, etc.)

### Testing Pattern

Follow the crate's existing test pattern — `#[cfg(test)] mod tests` at the bottom of the file. See `unit_data.rs` for examples of unit tests in this codebase. For the registry tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Create a minimal test entity implementing UnitDefinition
    struct TestUnit;
    impl EntityDefinition for TestUnit {
        const OBJECT: ObjectEnum = ObjectEnum::Peacekeeper; // reuse for test
        fn object_type() -> ObjectType { ... }
        fn base_stats() -> BaseStats { ... }
        fn spawn(commands: &mut EntityCommands, params: SpawnParams) { /* no-op for test */ }
    }
    impl UnitDefinition for TestUnit {
        fn unit_stats() -> UnitStats { ... }
        fn movement_params() -> MovementParams { ... }
    }

    #[test]
    fn test_register_and_lookup() { ... }

    #[test]
    #[should_panic]
    fn test_duplicate_registration_panics() { ... }

    #[test]
    fn test_empty_registry_returns_none() { ... }

    #[test]
    fn test_entity_kind_is_unit() { ... }
}
```

Note: The `spawn` fn in tests can be a no-op since registry tests don't need to actually spawn Bevy entities. The spawn_fn is stored as a function pointer and only called at runtime.

## Dependencies

None — this is the first task in the declarative entity definitions feature. All other entity migration tasks depend on this one.
