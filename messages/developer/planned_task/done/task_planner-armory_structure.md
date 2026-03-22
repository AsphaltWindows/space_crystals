# armory_structure

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-add_cults_armory.md

## Task

Add the Cults Armory structure to the game.

### ObjectEnum
Add `ObjectEnum::CultsArmory` variant.

### ObjectType
- Faction: TheCults
- Size: 3x2 grid units
- SymmetryType: ABCB (entrance side A, matching long sides B, exit side C)
- MaxHP: TBD (use placeholder, e.g. 300)
- Destructible: true
- SightRange: TBD (use placeholder, e.g. 3)
- Groupable: true

### StructureType
- PointArmor: TBD (use placeholder, e.g. 1)
- FullArmor: TBD (use placeholder, e.g. 4)
- Symmetry: ABCB

### ArmoryState component
Define `ArmoryState` with:
- `stored_recruits: Vec<Entity>` (max 10, the internal Recruit pool)
- `training_queue: Option<ObjectEnum>` (current training order — use simple one-at-a-time for now since queue design is TBD)
- `training_progress: u32` (frames elapsed)
- `rally_point: Option<RallyTarget>` (reuse existing rally point pattern from BarracksState/HeadquartersState)

### spawn_cults_armory function
Follow existing spawn patterns (spawn_barracks, spawn_headquarters). Spawn with:
- ObjectInstance::destructible (or under_construction with ConstructionHP rule)
- StructureInstance
- ArmoryState (default empty)
- All standard structure components (Owner, FactionEnum, GridPosition, etc.)

### Constants
Define placeholder constants:
- ARMORY_MAX_HP
- ARMORY_POINT_ARMOR, ARMORY_FULL_ARMOR
- ARMORY_SC_COST (TBD placeholder)
- ARMORY_INTERNAL_RECRUIT_CAPACITY = 10
- SOLDIER_TRAINING_COST (TBD placeholder), SOLDIER_TRAINING_FRAMES (TBD placeholder)
- GUNNER_TRAINING_COST (TBD placeholder), GUNNER_TRAINING_FRAMES (TBD placeholder)

### Tests
- ObjectType returns correct size/symmetry/faction
- StructureType returns correct armor values
- ArmoryState default is empty (no recruits, no training, no rally)
- spawn function creates entity with all expected components

## Technical Context

### Files to Modify

1. **`artifacts/developer/src/shared/types.rs`** (line ~337):
   - Add `CultsArmory` variant to `ObjectEnum` enum, in the `// Cults Structures` section after `CultsStorage` (line 337).
   - **ABCB symmetry**: The `SymmetryTypeEnum` (line 446) does NOT currently have an `ABCB` variant. You must add `ABCB` to the enum. ABCB has two opposite sides matching (the B pair) plus two unique sides (A and C), so it allows non-square sizes. Also update the `validate_size()` method in `StructureType` (`artifacts/developer/src/game/types/objects.rs` line 33) — ABCB should NOT require square dimensions (it's like ABAB/ABAC/ABCD).

2. **`artifacts/developer/src/game/types/objects.rs`**:
   - Add `ObjectEnum::CultsArmory` arm to `object_type()` (line ~216): `ObjectType { name: "Armory".to_string(), size: (3, 2), destructible: true, sight_range: 3, groupable: true }`.
   - Add `ObjectEnum::CultsArmory` arm to `structure_type()` (line ~341): `StructureType { object_type: self.object_type(), symmetry_type: SymmetryTypeEnum::ABCB }`.
   - `is_structure()` (line 388) works via `structure_type().is_some()`, so no change needed — automatically works.
   - `is_unit()` (line 393) — no change needed (Armory is not a unit).

3. **`artifacts/developer/src/game/types/structures.rs`**:
   - Add constants to `cults_structure_stats` module (line ~468):
     ```rust
     // Armory
     pub const ARMORY_MAX_HP: f32 = 300.0;
     pub const ARMORY_POINT_ARMOR: u32 = 1;
     pub const ARMORY_FULL_ARMOR: u32 = 4;
     pub const ARMORY_SC_COST: u32 = 150; // TBD placeholder
     pub const ARMORY_INTERNAL_RECRUIT_CAPACITY: usize = 10;
     pub const SOLDIER_TRAINING_COST: u32 = 75; // TBD placeholder
     pub const SOLDIER_TRAINING_FRAMES: u32 = 160; // TBD placeholder
     pub const GUNNER_TRAINING_COST: u32 = 100; // TBD placeholder
     pub const GUNNER_TRAINING_FRAMES: u32 = 200; // TBD placeholder
     ```
   - Define `ArmoryState` component after `RecruitmentCenterState` (line ~528):
     ```rust
     #[derive(Component, Clone, Debug, Default)]
     pub struct ArmoryState {
         pub stored_recruits: Vec<Entity>,
         pub training_queue: Option<ObjectEnum>,
         pub training_progress: u32,
         pub rally_point: Option<RallyTarget>,
     }
     ```
   - **Rally point type**: The task says `RallyPointTarget` but this type does NOT exist. The existing pattern uses `RallyTarget` (defined at line 57 of structures.rs): `enum RallyTarget { Location(Vec3), Object(Entity) }`. Use `Option<RallyTarget>` — this matches `BarracksState.rally_point` (line 136).
   - Add `use crate::types::ObjectEnum;` if not already imported (it IS imported at line 4).

4. **`artifacts/developer/src/game/utils.rs`**:
   - Add `spawn_cults_armory()` function after `spawn_cults_storage()` (line ~1036). Follow the `spawn_cults_storage()` pattern closely since both are 3x2 Cults structures with rotation/flip support:
     - Parameters: `commands, meshes, materials, grid_x, grid_z, owner, rotation, flip_horizontal, flip_vertical`
     - Use `(base_x, base_z) = (3u32, 2u32)` and `rotated_building_size()`
     - Mesh: `Cuboid::new(3.0, 0.8, 2.0)` (similar to Barracks)
     - Material: Cults purple-ish color (e.g. `Color::srgb(0.55, 0.2, 0.55)`)
     - Components: `ObjectInstance::destructible(ObjectEnum::CultsArmory, ARMORY_MAX_HP)`, `StructureInstance { rotation, flip_horizontal, flip_vertical }`, `owner`, `Selectable`, `SelectionBounds::from_dimensions()`, `GridPosition`, `SightRange`, `ArmoryState::default()`, `Armor { point_armor: ARMORY_POINT_ARMOR as f32, full_armor: ARMORY_FULL_ARMOR as f32, directional_armor: false }`
     - Children: `spawn_structure_label(parent, "Armory", height)`, `spawn_side_labels(parent, SymmetryTypeEnum::ABCB, ...)`
   - Import `ArmoryState` — add to the existing structures import line (line 22-25).
   - Import `ARMORY_*` constants — already covered by `use super::types::cults_structure_stats::*;` (line 8).

5. **`artifacts/developer/src/game/types/objects.rs`** (tests section):
   - Add tests for `CultsArmory` ObjectType and StructureType. Follow existing test patterns (see tests for `CultsStorage` or `Barracks`).

### Key Patterns to Follow

- **Spawn function pattern**: See `spawn_cults_storage()` (utils.rs:988-1036) — identical structure for a 3x2 Cults building with rotation/flip. Copy this pattern.
- **Armor component**: `Armor` from `game/combat/types.rs` line 183: `Armor { point_armor: f32, full_armor: f32, directional_armor: bool }`. See how it's spawned on units at utils.rs:933-934. Structures with armor should spawn with this component.
- **Test pattern for spawn**: See `spawn_cults_storage_creates_entity_with_expected_components` (utils.rs:1083) — uses `TestApp::new()`, `run_system_once`, `step()`, then verifies components.
- **Default impl**: `ArmoryState` can derive `Default` (all fields have natural defaults: empty Vec, None, 0).

### Important Notes

- The task says `RallyPointTarget` but this type doesn't exist — use `RallyTarget` instead (structures.rs:57).
- `SymmetryTypeEnum::ABCB` must be ADDED to the enum — it's not there yet. Look at the `validate_size()` logic (objects.rs:33-48): ABCB allows non-square sizes (like ABAB/ABAC/ABCD), so it should NOT be in the `requires_square` match.
- The test "StructureType returns correct armor values" — note that `StructureType` itself has NO armor fields. Armor is a separate `Armor` component. The test should verify the armor constants and/or that the spawn function attaches the `Armor` component with correct values.
- CultsStorage does NOT have an `Armor` component in its spawn. However, Barracks units at utils.rs:482-483 DO spawn with armor. The Armory should spawn with `Armor` since it has specified PointArmor/FullArmor values.

## Dependencies

- **CultsRecruit (ObjectEnum::CultsRecruit)**: Already exists (shared/types.rs:334). The `stored_recruits: Vec<Entity>` field references Recruit entities, but this is just Entity references — no compile-time dependency.
- **Soldier/Gunner ObjectEnum variants**: Do NOT exist yet. `training_queue: Option<ObjectEnum>` will reference these, but since it's `Option<ObjectEnum>` and training is not implemented in this task, no new variants are needed now.
- No other planned_tasks are required before this one — it is standalone structural scaffolding.
