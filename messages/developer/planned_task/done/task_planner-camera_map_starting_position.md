# camera_map_starting_position

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-camera_starting_position_map_override.md

## Task

Add a `MapStartingPositions` resource (or equivalent) that stores an optional starting camera position (grid coordinates) per player slot index. Add a startup system (run after game start systems that spawn structures) that positions the camera at game start:

1. If `MapStartingPositions` has an entry for the local player's slot, center the camera on that grid position.
2. Otherwise, fall back to centering on the local player's primary structure — Deployment Center for GDO, starting Tunnel for Syndicate.
3. If neither exists (e.g., stub factions with no structures), leave the camera at its default position.

Implementation notes:
- The resource should be inserted during map loading / game setup. For now, it can default to empty (no map-defined positions), which triggers the fallback path.
- Use the existing camera snap formula: given a target grid position (x, z), set camera Transform to (x, cam.y, z + cam.y * 25.0 / 40.0) — this matches the snap logic in hud.rs (Alt-click portrait) and resources.rs (control group double-tap recall).
- Query `LocalPlayer` to determine which player slot is local, then query for `ObjectEnum::DeploymentCenter` or `ObjectEnum::Tunnel` (non-underground check or just first match) with matching `Owner`.
- The system should run in `Startup` schedule, ordered after `setup_gdo_game_start` / `setup_syndicate_game_start` so structures exist to query.
- Player slot index maps to Owner::player(N) — slot 0 is player 0, etc.
- The `MapStartingPositions` resource should be defined in game/world/types.rs alongside other map resources (GridMap, ElevationMap, FogOfWarMap).

## Technical Context

### Existing System to Modify

There is already a `center_camera_on_start` system at `artifacts/developer/src/game/world/faction.rs:202-226` that does the structure fallback logic (step 2-3). This task extends it to also check a `MapStartingPositions` resource first (step 1).

### Files to Change

1. **`artifacts/developer/src/game/world/types.rs`** — Add the `MapStartingPositions` resource definition.
   - Place it near the other map resources (`GridMap` at line 7, `ElevationMap` at line 232, `FogOfWarMap` at line 305).
   - Definition: `HashMap<u8, (i32, i32)>` mapping player slot index to grid coordinates.
   - Derive `Resource, Default` so empty map means no overrides.
   - Pattern to follow: see `ElevationMap` (lines 232-249) for a simple `Resource + Default` type with a `HashMap`.

2. **`artifacts/developer/src/game/world/faction.rs`** — Modify `center_camera_on_start` (lines 202-226) to:
   - Add `map_positions: Res<MapStartingPositions>` parameter.
   - Before the existing structure-fallback logic, check if `map_positions` has an entry for `local_player.0`.
   - If yes, convert grid coords to world coords using `grid_to_world(x, z, 1.0)` (imported from `super::utils`, already imported on line 10), then apply the camera snap formula.
   - If no entry, fall through to the existing structure-based logic.
   - Import `MapStartingPositions` from `super::types`.

3. **`artifacts/developer/src/game/world/mod.rs`** — Register `MapStartingPositions` as a default resource.
   - In `FactionPlugin::build()` (line 61), add `app.init_resource::<types::MapStartingPositions>();` alongside the other `init_resource` calls (lines 62-66).
   - The system ordering at lines 75-79 already runs `center_camera_on_start` after all faction setup systems — no ordering changes needed.

### Camera Snap Formula (used in 3+ places)

```rust
let z_offset = cam_transform.translation.y * 25.0 / 40.0;
cam_transform.translation.x = target_world_x;
cam_transform.translation.z = target_world_z + z_offset;
```

For grid-based input, first convert with `grid_to_world(grid_x, grid_z, 1.0)` (from `game/world/utils.rs:23`). This converts grid (32,32) → world (0.5, 0.0, 0.5), accounting for the 64x64 grid centered at origin with cell_size=1.0.

### Existing center_camera_on_start System (faction.rs:202-226)

```rust
pub fn center_camera_on_start(
    local_player: Res<LocalPlayer>,
    selected_faction: Res<SelectedFaction>,
    structures: Query<(&ObjectInstance, &Owner, &Transform), Without<MainCamera>>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
) {
    let primary_type = match selected_faction.0 {
        FactionEnum::GlobalDefenseOrdinance => ObjectEnum::DeploymentCenter,
        FactionEnum::TheSyndicate => ObjectEnum::Tunnel,
        _ => return,
    };
    let local_owner = Owner::player(local_player.0);
    for (obj, owner, transform) in structures.iter() {
        if obj.object_type == primary_type && *owner == local_owner {
            if let Ok(mut cam_transform) = camera_query.single_mut() {
                let z_offset = cam_transform.translation.y * 25.0 / 40.0;
                cam_transform.translation.x = transform.translation.x;
                cam_transform.translation.z = transform.translation.z + z_offset;
            }
            return;
        }
    }
}
```

### Key Types

- `LocalPlayer(pub u8)` — `shared/types.rs:18`, wraps the player slot index
- `Owner::player(n: u8)` — faction ownership component
- `ObjectInstance { object_type: ObjectEnum, hp, max_hp }` — `game/types/objects.rs:55`
- `ObjectEnum::DeploymentCenter`, `ObjectEnum::Tunnel` — primary structures per faction
- `SelectedFaction(FactionEnum)` — resource set during game setup
- `MainCamera` — marker component for the game camera
- `grid_to_world(grid_x: i32, grid_z: i32, cell_size: f32) -> Vec3` — `game/world/utils.rs:23`

### Existing Tests (faction.rs:2668-2759)

Three existing tests for `center_camera_on_start`:
- `test_center_camera_on_start_gdo` (line 2669) — GDO with DeploymentCenter
- `test_center_camera_on_start_syndicate` (line 2700) — Syndicate with Tunnel
- `test_center_camera_on_start_no_structure` (line 2731) — no structure, camera unchanged

These use `run_system_once(center_camera_on_start)`. After adding the `MapStartingPositions` param, these tests will need `app.init_resource::<MapStartingPositions>()` inserted before the system run (the default empty map preserves their existing behavior). Add new tests for:
- Map position override takes priority over structure
- Map position with no matching slot falls through to structure
- Grid-to-world conversion is applied correctly

### Registration Pattern

The system is already registered in `OnEnter(AppState::InGame)` at `mod.rs:75-79` with proper `.after()` constraints. The `MapStartingPositions` just needs `init_resource` so it exists when the system runs.

## Dependencies

None — this is a standalone task. The existing `center_camera_on_start` system, `grid_to_world` utility, and camera snap formula are all already in the codebase.
