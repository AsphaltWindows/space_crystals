# tile_elevation_rendering

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-tile_terrain_system_r1.md

## Task

Fix tile elevation rendering so tiles at different elevations appear at visibly different heights in the 3D scene.

**Current state**: In `spawn_grid` (game/world/map.rs), all tiles are assigned `elevation = 0u8` (line 109) and spawned at `Transform::from_xyz(world_x, 0.0, world_z)` (line 114). The TilePlacement component stores elevation 0-16 but the value is always 0 and never affects the Y coordinate.

**What to implement**:

1. **Assign varied elevation values** based on tile type and position. A reasonable approach:
   - Water tiles: elevation 0
   - Plane tiles: elevation in range ~2-5 (gentle variation)
   - RuggedTerrain tiles: elevation in range ~4-8
   - Cliff tiles: elevation in range ~8-12
   - Mountain tiles: elevation in range ~10-16
   Use the existing `simple_hash` or similar deterministic function for per-tile variation within each type's range.

2. **Map elevation to Y coordinate** in the Transform. Use a scale factor like `elevation as f32 * ELEVATION_HEIGHT_STEP` where ELEVATION_HEIGHT_STEP is a constant (suggest ~0.1 to 0.15, so max elevation 16 produces Y ~1.6 to 2.4 -- visible but not extreme relative to cell_size=1.0). Define a constant (e.g., `const ELEVATION_HEIGHT_STEP: f32 = 0.1;`) in map.rs.

3. **Update the ElevationMap** to store the actual varied elevation values (it already does, just needs the non-zero input).

**Files to modify**: `artifacts/developer/src/game/world/map.rs` (spawn_grid function, add elevation constant and generation logic)

**Testing**: Add a test that verifies tiles of different types get different elevation ranges, and that the elevation is stored in TilePlacement and ElevationMap correctly.

## Technical Context

### Primary file to modify

**`artifacts/developer/src/game/world/map.rs`** -- this is the only file that needs changes.

#### `spawn_grid` function (line 75-135)
- Line 109: `let elevation = 0u8;` -- replace with a call to a new `determine_elevation()` function
- Line 114: `Transform::from_xyz(world_x, 0.0, world_z)` -- change `0.0` to `elevation as f32 * ELEVATION_HEIGHT_STEP`
- Line 119: `TilePlacement::new(tile_type, grid_pos, elevation)` -- already correct, will use the new non-zero value
- Line 124: `elevation_map.insert(grid_pos.x, grid_pos.z, elevation)` -- already correct, will store the new value

#### New function: `determine_elevation(tile_type: TilePresetEnum, x: u32, z: u32) -> u8`
- Pattern to follow: mirrors `determine_tile_type()` (line 12) -- takes grid coordinates, returns a value
- Use the existing `simple_hash(x, z)` function (line 61) for deterministic per-tile variation within each type's elevation range
- Elevation ranges per type (must stay within 0..=16 per `MAX_ELEVATION` in types.rs line 129):
  - `TilePresetEnum::Water` -> 0
  - `TilePresetEnum::Plane` -> 2..=5 (hash % 4 + 2)
  - `TilePresetEnum::RuggedTerrain` -> 4..=8 (hash % 5 + 4)
  - `TilePresetEnum::Cliff` -> 8..=12 (hash % 5 + 8)
  - `TilePresetEnum::Mountain` -> 10..=16 (hash % 7 + 10)

#### New constant: `ELEVATION_HEIGHT_STEP`
- Add near the top of map.rs or near the other grid constants (line 140 area)
- Suggested value: `0.1` (elevation 16 => Y=1.6, visible but not extreme vs cell_size=1.0)
- This is a **public** constant -- other systems may need it for placing entities at correct Y heights

### Key types involved

- **`TilePresetEnum`** (types.rs:38): `Plane`, `RuggedTerrain`, `Cliff`, `Mountain`, `Water` -- used as match discriminant
- **`TilePlacement`** (types.rs:133): component with `tile_type`, `location`, `elevation: u8` fields. `TilePlacement::new()` validates elevation <= `MAX_ELEVATION` (16)
- **`ElevationMap`** (types.rs:232): resource with `HashMap<(i32, i32), u8>`. `.insert(x, z, elevation)` and `.get(x, z)` methods. Already consumed by combat systems for elevation modifiers
- **`GridMap`** (types.rs): resource with `width`, `height`, `cell_size` fields
- **`simple_hash(x, z)`** (map.rs:61): deterministic spatial hash -- returns `u32`, use modulo to map to elevation ranges

### Downstream consumers (read-only context, DO NOT modify)

The `ElevationMap` is already consumed by multiple combat systems for attack range modifiers:
- `attack_phase_system` (combat/systems/core.rs:53)
- `turret_autonomous_scanning_system` (combat/systems/core.rs:280)
- `base_auto_target_system` (combat/systems/core.rs:379)
- `attack_move_retarget_system` (combat/systems/core.rs:481)
- Multiple combat behavior systems (combat/systems/behaviors.rs)

These all call `elevation_map.get(x, z)` and `elevation_modifier()`. They will automatically benefit from non-zero elevation values -- no changes needed.

The fog-of-war system (map.rs:292) has a NOTE comment acknowledging it doesn't use elevation yet -- this is out of scope.

### Grid line rendering consideration

`draw_grid_lines` (map.rs:179) draws at `y = 0.005` (line 186). With elevated tiles, grid lines will appear below elevated terrain. This is a known cosmetic issue but is OUT OF SCOPE for this task -- grid lines can be addressed separately if needed.

### Testing approach

Existing test module starts at line 453. Add tests that:
1. Verify `determine_elevation()` returns values in expected ranges for each tile type
2. Verify Water always returns 0
3. Verify determinism (same x,z,type -> same elevation)
4. Optionally verify the constant value `ELEVATION_HEIGHT_STEP` is defined and positive

No ECS/App test harness needed -- `determine_elevation` is a pure function like `determine_tile_type`.

## Dependencies

None -- this is the sole task in the tile_terrain_system_r1 feature. All types (`TilePlacement`, `ElevationMap`, `TilePresetEnum`, `MAX_ELEVATION`, `simple_hash`) already exist. No other tasks produce or consume outputs needed here.
