# recruitment_area_tile_claiming

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-cults_recruitment_center_and_storage.md

## Task

Implement the Recruitment Area tile claiming system for Recruitment Centers.

### RecruitmentArea Concept
Each Recruitment Center has a 10x10 Recruitment Area centered on its 4x4 footprint. The area contains 100 tiles total. The center claims Recruitable tiles (tiles where `TilePreset.recruitable == true`) within this area that are not already claimed by another active Recruitment Center.

### TileClaimMap Resource
Create a new `TileClaimMap` resource (in `game/world/types.rs` or `game/types/cults.rs`):
- `claims: HashMap<(i32, i32), Entity>` — maps grid position to the RecruitmentCenter entity that claimed it
- Methods: `claim_tile(pos, entity)`, `unclaim_tile(pos)`, `is_claimed(pos) -> Option<Entity>`, `unclaim_all_for(entity)`

### Tile Claiming System
Create `recruitment_tile_claiming_system` that runs each frame (or on structure spawn/destroy events):
1. Query all RecruitmentCenter entities, sorted by `build_order` (ascending = first-built has priority)
2. For each center, compute its 10x10 area (centered on the 4x4 footprint — the area extends 3 tiles beyond each edge)
3. For each tile in the area: if the tile is Recruitable (query TilePreset) and not claimed by a center with lower build_order, claim it for this center
4. Update `RecruitmentCenterState.claimed_tiles` with the list of claimed positions
5. Compute `effectiveness = claimed_tiles.len() as f32 / 100.0` (100 = total tiles in 10x10 area)
6. Update `local_capacity = (20.0 * effectiveness).floor() as u32`

### Reclaim on Destruction
When a Recruitment Center is destroyed (entity despawned or HP reaches 0):
- Remove all its claims from TileClaimMap
- The claiming system will naturally reassign freed tiles to other centers on next run (by build_order priority)

Use `RemovedComponents<RecruitmentCenterState>` or a destruction cleanup system to detect center removal and call `unclaim_all_for(entity)`.

### Registration
Register the system in the appropriate plugin (WorldPlugin or a new CultsPlugin) to run in the game update phase.

### Tests
- Place one RC on all-Recruitable terrain: effectiveness = 1.0, capacity = 20
- Place one RC where 50 of 100 tiles are Recruitable (e.g., half Water): effectiveness = 0.5, capacity = 10
- Place two RCs with overlapping areas: first-built claims contested tiles, second gets reduced effectiveness
- Destroy first RC: second reclaims freed tiles, effectiveness increases
- TileClaimMap correctly tracks claims and unclaims

## Technical Context

### Files to Create/Modify

1. **`artifacts/developer/src/game/world/types.rs`** — Add `TileClaimMap` resource
   - This file already contains map-level resources: `GridMap` (line 8), `FogOfWarMap`, `ElevationMap`, etc.
   - Add `use std::collections::HashMap;` (already imported on line 2)
   - Define `TileClaimMap` struct with `#[derive(Resource, Default)]` and the HashMap + methods

2. **`artifacts/developer/src/game/world/faction.rs`** — Add `recruitment_tile_claiming_system` and `recruitment_center_destruction_cleanup_system`
   - This file already contains production/construction tick systems (e.g., `dc_construction_tick_system`, `barracks_production_tick_system`)
   - Imports already include: `Tile`, `TilePreset` (line 10), `GridPosition` (via types), `spawn_recruitment_center` (line 8)
   - Need to add import for `TileClaimMap` from `super::types`

3. **`artifacts/developer/src/game/world/mod.rs`** — Register systems and resource
   - Add `app.init_resource::<types::TileClaimMap>();` in `FactionPlugin::build()` (around line 68)
   - Register `recruitment_tile_claiming_system` in the `Update` systems block under `DiagCategory::Faction` (line 83-106)
   - Register `recruitment_center_destruction_cleanup_system` in `FixedUpdate` under `DiagCategory::Construction` (line 107-118) or in `Update` — it should run before the claiming system

### Key Types and Components

- **`RecruitmentCenterState`** (`game/types/structures.rs:494`): Already has `claimed_tiles: Vec<(i32, i32)>`, `effectiveness: f32`, `local_capacity: u32`, `build_order: u64` — these fields are ready to be populated by the claiming system
- **`RecruitmentCenterCounter`** (`game/types/structures.rs:479`): Resource for assigning monotonic build_order, already initialized in FactionPlugin (mod.rs:68)
- **`GridPosition`** (`shared/types.rs:69`): Component with `x: i32, z: i32` — used on both tiles and structures
- **`TilePreset`** (`game/world/types.rs:117`): Component on tiles with `recruitable: bool` field (line 125)
- **`TilePresetEnum`** (`game/world/types.rs:38`): Enum component also on tiles. All presets except `Water` are recruitable
- **`Tile`** (`game/world/types.rs:159`): Marker component on tile entities
- **`ObjectInstance`** (`game/types/objects.rs`): Has `is_alive()` method for checking if structure is destroyed

### Query Patterns

For tiles: `Query<(&GridPosition, &TilePreset), With<Tile>>` — this exact pattern is used in `faction.rs` line 357 and line 431
For RCs: `Query<(Entity, &GridPosition, &mut RecruitmentCenterState, &ObjectInstance)>` — entity needed for TileClaimMap claims, ObjectInstance for alive check

### 10x10 Area Calculation

RC footprint is 4x4 starting at `GridPosition { x, z }`. The 10x10 area centered on this means:
- x range: `grid_pos.x - 3` to `grid_pos.x + 6` (inclusive, 10 tiles)
- z range: `grid_pos.z - 3` to `grid_pos.z + 6` (inclusive, 10 tiles)
- The 4x4 footprint occupies `[x, x+3] x [z, z+3]`, so 3 tiles of padding on each side centers the 10x10 area

Grid bounds: 64x64 grid (0..63), so clamp to valid range.

### Destruction Cleanup

Two approaches (prefer approach B for simplicity):

**A) RemovedComponents**: Use `RemovedComponents<RecruitmentCenterState>` — fires when component is removed. However, in Bevy 0.17 `RemovedComponents` only gives you the Entity, not the component data. Since `unclaim_all_for(entity)` only needs the entity, this works.

**B) Pre-despawn check**: In the claiming system itself, clear stale claims at the start: iterate the TileClaimMap, remove entries whose entity is no longer alive (use `Query<&ObjectInstance>` to check). This is simpler and avoids ordering issues with `remove_dead_entities_system` (`combat/systems/core.rs:757`) which despawns dead entities.

### System Ordering

- `recruitment_tile_claiming_system` should run in `Update` / `DiagCategory::Faction` — same phase as other faction management systems
- If using approach B (stale cleanup in claiming system), no explicit ordering needed relative to `remove_dead_entities_system` since it self-heals
- If using approach A, the cleanup system must run AFTER `remove_dead_entities_system` (which runs in `FixedUpdate` / `DiagCategory::Combat` — mod.rs:42) — cross-schedule ordering can be tricky, prefer approach B

### Testing Patterns

- Use `TestHarness` (`shared/testing/harness.rs`) for test setup
- `harness.set_tile(grid_x, grid_z, TilePresetEnum::Water)` to set non-recruitable tiles (line 365)
- Tiles are spawned by `map::spawn_grid` which runs on `OnEnter(AppState::InGame)` — tests using `App::new()` need `MinimalPlugins` and the MapPlugin or manual tile spawning
- For spawning RC in tests: use `spawn_recruitment_center()` from `game/utils.rs:947` — requires `&mut Commands, &mut ResMut<Assets<Mesh>>, &mut ResMut<Assets<StandardMaterial>>, grid_x, grid_z, owner, build_order`
- Alternative: spawn minimal test entities with just `(GridPosition, RecruitmentCenterState, ObjectInstance::destructible(ObjectEnum::RecruitmentCenter, RC_MAX_HP))` to avoid mesh dependencies

### Existing Constants

- `RC_MAX_HP`: defined in structures.rs alongside other structure HP constants
- Grid is 64x64, cell_size=1.0, half_size=32 (world coords: -32..+32)

## Dependencies

- **Existing `RecruitmentCenterState` fields** (structures.rs:494-522): Already defined with `claimed_tiles`, `effectiveness`, `local_capacity`, `build_order` — no structural changes needed, just populating these fields at runtime.
- **`spawn_recruitment_center()`** (utils.rs:947): Already spawns RC with `RecruitmentCenterState` including `build_order` — no changes needed.
- **Tile system** (`map::spawn_grid`): Tiles must exist with `TilePreset` components for the claiming system to query. Already functional.
- **Sibling tasks**: `recruitment_center_auto_production` reads `local_capacity` and `effectiveness` from `RecruitmentCenterState` — those values are populated by THIS task's system. `cults_unit_control_tracking` reads/writes `local_used` — independent of tile claiming.
