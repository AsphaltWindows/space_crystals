# recruitment_center_structure

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-cults_recruitment_center_and_storage.md

## Task

Add the Recruitment Center structure to the Cults faction.

### ObjectEnum
Add `ObjectEnum::RecruitmentCenter` variant to `shared/types.rs`.

### ObjectType
Implement `object_type()` for RecruitmentCenter:
- Size: 4x4
- SymmetryType: AAAA
- Destructible: true
- ConstructionHP rule: applies
- SightRange: 6 (matches DC as primary structure)
- Groupable: false (unique primary structure, like DC)
- Faction: TheCults

Define constants: RC_MAX_HP, RC_POINT_ARMOR (1), RC_FULL_ARMOR (16) — same class as DC (4x4 primary structure).

### RecruitmentCenterState Component
Create a `RecruitmentCenterState` component (in `game/types/structures.rs` or a new `game/types/cults.rs`):
- `rally_point: Option<Vec3>` — where produced Recruits move to
- `claimed_tiles: Vec<(i32, i32)>` — grid positions of claimed Recruitable tiles
- `effectiveness: f32` — cached ratio (claimed_recruitable / total_tiles_in_area)
- `local_capacity: u32` — floor(20 * effectiveness), max unit control from this center
- `local_used: u32` — unit control currently used by units originating from this center
- `production_progress: u32` — frames accumulated toward next Recruit
- `build_order: u64` — monotonic counter for first-built priority (use a global AtomicU64 or Resource counter)

### spawn_recruitment_center Function
In `game/utils.rs`, add `spawn_recruitment_center()` following the pattern of `spawn_deployment_center()`:
- 4x4 cuboid mesh, purple-tinted material (Cults faction color)
- Components: ObjectInstance::destructible, StructureInstance, Owner, Selectable, SelectionBounds, GridPosition, SightRange, RecruitmentCenterState (initialized with build_order from counter)
- Label: "Recruitment Center"
- Side labels: AAAA symmetry

### Cults Game Start
Update `setup_cults_game_start()` in `faction.rs` to spawn one RecruitmentCenter at grid position (50, 50) for the Cults player (similar to how GDO spawns a DC and Syndicate spawns a Tunnel+HQ).

### Tests
- RecruitmentCenter object_type() returns correct size, symmetry, destructible, sight_range
- spawn_recruitment_center creates entity with all expected components
- RecruitmentCenterState initializes with 0 progress, empty claimed_tiles, effectiveness 0.0
- setup_cults_game_start spawns a RecruitmentCenter owned by the Cults player

## Technical Context

### Files to Modify

1. **`artifacts/developer/src/shared/types.rs`** (line ~330-336)
   - Add `RecruitmentCenter` variant to `ObjectEnum` enum. Place it in a new section comment block `// Cults Structures` after the Syndicate Structures block (after `Headquarters` on line 332).
   - The enum is at line 316. Current last variant before neutrals is `Headquarters` (line 332).

2. **`artifacts/developer/src/game/types/objects.rs`**
   - Add `object_type()` match arm at ~line 301 (before the neutral resources block starting at line 302):
     ```rust
     ObjectEnum::RecruitmentCenter => ObjectType {
         name: "Recruitment Center".to_string(),
         size: (4, 4),
         destructible: true,
         sight_range: 6,
         groupable: false,
     },
     ```
   - Add `structure_type()` match arm at ~line 353 (before the `_ => None` catch-all at line 354):
     ```rust
     ObjectEnum::RecruitmentCenter => Some(StructureType {
         object_type: self.object_type(),
         symmetry_type: SymmetryTypeEnum::AAAA,
     }),
     ```
   - **Update existing exhaustive test arrays**:
     - `test_all_existing_structures_pass_validation` (line 688): add `ObjectEnum::RecruitmentCenter` to the structures array
     - `test_all_object_enum_variants_have_valid_object_type` (line 738): add `ObjectEnum::RecruitmentCenter` to the all_variants array
   - `is_structure()` (line 359) uses `structure_type().is_some()` — automatically works once structure_type arm is added
   - `is_unit()` (line 364) and `is_resource()` (line 369) won't match — correct, no change needed

3. **`artifacts/developer/src/game/types/structures.rs`**
   - Add a new `cults_structure_stats` module (after `syndicate_structure_stats` which ends at line 465):
     ```rust
     pub mod cults_structure_stats {
         pub const RC_MAX_HP: f32 = 1000.0;  // Same as DC
         pub const RC_POINT_ARMOR: u32 = 1;
         pub const RC_FULL_ARMOR: u32 = 16;
     }
     ```
   - Add `RecruitmentCenterState` component. Pattern to follow: `DeploymentCenterState` (line 74) and `HeadquartersState` (line 256). The component should be:
     ```rust
     #[derive(Component, Clone, Debug)]
     pub struct RecruitmentCenterState {
         pub rally_point: Option<Vec3>,
         pub claimed_tiles: Vec<(i32, i32)>,
         pub effectiveness: f32,
         pub local_capacity: u32,
         pub local_used: u32,
         pub production_progress: u32,
         pub build_order: u64,
     }
     ```
   - Implement `Default` manually (not derive) so `build_order` defaults to 0, all fields zeroed/empty, effectiveness 0.0.
   - For the build_order counter: add a `RecruitmentCenterCounter` Bevy `Resource` with a `next() -> u64` method that increments and returns. This is simpler than AtomicU64 and fits Bevy's single-threaded system model. Initialize it in the plugin or as a default resource.

4. **`artifacts/developer/src/game/types/mod.rs`** (line 10)
   - The existing `pub use structures::*;` (line 10) re-exports everything from structures.rs. New types and the `cults_structure_stats` module will be automatically available via `use crate::game::types::*`.

5. **`artifacts/developer/src/game/utils.rs`**
   - Add `use super::types::cults_structure_stats::*;` to imports (after line 7 where `syndicate_structure_stats` is imported).
   - Add `spawn_recruitment_center()` function (after the last spawn function, `spawn_supply_chopper` ending at ~line 943). Follow the `spawn_deployment_center()` pattern (line 153-190):
     - Parameters: `commands, meshes, materials, grid_x, grid_z, owner, build_order: u64`
     - World position: `(grid_x as f32 - 32.0) + 2.0` for 4x4 center (same as DC)
     - Mesh: `Cuboid::new(4.0, 1.5, 4.0)`
     - Material: Use `Color::srgb(0.5, 0.2, 0.6)` (Cults purple from `FactionEnum::TheCults.color()` in shared/types.rs line 298) rather than `owner.color()` — the task says "purple-tinted material"
     - Components bundle: `ObjectInstance::destructible(ObjectEnum::RecruitmentCenter, RC_MAX_HP)`, `StructureInstance::default()`, `owner`, `Selectable`, `SelectionBounds::from_dimensions(4.0, 1.5, 4.0)`, `GridPosition { x: grid_x, z: grid_z }`, `SightRange(6)`, `RecruitmentCenterState { build_order, ..Default::default() }`
     - Children: `spawn_structure_label(parent, "Recruitment Center", 1.05)` and `spawn_side_labels(parent, SymmetryTypeEnum::AAAA, 2.0, 2.0, 1.5)` (same as DC)
   - Note: DC uses `PowerValue` and `BuildRadiusExtension` components — RC does NOT need these (Cults don't use build areas or power)

6. **`artifacts/developer/src/game/world/faction.rs`**
   - Add `spawn_recruitment_center` to the import list (line 4-8, add to the `use crate::game::utils::{...}` block).
   - Update `setup_cults_game_start()` (line 171-184):
     - Add `mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>` parameters (follow `setup_gdo_game_start` pattern at line 104-107).
     - Add `mut rc_counter: ResMut<RecruitmentCenterCounter>` parameter.
     - Spawn RC at grid (50, 50): `spawn_recruitment_center(&mut commands, &mut meshes, &mut materials, 50, 50, cults_owner, rc_counter.next())`
     - Update the info! log message.
   - The `RecruitmentCenterCounter` resource must be initialized. Options: insert it as a default resource in the world plugin (`artifacts/developer/src/game/world/mod.rs`, line ~60-80 in the `WorldPlugin` impl), or use `init_resource::<RecruitmentCenterCounter>()`.

### Patterns to Follow

- **Spawn function pattern**: See `spawn_deployment_center()` at utils.rs:153-190 — this is the closest analog (4x4, AAAA symmetry, primary structure).
- **Structure state pattern**: See `DeploymentCenterState` (structures.rs:74) and `HeadquartersState` (structures.rs:256) for component structure.
- **Stat constants pattern**: See `gdo_structure_stats` module (structures.rs:390) and `syndicate_structure_stats` module (structures.rs:442) for how to define a new stats module.
- **Game start pattern**: See `setup_gdo_game_start()` (faction.rs:104-130) for spawning a primary structure at game start.
- **Faction color**: TheCults = `Color::srgb(0.5, 0.2, 0.6)` (purple) — defined at shared/types.rs:298.

### Bevy ECS Considerations

- `RecruitmentCenterCounter` should be a `#[derive(Resource, Default)]` with a `u64` inside and a `next(&mut self) -> u64` method. Insert via `app.init_resource::<RecruitmentCenterCounter>()` in the world plugin.
- `setup_cults_game_start` runs in `Startup` schedule, registered at world/mod.rs:73. Its system signature will need `Commands`, `ResMut<Assets<Mesh>>`, `ResMut<Assets<StandardMaterial>>`, and `ResMut<RecruitmentCenterCounter>` added.
- No system ordering changes needed — the existing `.after(map::spawn_grid)` constraint (world/mod.rs:73) is sufficient.

## Dependencies

- **None** — This is a foundational structure definition task. It adds types, spawn logic, and game start integration without depending on other pending tasks. The Recruitment Center's tile claiming, production, and unit control systems will be separate follow-up tasks that depend on this one.
