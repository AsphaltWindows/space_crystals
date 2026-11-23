# entity-migration-neutral

## Metadata
- **From**: task_planner
- **To**: developer

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

## Technical Context

### Files to Create

1. **`artifacts/developer/src/game/entities/neutral/space_crystals_patch.rs`** — New entity definition for Space Crystal Patch
2. **`artifacts/developer/src/game/entities/neutral/supply_delivery_station.rs`** — New entity definition for Supply Delivery Station

### Files to Modify

3. **`artifacts/developer/src/game/entities/neutral/mod.rs`** — Currently just a doc comment. Add `pub mod space_crystals_patch;` and `pub mod supply_delivery_station;`
4. **`artifacts/developer/src/game/entities/types.rs`** — Add `register_resource<T: EntityDefinition>()` method to `EntityRegistry` (see below)
5. **`artifacts/developer/src/game/entities/registry.rs`** — Add `register_resource::<SpaceCrystalsPatchDef>()` and `register_resource::<SupplyDeliveryStationDef>()` calls in `build_registry()`. Import both from `super::neutral::*`. Update the `test_build_registry_has_peacekeeper` test assertion from `registry.len() == 10` to `registry.len() == 12`.

### Key Pattern: register_resource Method

`EntityKind::Resource` already exists in `types.rs:76` but has NO corresponding `register_resource()` method on `EntityRegistry`. You must add one, following the pattern of `register_unit()` (line 128) and `register_structure()` (line 152):

```rust
/// Register a resource entity definition. Panics on duplicate registration.
pub fn register_resource<T: EntityDefinition>(&mut self) {
    let object = T::OBJECT;
    if self.entries.contains_key(&object) {
        panic!("Duplicate entity registration for {:?}", object);
    }
    let entry = RegistryEntry {
        object_type: T::object_type(),
        base_stats: T::base_stats(),
        cost: T::cost(),
        command_slots: T::command_slots(),
        spawn_fn: T::spawn,
        kind: EntityKind::Resource,
    };
    self.entries.insert(object, entry);
}
```

Note: `register_resource` takes `T: EntityDefinition` (NOT `UnitDefinition` or `StructureDefinition`) since resources have no unit or structure stats.

### Entity Definition Pattern (follow Peacekeeper at `entities/factions/gdo/units/peacekeeper.rs`)

Each entity file contains:
- A marker struct (e.g., `pub struct SpaceCrystalsPatchDef;`)
- `impl EntityDefinition for SpaceCrystalsPatchDef { ... }` with:
  - `const OBJECT: ObjectEnum = ObjectEnum::SpaceCrystalsPatch;`
  - `fn object_type() -> ObjectType` — matching data from `objects.rs:345-351`
  - `fn base_stats() -> BaseStats` — hp=0, armor=0, faction needs a value (see note below)
  - `fn cost() -> Option<StructureCost>` — `None` (not producible)
  - `fn command_slots() -> &'static [CommandSlotDef]` — `&[]` (no commands)
  - `fn spawn(commands: &mut EntityCommands, params: SpawnParams)` — insert gameplay components
- A `#[cfg(test)] mod tests` block

### Faction for Neutral Entities

`FactionEnum` (`shared/types.rs:275`) does NOT have a `Neutral` variant. The four factions are: GDO, Syndicate, Cults, Colonists. For `BaseStats.faction`, pick a convention:
- **Option A**: Use `FactionEnum::Colonists` as a placeholder for neutral (document this)
- **Option B**: Add a comment noting this is a neutral entity and use any faction value
- The `Owner::neutral()` (`Owner(None)`) is what actually marks an entity as neutral in gameplay — this goes in the spawn function via `params.owner`

### Source Data for SpaceCrystalsPatch

From `objects.rs:345-351`:
- name: "Space Crystals Patch", size: (1,1), destructible: false, sight_range: 0, groupable: false

From `world/types.rs:162-168` (`SpaceCrystalPatch` component):
- `remaining_amount: u32`, `initial_amount: u32`, `has_plate: bool`

From `world/resources.rs:53-67` (spawn logic):
- Components: `ObjectInstance::indestructible(ObjectEnum::SpaceCrystalsPatch)`, `Owner::neutral()`, `SpaceCrystalPatch { remaining_amount, initial_amount, has_plate: false }`, `Selectable`, `SelectionBounds::from_dimensions(0.6, 0.8, 0.6)`, `GridPosition`
- Transform offset: `y: 0.4`
- Visual components (Mesh3d, MeshMaterial3d) are NOT inserted by entity definitions — leave those to the caller

**SpawnParams gap**: The `SpawnParams` struct (`types.rs:55-62`) has `position`, `owner`, `player`, `rotation`, `under_construction`. It does NOT have `remaining_amount`/`initial_amount` fields. For SpaceCrystalPatch, the spawn function will need to use **default values** (e.g., `remaining_amount: 5000, initial_amount: 5000, has_plate: false`) OR `SpawnParams` needs extending. **Recommended**: use sensible defaults in spawn(), and the actual spawning code in `resources.rs` can override component values after spawn. OR keep `resources.rs` spawning as-is (with the thin wrapper approach) until the spawn-dispatch task.

### Source Data for SupplyDeliveryStation

From `objects.rs:352-358`:
- name: "Supply Delivery Station", size: (2,2), destructible: false, sight_range: 0, groupable: false

From `world/types.rs:172-177` (`SupplyDeliveryStation` component):
- `delivery_size: u32`, `delivery_interval: f32`, `current_supplies: u32`, `time_until_next_delivery: f32`

From `world/resources.rs:582-597` (spawn logic):
- Components: `ObjectInstance::indestructible(ObjectEnum::SupplyDeliveryStation)`, `Owner::neutral()`, `SupplyDeliveryStation { delivery_size, delivery_interval, current_supplies: delivery_size, time_until_next_delivery: delivery_interval }`, `Selectable`, `SelectionBounds::from_dimensions(1.6, 0.2, 1.6)`, `GridPosition`
- Transform offset: `y: 0.1`
- Same SpawnParams gap: delivery_size/delivery_interval not in SpawnParams

### Imports Needed in Entity Files

From Peacekeeper pattern, neutral entities need a subset:
```rust
use bevy::prelude::*;
use crate::types::ObjectEnum;
use crate::game::entities::types::{EntityDefinition, BaseStats, SpawnParams};
use crate::game::types::objects::ObjectType;
use crate::game::types::ObjectInstance;
use crate::shared::types::{Selectable, SelectionBounds, GridPosition};
use crate::game::world::types::{SpaceCrystalPatch}; // or SupplyDeliveryStation
```

Note: `FactionEnum` is in `crate::types::FactionEnum`. `Owner` is in `crate::types::Owner`.

### Test Pattern (follow Peacekeeper tests)

Each entity file should have tests for:
1. `base_stats()` — verify hp=0 (or max_hp for indestructible), armor=0
2. `object_type()` — verify name, size, destructible=false, sight_range=0, groupable=false
3. `cost()` — verify None
4. `command_slots()` — verify empty
5. `OBJECT` — verify correct ObjectEnum variant
6. Registry round-trip: create registry, register, lookup, verify entry matches

### Keep Old Spawn Functions as Thin Wrappers

The task says to keep old functions during transition. The `spawn_space_crystal_patches` and `spawn_supply_delivery_stations` in `resources.rs` should remain UNCHANGED for now — they are map-level spawn orchestrators (iterate over map positions, set tile properties, handle mesh/material). The entity definition's `spawn()` only handles the per-entity component insertion. The transition to using entity spawn will happen in the `entity-registry-spawn-dispatch` task.

## Dependencies

- **Entity definition foundation** (already complete): `src/game/entities/mod.rs`, `types.rs`, `registry.rs` exist with `EntityDefinition`, `UnitDefinition`, `StructureDefinition` traits and `EntityRegistry` resource. `EntityKind::Resource` variant exists but `register_resource()` method does not — must be added.
- **No dependency on other migration tasks**: Neutral resources are independent of faction entity migrations (GDO/Syndicate/Cults units and buildings).
- **Downstream dependency**: `entity-registry-grid-layout`, `entity-registry-spawn-dispatch`, `entity-registry-costs`, and `entity-definition-cleanup` all depend on ALL entity migrations (including this one) being complete before they can proceed.
