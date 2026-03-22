# storage_structure

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-cults_recruitment_center_and_storage.md

## Task

Add the Storage structure to the Cults faction.

### ObjectEnum
Add `ObjectEnum::CultsStorage` variant to `shared/types.rs`. Use 'CultsStorage' to avoid ambiguity with any generic 'Storage' concept.

### ObjectType
Implement `object_type()` for CultsStorage:
- Size: 3x2
- SymmetryType: ABAB
- Destructible: true
- ConstructionHP rule: applies
- SightRange: 3 (small utility building, matches PowerPlant)
- Groupable: true (multiple can be selected together)
- Faction: TheCults

Define constants: STORAGE_MAX_HP (200 — small building), STORAGE_POINT_ARMOR (1), STORAGE_FULL_ARMOR (4).

### StorageState Component (optional, minimal)
No complex state needed for now. Storage is a passive drop-off point. If desired, add a marker component `CultsStorageMarker` to distinguish from other structures for drop-off targeting.

### spawn_cults_storage Function
In `game/utils.rs`, add `spawn_cults_storage()` following the pattern of `spawn_barracks()` (3x2 building):
- 3x2 cuboid mesh, dark purple material
- Components: ObjectInstance::destructible, StructureInstance, Owner, Selectable, SelectionBounds, GridPosition, SightRange
- ABAB side labels (like Barracks uses ABAC — Storage uses ABAB per design)
- Label: "Storage"

### Simultaneous Drop-Off
The design specifies multiple Recruits can drop off Space Crystals simultaneously. This is primarily a behavior system concern (the gathering/drop-off behaviors for Recruits, which will come in a separate feature). For now, just note in a code comment that Storage supports concurrent drop-off (no queuing/blocking needed at the structure level).

### Tests
- CultsStorage object_type() returns correct size (3x2), symmetry (ABAB), destructible
- spawn_cults_storage creates entity with all expected components
- CultsStorage is groupable

## Technical Context

### Files to modify:

1. **`artifacts/developer/src/shared/types.rs`** (line ~334):
   - Add `CultsStorage` variant to `ObjectEnum` enum under a `// Cults Structures` comment section (currently only has `RecruitmentCenter` at line 334)

2. **`artifacts/developer/src/game/types/objects.rs`**:
   - **`object_type()` match** (~line 302): Add `ObjectEnum::CultsStorage` arm after `RecruitmentCenter`, returning `ObjectType { name: "Storage".to_string(), size: (3, 2), destructible: true, sight_range: 3, groupable: true }`
   - **`structure_type()` match** (~line 361): Add `ObjectEnum::CultsStorage` arm returning `Some(StructureType { object_type: self.object_type(), symmetry_type: SymmetryTypeEnum::ABAB })`
   - **`is_unit()`** (line 376): No change needed — CultsStorage is not a unit
   - **Update test arrays**: Add `ObjectEnum::CultsStorage` to the `structures` array in `test_all_structures_have_valid_sizes` (line 700) and the `all_variants` array in `test_all_object_enum_variants_have_valid_object_type` (line 751)

3. **`artifacts/developer/src/game/types/structures.rs`** (~line 468):
   - Add constants to `cults_structure_stats` module:
     ```rust
     // Storage
     pub const STORAGE_MAX_HP: f32 = 200.0;
     pub const STORAGE_POINT_ARMOR: u32 = 1;
     pub const STORAGE_FULL_ARMOR: u32 = 4;
     ```

4. **`artifacts/developer/src/game/utils.rs`** (after line 983, end of file):
   - Add `spawn_cults_storage()` function following `spawn_barracks()` pattern (line 247-298)
   - Key differences from barracks:
     - Size: (3, 2) same as barracks
     - Takes `rotation`, `flip_horizontal`, `flip_vertical` params (ABAB symmetry supports 2 distinct orientations)
     - Mesh: `Cuboid::new(3.0, 0.6, 2.0)` (smaller height than barracks 0.8 — it's a small utility building)
     - Material: Dark purple `Color::srgb(0.4, 0.15, 0.5)` (slightly darker than Cults purple 0.5/0.2/0.6 used for RC)
     - Uses `STORAGE_MAX_HP` from `cults_structure_stats`
     - No `PowerValue` or `BuildRadiusExtension` (Storage is passive)
     - No state component needed (or add `CultsStorageMarker` if desired)
     - Label: "Storage"
     - Side labels: `SymmetryTypeEnum::ABAB` with dimensions `(1.5, 1.0, 0.8)` (same as barracks)
   - World position calculation: `(grid_x as f32 - 32.0) + (rot_x as f32) / 2.0` (uses `rotated_building_size`)
   - Comment that Storage supports concurrent drop-off

### Patterns to follow:
- **ObjectEnum exhaustive matches**: After adding `CultsStorage`, ensure all match arms that don't use `_` wildcard are updated. Key locations to check: `is_unit()`, `is_resource()`, `unit_control_cost()` in objects.rs — all use wildcard catch-all so no changes needed
- **Spawn function signature**: Follow `spawn_barracks` exactly — `(commands, meshes, materials, grid_x, grid_z, owner, rotation, flip_horizontal, flip_vertical) -> Entity`
- **Import in utils.rs**: `cults_structure_stats::*` is already imported at line 8
- **Export**: Add `spawn_cults_storage` to the pub use in `game/world/faction.rs` line 5 (the import line for spawn functions) when it needs to be called from faction setup — or leave it pub and importable

### Bevy ECS notes:
- No new systems needed — Storage is a passive entity
- No plugin registration changes needed
- Tests can use the same pattern as existing object_type tests (pure function tests, no App needed)
- For spawn tests, use the `TestApp` harness from `shared/testing/test_app.rs` if integration testing is needed, or follow the simpler pattern of just checking returned entity has expected components

## Dependencies

None. This is a standalone structure definition task. The Storage entity will later be used by the Recruit gathering/drop-off behavior system (separate feature), but this task has no prerequisites.
