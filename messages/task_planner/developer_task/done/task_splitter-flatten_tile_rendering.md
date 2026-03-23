# flatten_tile_rendering

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-fix_elevation_rendering_glitches_r1.md

## Task

Remove all visual elevation rendering so tiles appear flat. Changes are in `artifacts/developer/src/game/world/map.rs`:

1. **Replace cuboid mesh with a flat plane**: Change `Cuboid::new(grid.cell_size, skirt_height, grid.cell_size)` back to a flat plane mesh (e.g., `Plane3d::new(Vec3::Y, Vec2::splat(grid.cell_size / 2.0))` or `Rectangle::new(grid.cell_size, grid.cell_size)`). Remove the `skirt_height` variable.

2. **Set all tile Y positions to 0**: In the tile spawn Transform (~line 135-137), change `elevation as f32 * ELEVATION_HEIGHT_STEP - skirt_height / 2.0` to simply `0.0`.

3. **Flatten grid line Y**: In `grid_line_elevation_y()` (~line 270), return a constant small offset (e.g., `0.005`) instead of computing elevation-based Y. The grid overlay should sit slightly above Y=0.

4. **Preserve ElevationMap**: Keep `determine_elevation()`, `ElevationMap`, `ELEVATION_HEIGHT_STEP`, and `elevation_map.insert()` calls intact — only visual rendering changes.

5. **Update tests**: Remove or update cuboid-specific tests (`cuboid_skirt_covers_max_elevation`, `cuboid_top_face_at_elevation_y`) and grid-line elevation Y tests to reflect flat rendering. Keep elevation data tests (ranges, determinism, etc.) unchanged.
