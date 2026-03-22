# recruitment_center_auto_production

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-cults_recruitment_center_and_storage.md

## Task

Implement the auto-production system for Recruitment Centers and HUD unit control aggregation.

### Production System
Create `recruitment_center_production_system` in `game/world/faction.rs` (or a new cults module):

Each frame, for each RecruitmentCenter with effectiveness > 0:
1. Check if `local_used < local_capacity` — if at capacity, skip (do not produce)
2. Compute production frames needed: `base_frames = 12 * FRAMES_PER_SECOND` (12 seconds at 100%), scaled by effectiveness: `required_frames = (base_frames as f32 / effectiveness).ceil() as u32`. At 50% effectiveness, production takes 24 seconds (384 frames at 16 FPS).
3. Increment `production_progress += 1`
4. When `production_progress >= required_frames`:
   - Reset `production_progress = 0`
   - Spawn a Recruit unit at the center's grid position (use a placeholder spawn — the Recruit unit type will be defined in a future feature; for now spawn a minimal entity with ObjectEnum::CultsRecruit if the variant exists, or leave a clear TODO/stub)
   - Add `OriginatingCenter(center_entity)` component to the spawned Recruit (see unit_control_tracking task)
   - Increment `local_used += 1`
   - If `rally_point` is set, issue a Move command to the spawned Recruit

### Recruit Unit Stub
Add `ObjectEnum::CultsRecruit` to `shared/types.rs` with minimal object_type data:
- Size: 1x1 (unit, not structure)
- Groupable: true
- Faction: TheCults
- This is a placeholder — full Recruit stats/behaviors will come in a separate feature

Add a minimal `spawn_cults_recruit()` function that spawns a unit entity with ObjectEnum::CultsRecruit, Owner, basic Transform, Unit marker, Selectable, SelectionBounds.

### HUD Unit Control Aggregation
Update `update_resource_bar_system` in `hud.rs` for TheCults faction:
- Query all RecruitmentCenterState components owned by the local player
- Sum `local_capacity` across all centers → `unit_control_available`
- Sum `local_used` across all centers → `unit_control_used`
- Update CultsPlayerResources: `unit_control_available = sum_capacity`, `unit_control_used = sum_used`
- Display format: "UC: {used} / {available}"

Alternatively, create a dedicated `cults_unit_control_aggregation_system` that syncs RecruitmentCenterState totals into CultsPlayerResources each frame, and let the existing HUD display read from CultsPlayerResources as it already does.

### Registration
Register the production system and aggregation system. Production should run in the same phase as other production systems (barracks_production_tick_system, headquarters_production_tick_system).

### Tests
- RC at 100% effectiveness: produces a Recruit every 192 frames (12s * 16fps)
- RC at 50% effectiveness: produces a Recruit every 384 frames
- Production stops when local_used == local_capacity
- Production resumes when local_used drops below local_capacity (e.g., unit dies)
- HUD aggregation: 2 centers with capacity 20 and 10 → displays "UC: 0 / 30"
- Recruit spawns at center position and receives rally command if rally_point set

## Technical Context

### Files to Change

1. **`artifacts/developer/src/shared/types.rs`** (line ~316):
   - Add `CultsRecruit` variant to `ObjectEnum` enum (after `CultsStorage`, line 335). It goes in the Cults section alongside existing `RecruitmentCenter` and `CultsStorage`.

2. **`artifacts/developer/src/game/types/objects.rs`** (line ~302):
   - Add `ObjectEnum::CultsRecruit` arm to `object_type()` match (after `CultsStorage`, line ~309): `ObjectType { name: "Recruit", size: (1,1), destructible: true, sight_range: 3, groupable: true }`.
   - The `structure_type()` match does NOT need a new entry — CultsRecruit is a unit, not a structure.
   - The `is_structure()` method auto-returns false since `structure_type()` returns None.

3. **`artifacts/developer/src/game/utils.rs`** (after `spawn_cults_storage`, line ~1040):
   - Add `spawn_cults_recruit()` function. Follow the pattern of `spawn_peacekeeper()` (line 398-492) but much simpler:
     - Takes `commands, meshes, materials, grid_x, grid_z, owner` params.
     - World position: `(grid_x as f32 - 32.0) + 0.5`, same as peacekeeper.
     - Spawns with: `Mesh3d`, `MeshMaterial3d` (use Cults purple `Color::srgb(0.5, 0.2, 0.6)`), `Transform`, `Unit`, `ObjectInstance::destructible(ObjectEnum::CultsRecruit, RECRUIT_MAX_HP)`, `owner`, `UnitType { name: "Recruit" }`, `Selectable`, `SelectionBounds::unit()`, `GridPosition`, `UnitBaseEnum::LightInfantry` (placeholder), `UnitCommand::Idle`.
     - Define `RECRUIT_MAX_HP` constant (e.g., 50.0 placeholder) near the spawn function.
     - Do NOT add attack components — Recruit is unarmed.
     - Do NOT add movement params yet — stub unit, movement will come in a future feature.
     - Return `Entity`.

4. **`artifacts/developer/src/game/world/faction.rs`** (production system):
   - Add `recruitment_center_production_system`. Place it near the other production systems (after `headquarters_production_tick_system`, around line ~600).
   - **Pattern to follow**: `barracks_production_tick_system` (line 351-420) is the closest pattern, but simpler — RC has no build queue, no power ratio, and uses integer frame counting.
   - System signature:
     ```rust
     pub fn recruitment_center_production_system(
         mut commands: Commands,
         mut meshes: ResMut<Assets<Mesh>>,
         mut materials: ResMut<Assets<StandardMaterial>>,
         mut rc_query: Query<(Entity, &Owner, &GridPosition, &mut RecruitmentCenterState)>,
         tiles: Query<(&GridPosition, &TilePreset), With<Tile>>,
         grid: Res<super::types::GridMap>,
         rally_targets: Query<(&Transform, &Owner), With<ObjectInstance>>,
         occupancy: Res<crate::game::units::types::OccupancyMap>,
     )
     ```
   - Key logic: For each RC with `effectiveness > 0.0` and `local_used < local_capacity`:
     - `let base_frames = 12 * FRAMES_PER_SECOND;` (import from `crate::simulation::FRAMES_PER_SECOND`, value=16, so base_frames=192)
     - `let required_frames = (base_frames as f32 / rc_state.effectiveness).ceil() as u32;`
     - Increment `rc_state.production_progress += 1`
     - When `>= required_frames`: reset, spawn recruit, add `OriginatingCenters { centers: vec![center_entity] }`, increment `local_used`
     - If `rc_state.rally_point` is `Some(pos)` — it's `Option<Vec3>` (not `RallyTarget`), so directly pathfind+issue Move command (simpler than `issue_rally_command` which handles `RallyTarget::Object`). Use `find_path_for_domain`, `smooth_path`, insert `MoveTarget`, `Path`, `UnitCommand::Move(pos)`.
   - Import `spawn_cults_recruit` from `crate::game::utils`.
   - Import `FRAMES_PER_SECOND` from `crate::simulation`.

5. **`artifacts/developer/src/game/world/faction.rs`** (aggregation system):
   - Add `cults_unit_control_aggregation_system`:
     ```rust
     pub fn cults_unit_control_aggregation_system(
         rc_query: Query<(&Owner, &RecruitmentCenterState)>,
         mut players: Query<(&Player, &mut CultsPlayerResources)>,
     )
     ```
   - For each player with CultsPlayerResources, sum `local_capacity` and `local_used` from all RCs owned by that player.
   - Write sums into `CultsPlayerResources.unit_control_available` and `unit_control_used`.
   - The HUD already displays these fields correctly at `hud.rs:1350-1352`: `format!("UC: {} / {}", res.unit_control_used, res.unit_control_available)`.

6. **`artifacts/developer/src/game/world/mod.rs`** (line ~107-118):
   - Register both new systems in `FixedUpdate` under `DiagCategory::Construction` alongside existing production systems:
     ```
     faction::recruitment_center_production_system,
     faction::cults_unit_control_aggregation_system,
     ```

7. **`artifacts/developer/src/ui/hud.rs`** — NO changes needed. The TheCults branch (line 1343-1356) already reads from `CultsPlayerResources.unit_control_used` and `unit_control_available` and formats as "UC: {used} / {available}".

### Key Types & Components

- **`RecruitmentCenterState`** (`game/types/structures.rs:499`): Has `effectiveness: f32`, `local_capacity: u32`, `local_used: u32`, `production_progress: u32`, `rally_point: Option<Vec3>`, `build_order: u64`, `claimed_tiles: Vec<(i32, i32)>`.
- **`CultsPlayerResources`** (`game/types/factions.rs:171`): Has `space_crystals: i32`, `unit_control_used: u32`, `unit_control_available: u32`. Method `has_unit_control(cost)` already exists.
- **`RecruitmentCenterCounter`** (`game/types/structures.rs:484`): Resource for monotonic build_order. Already initialized in `WorldPlugin` (mod.rs:68).
- **`FRAMES_PER_SECOND`** (`simulation/mod.rs:15`): Value is 16.
- **`OriginatingCenters`** — defined by the sibling `cults_unit_control_tracking` task. Use `OriginatingCenters { centers: vec![center_entity] }`. If the component doesn't exist yet, define a minimal version or add a TODO.
- **`Owner`** (`shared/types.rs`): Has `player_number() -> Option<u32>`.
- **`Player`** (`game/types/mod.rs`): Has `player_number: u32`, `faction: FactionEnum`.

### Rally Point Pattern

RC's `rally_point` is `Option<Vec3>` — simpler than the `Option<RallyTarget>` used by Barracks/HQ. No need for the complex `issue_rally_command` helper. Just pathfind to the Vec3 directly:
```rust
if let Some(rally_pos) = rc_state.rally_point {
    let spawn_grid = GridPosition { x: spawn_x, z: spawn_z };
    let target_grid = world_to_grid(rally_pos);
    if let Some(path) = find_path_for_domain(spawn_grid, target_grid, &tiles, &UnitBaseEnum::LightInfantry, grid.width as i32, grid.height as i32, &occupancy, (spawn_x, spawn_z)) {
        let smoothed = smooth_path(path);
        commands.entity(unit_entity).insert((MoveTarget(rally_pos), Path { waypoints: smoothed, current_waypoint: 0 }, UnitCommand::Move(rally_pos)));
    }
}
```

### Spawn Position

RC is 4x4 at GridPosition. The center's world position is `(grid_x - 32 + 2, 0.75, grid_z - 32 + 2)` (see `spawn_recruitment_center` at utils.rs:956-957). Recruit should spawn at the RC's grid position — use `grid_x, grid_z` directly as the spawn grid coords (top-left corner of RC). Alternatively, offset to center of RC: `grid_x + 2, grid_z + 2`. Either works for a placeholder.

### Test Patterns

Follow the existing test patterns in faction.rs (test module starts around line 2500+). Tests use `App::new()` with `MinimalPlugins`, insert resources manually, spawn entities, then `app.update()` in loops.
- Spawn an RC entity with `RecruitmentCenterState { effectiveness: 1.0, local_capacity: 20, ..Default::default() }` and `Owner::player(0)`.
- Add `Assets<Mesh>` and `Assets<StandardMaterial>` resources.
- Add `GridMap` resource.
- Add `OccupancyMap` resource.
- Run 192 frames → assert a CultsRecruit entity was spawned.
- For aggregation test: spawn RC entities, insert `CultsPlayerResources` on a player entity, run system, check resource values.

## Dependencies

- **`cults_unit_control_tracking` (sibling task)**: Defines `OriginatingCenters` component that must be attached to spawned Recruits. This task should either: (a) define a minimal `OriginatingCenters` component itself if built first, or (b) use the component if the sibling task lands first. The production system inserts `OriginatingCenters { centers: vec![center_entity] }` on each spawned Recruit. If building this task first, add `OriginatingCenters` as a simple `#[derive(Component, Clone, Debug)] pub struct OriginatingCenters { pub centers: Vec<Entity> }` in `game/types/structures.rs` or a new `game/types/cults.rs`.
- **Tile claiming system (separate task, not in this batch)**: `RecruitmentCenterState.effectiveness` and `local_capacity` are currently 0.0/0 by default. The production system correctly skips RCs with `effectiveness == 0.0`. For testing, set these fields manually. In production, the tile claiming system (future task) will populate them.
