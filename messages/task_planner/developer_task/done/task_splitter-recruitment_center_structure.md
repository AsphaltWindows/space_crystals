# recruitment_center_structure

## Metadata
- **From**: task_splitter
- **To**: task_planner

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
Update `setup_cults_game_start()\ in `faction.rs` to spawn one RecruitmentCenter at grid position (50, 50) for the Cults player (similar to how GDO spawns a DC and Syndicate spawns a Tunnel+HQ).

### Tests
- RecruitmentCenter object_type() returns correct size, symmetry, destructible, sight_range
- spawn_recruitment_center creates entity with all expected components
- RecruitmentCenterState initializes with 0 progress, empty claimed_tiles, effectiveness 0.0
- setup_cults_game_start spawns a RecruitmentCenter owned by the Cults player
