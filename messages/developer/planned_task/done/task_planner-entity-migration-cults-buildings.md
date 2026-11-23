# entity-migration-cults-buildings

## Metadata
- **From**: task_planner
- **To**: developer

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

## Technical Context

### Reference Pattern — Follow GDO Buildings Exactly

Use the GDO building definitions as your template:
- `artifacts/developer/src/game/entities/factions/gdo/buildings/base.rs` — DeploymentCenter + PowerPlant (primary structure + inert structure)
- `artifacts/developer/src/game/entities/factions/gdo/buildings/military.rs` — Barracks + SupplyTower (production structures with command slots)

### Files to Create

1. **`artifacts/developer/src/game/entities/factions/cults/buildings/recruitment_center.rs`**
2. **`artifacts/developer/src/game/entities/factions/cults/buildings/storage.rs`**
3. **`artifacts/developer/src/game/entities/factions/cults/buildings/armory.rs`**

### Files to Modify

4. **`artifacts/developer/src/game/entities/factions/cults/buildings/mod.rs`** — Currently just a doc comment. Add `pub mod recruitment_center;`, `pub mod storage;`, `pub mod armory;`.
5. **`artifacts/developer/src/game/entities/registry.rs`** — Add imports and `register_structure::<>()` calls for all 3. Update test `test_build_registry_has_peacekeeper` assertion from `registry.len() == 10` to `13`.

### Trait Pattern

Each struct implements:
- `EntityDefinition` — `OBJECT`, `object_type()`, `base_stats()`, `cost()`, `command_slots()`, `spawn()`
- `StructureDefinition` — `structure_stats()`

### Per-Entity Details

#### RecruitmentCenter (recruitment_center.rs)

**Marker struct**: `pub struct RecruitmentCenterDef;`

**ObjectType** (from objects.rs:324-330):
```
name: "Recruitment Center", size: (4,4), destructible: true, sight_range: 6, groupable: false
```

**BaseStats** (from structures.rs cults_structure_stats:470-472):
```
max_hp: RC_MAX_HP (1000), point_armor: RC_POINT_ARMOR (1), full_armor: RC_FULL_ARMOR (16), faction: TheCults
```

**StructureStats**:
```
power_value: 0 (Cults have no power system), build_radius: 0, symmetry: AAAA
```

**cost()**: `None` — RC is the Cults primary structure (not built by another structure, spawned at game start)

**Command slots** (from grid_layout.rs:115-118):
```
StructureMenuState::RecruitmentCenterMenu:
  (2,1) if bk_has_queue => RcCancel
  (2,2) => SetRallyPoint
```
Static slice: 2 entries (the `bk_has_queue` guard is a runtime conditional, the slot definition still exists in the slice).

**spawn()** (from utils.rs:952-988):
Insert: Transform, ObjectInstance (destructible/under_construction), StructureInstance::default(), Owner, Selectable, SelectionBounds::from_dimensions(4.0, 1.5, 4.0), GridPosition, SightRange(6), RecruitmentCenterState { build_order: 0, ..Default::default() }.
Note: spawn() takes `SpawnParams` which lacks `build_order` — use 0 default. The visual components (Mesh3d, MeshMaterial3d, children) are NOT inserted by spawn().

**Imports needed**: `RecruitmentCenterState` from `crate::game::types::structures`, `cults_structure_stats::*` from same module.

#### CultsStorage (storage.rs)

**Marker struct**: `pub struct CultsStorageDef;`

**ObjectType** (from objects.rs:331-337):
```
name: "Storage", size: (3,2), destructible: true, sight_range: 3, groupable: true
```

**BaseStats** (from structures.rs cults_structure_stats:486-488):
```
max_hp: STORAGE_MAX_HP (200), point_armor: STORAGE_POINT_ARMOR (1), full_armor: STORAGE_FULL_ARMOR (4), faction: TheCults
```

**StructureStats**:
```
power_value: 0, build_radius: 0, symmetry: ABAB
```

**cost()**: `Some(StructureCost { space_crystals: 0, build_frames: STORAGE_BUILD_FRAMES (300) })`
Note: Cults buildings cost Recruits (consumed on completion), not crystals. The StructureCost only tracks SC cost (0) and build time.

**Command slots**: `&[]` — CultsStorage is an inert drop-off point with no command panel buttons.

**spawn()** (from utils.rs:993-1041):
Insert: Transform (with rotation + flip_scale), ObjectInstance (destructible/under_construction), StructureInstance { rotation, flip_horizontal: false, flip_vertical: false }, Owner, Selectable, SelectionBounds::from_dimensions(rot_x, 0.6, rot_z), GridPosition, SightRange(3).
Note: Storage uses `rotated_building_size(3, 2, &rotation)` from `crate::game::world::utils`. SpawnParams doesn't have flip_horizontal/flip_vertical — use defaults (false).
For under_construction: also insert ConstructionHP::new(STORAGE_BUILD_FRAMES) and CultsConstructionState::new(STORAGE_BUILD_FRAMES) — from `crate::game::types::structures`.

#### CultsArmory (armory.rs)

**Marker struct**: `pub struct CultsArmoryDef;`

**ObjectType** (from objects.rs:338-344):
```
name: "Armory", size: (3,2), destructible: true, sight_range: 3, groupable: true
```

**BaseStats** (from structures.rs cults_structure_stats:475-477):
```
max_hp: ARMORY_MAX_HP (300), point_armor: ARMORY_POINT_ARMOR (1), full_armor: ARMORY_FULL_ARMOR (4), faction: TheCults
```

**StructureStats**:
```
power_value: 0, build_radius: 0, symmetry: ABCB
```

**cost()**: `Some(StructureCost { space_crystals: ARMORY_SC_COST (150), build_frames: 0 })`
Note: Armory has an SC cost (150 placeholder) but no explicit build_frames constant. Use 0 or add a constant (check if Armory uses Cults construction which has its own frame tracking).

**Command slots** (from grid_layout.rs:120-126):
```
StructureMenuState::ArmoryMenu:
  (0,0) => ArmoryTrainSoldier
  (0,1) => ArmoryTrainGunner
  (0,2) => ArmoryEjectAll
  (2,2) => SetRallyPoint
```
Static slice: 4 entries.

**spawn()** (from utils.rs:1099-1149):
Insert: Transform (with rotation + flip_scale), ObjectInstance::destructible(CultsArmory, ARMORY_MAX_HP), StructureInstance { rotation, flip_h, flip_v }, Owner, Selectable, SelectionBounds::from_dimensions(rot_x, 0.8, rot_z), GridPosition, SightRange(3), ArmoryState::default(), Armor { point_armor: ARMORY_POINT_ARMOR as f32, full_armor: ARMORY_FULL_ARMOR as f32, directional_armor: false }.
Note: Armory is the only Cults building that inserts Armor component. Import from `crate::game::combat::types::Armor`.
Note: ArmoryState from `crate::game::types::structures`.

### Key Import Paths

- Traits + types: `crate::game::entities::types::{EntityDefinition, StructureDefinition, BaseStats, StructureStats, CommandSlotDef, SpawnParams}`
- Cults stats: `crate::game::types::structures::cults_structure_stats::*`
- State components: `crate::game::types::structures::{RecruitmentCenterState, ArmoryState, CultsConstructionState, ConstructionHP, StructureCost}`
- Shared types: `crate::shared::types::{Selectable, SelectionBounds, GridPosition, SightRange}`
- StructureInstance: `crate::game::types::StructureInstance` (re-exported) or `crate::shared::types::StructureInstance`
- ObjectInstance: `crate::game::types::ObjectInstance`
- UI types: `crate::ui::types::{ObjectInterfaceState, StructureMenuState, CommandButtonAction}`
- Armor: `crate::game::combat::types::Armor` (only for Armory)
- FactionEnum::TheCults: `crate::types::FactionEnum`
- SymmetryTypeEnum: `crate::types::SymmetryTypeEnum`

### Registry Integration

In `artifacts/developer/src/game/entities/registry.rs`:
- Add imports: `use super::factions::cults::buildings::recruitment_center::RecruitmentCenterDef;` (and similarly for storage/armory)
- Add registration calls after the GDO structures block:
  ```rust
  // Cults structures
  registry.register_structure::<RecruitmentCenterDef>();
  registry.register_structure::<CultsStorageDef>();
  registry.register_structure::<CultsArmoryDef>();
  ```
- Update test assertion: `assert_eq!(registry.len(), 13);` (was 10)

### Test Pattern

Follow GDO base.rs test pattern — tests per entity:
- `rc_base_stats` — verify HP=1000, point_armor=1, full_armor=16, faction=TheCults
- `rc_object_type` — verify name, size=(4,4), destructible, sight_range=6, !groupable
- `rc_structure_stats` — verify power=0, build_radius=0, symmetry=AAAA
- `rc_cost_is_none` — primary structure, no cost
- `rc_command_slots_count` — 2 slots
- `rc_registry_lookup` — register + get + assert
Similar pattern for Storage (0 SC cost, 300 build_frames, 0 command slots) and Armory (150 SC cost, 4 command slots).

## Dependencies

- **entity-registry-grid-layout** (pending sibling task) — May refine CommandSlotDef or command slot patterns. However, the current CommandSlotDef type is stable and already used by 6 GDO entity definitions, so this task can proceed independently.
- **entity-definition-cleanup** (pending sibling task) — Will clean up old scattered definitions after migration. This task creates the new files; cleanup happens later. Keep old spawn functions as thin wrappers per task description.
