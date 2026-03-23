# flatten_tile_rendering

## Metadata
- **From**: task_planner
- **To**: developer

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

## Technical Context

**File to change:** `artifacts/developer/src/game/world/map.rs` — this is the ONLY file that needs modification. All elevation rendering code is isolated here.

### Current Implementation (lines to change)

1. **Mesh creation (line 100-103):**
   ```rust
   // Current:
   let skirt_height = 2.0_f32;
   let tile_mesh = meshes.add(Cuboid::new(grid.cell_size, skirt_height, grid.cell_size));
   // Replace with a flat plane:
   let tile_mesh = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(grid.cell_size / 2.0)));
   ```
   Remove the `skirt_height` variable entirely and delete the comment on lines 100-101.

2. **Transform Y position (line 135-137):**
   ```rust
   // Current:
   Transform::from_xyz(world_x, elevation as f32 * ELEVATION_HEIGHT_STEP - skirt_height / 2.0, world_z)
   // Replace with:
   Transform::from_xyz(world_x, 0.0, world_z)
   ```

3. **grid_line_elevation_y function (line 270-274):**
   ```rust
   // Current:
   pub fn grid_line_elevation_y(elevation_map: &ElevationMap, ax: i32, az: i32, bx: i32, bz: i32) -> f32 {
       let elev_a = elevation_map.get(ax, az) as f32;
       let elev_b = elevation_map.get(bx, bz) as f32;
       elev_a.max(elev_b) * ELEVATION_HEIGHT_STEP + 0.005
   }
   // Replace body with:
   pub fn grid_line_elevation_y(_elevation_map: &ElevationMap, _ax: i32, _az: i32, _bx: i32, _bz: i32) -> f32 {
       0.005
   }
   ```
   Prefix params with underscore to suppress unused warnings. Keep the function signature so callers don't break.

### Tests to Remove (lines 843-960)

- **Remove** `cuboid_skirt_covers_max_elevation` (line 845-857) — tests cuboid skirt height, no longer relevant
- **Remove** `cuboid_top_face_at_elevation_y` (line 859-876) — tests cuboid top face formula, no longer relevant
- **Update** all `grid_line_elevation_y_*` tests (lines 905-960) — all 5 tests should now assert `y == 0.005` regardless of elevation inputs. Simplest approach: replace the 5 individual tests with a single test:
  ```rust
  #[test]
  fn grid_line_elevation_y_always_flat() {
      let mut elev_map = ElevationMap::default();
      elev_map.insert(3, 5, 10);
      elev_map.insert(10, 10, 16);
      // All calls return constant 0.005 regardless of elevation
      assert!((grid_line_elevation_y(&elev_map, 0, 0, 1, 0) - 0.005).abs() < f32::EPSILON);
      assert!((grid_line_elevation_y(&elev_map, 3, 5, 4, 5) - 0.005).abs() < f32::EPSILON);
      assert!((grid_line_elevation_y(&elev_map, -1, 5, 0, 5) - 0.005).abs() < f32::EPSILON);
      assert!((grid_line_elevation_y(&elev_map, 10, 10, 11, 10) - 0.005).abs() < f32::EPSILON);
  }
  ```

### Tests to KEEP unchanged
- `determine_elevation_*` tests (elevation data tests) — lines ~720-841
- `different_tile_types_have_different_elevation_ranges` (line 878-901)
- Any other non-rendering elevation tests

### Important Notes
- `ELEVATION_HEIGHT_STEP` constant (line 69) should be KEPT — it's used in preserved elevation data tests
- `elevation_map.insert()` call (line 149) must be KEPT — ElevationMap is still populated for future gameplay use
- `determine_elevation()` call (line 130) must be KEPT — it feeds into ElevationMap
- The `elevation` variable (line 130) is still used by `TilePlacement::new()` (line 144) so keep it
- No other files in the project reference `grid_line_elevation_y`, `skirt_height`, or `ELEVATION_HEIGHT_STEP` — this change is fully isolated

## Dependencies

None — this is a standalone visual change isolated to a single file.
