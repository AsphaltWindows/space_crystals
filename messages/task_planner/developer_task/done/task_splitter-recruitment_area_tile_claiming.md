# recruitment_area_tile_claiming

## Metadata
- **From**: task_splitter
- **To**: task_planner

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
