# tile_elevation_rendering

## Metadata
- **From**: task_splitter
- **To**: task_planner

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

2. **Map elevation to Y coordinate** in the Transform. Use a scale factor like `elevation as f32 * ELEVATION_HEIGHT_STEP` where ELEVATION_HEIGHT_STEP is a constant (suggest ~0.1 to 0.15, so max elevation 16 produces Y ~1.6 to 2.4 — visible but not extreme relative to cell_size=1.0). Define a constant (e.g., `const ELEVATION_HEIGHT_STEP: f32 = 0.1;`) in map.rs.

3. **Update the ElevationMap** to store the actual varied elevation values (it already does, just needs the non-zero input).

**Files to modify**: `artifacts/developer/src/game/world/map.rs` (spawn_grid function, add elevation constant and generation logic)

**Testing**: Add a test that verifies tiles of different types get different elevation ranges, and that the elevation is stored in TilePlacement and ElevationMap correctly.
