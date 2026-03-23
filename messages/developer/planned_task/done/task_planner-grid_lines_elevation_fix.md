# grid_lines_elevation_fix

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-fix_grid_lines_elevation.md

## Task

Fix the `draw_grid_lines()` function in `artifacts/developer/src/game/world/map.rs` so that grid lines render at the correct elevation instead of at a hardcoded `y = 0.005`.

### What to change

1. Add `elevation_map: Res<ElevationMap>` parameter to `draw_grid_lines()`
2. Instead of drawing continuous lines spanning the visible range, draw per-cell line segments
3. For each segment, compute Y from the max elevation of the two adjacent cells:
   - Vertical lines (along Z) at grid column `x` between rows `z` and `z+1`: use `max(elevation[x-1,z], elevation[x,z]) * ELEVATION_HEIGHT_STEP + 0.005`
   - Horizontal lines (along X) at grid row `z` between columns `x` and `x+1`: use `max(elevation[x,z-1], elevation[x,z]) * ELEVATION_HEIGHT_STEP + 0.005`
4. Stay within the existing `GRID_LINE_DRAW_RADIUS` camera-culled range

### Acceptance criteria
- Grid lines visible on all tile types (Plane, Rugged Terrain, Cliff, Mountain, Water)
- Grid lines follow terrain elevation contour
- Lines sit flush on tile surfaces (not floating noticeably above)

## Technical Context

### File to modify
- **`artifacts/developer/src/game/world/map.rs`** — the `draw_grid_lines()` function at line 204

### Current implementation (lines 204-261)
The function currently:
- Takes `mut gizmos: Gizmos`, `grid: Res<GridMap>`, `camera_query: Query<&Transform, With<MainCamera>>`
- Uses hardcoded `y = 0.005` for all lines (line 211)
- Draws full-length continuous lines per grid column/row: one `gizmos.line()` call per column spanning `clip_min_z..clip_max_z`, and one per row spanning `clip_min_x..clip_max_x`
- Uses distance-based alpha fading via `grid_line_alpha(dist)` (cubic falloff)

### Key types and resources
- **`ElevationMap`** (Resource, `game/world/types.rs` line 232): `HashMap<(i32,i32), u8>` with `.get(x, z) -> u8` (returns 0 for missing). Already imported via `super::types::*` on line 7.
- **`GridMap`** (Resource, `game/world/types.rs` line 8): `width: u32`, `height: u32`, `cell_size: f32`. Already used by the function.
- **`ELEVATION_HEIGHT_STEP`** (const, map.rs line 69): `0.1f32`. Converts u8 elevation to world Y. Already defined in the same file.

### How to transform the function

1. **Add parameter**: Add `elevation_map: Res<ElevationMap>` to the function signature (line 204-208).

2. **Replace the two line-drawing loops** (lines 230-260). Instead of one line per column/row, iterate over individual cell edges:

   **Vertical lines (along Z):** For each grid column `x` in `min_x..=max_x`, iterate over each row `z` in `min_z..max_z` (not `=max_z`). Each segment goes from cell (x,z) to cell (x,z+1). Compute the Y for this segment:
   ```rust
   let elev_left = elevation_map.get(x as i32 - 1, z as i32) as f32;
   let elev_right = elevation_map.get(x as i32, z as i32) as f32;
   let y = elev_left.max(elev_right) * ELEVATION_HEIGHT_STEP + 0.005;
   ```
   Note: `ElevationMap.get()` uses grid coordinates directly (i32). The grid index `x` (u32, 0..=width) maps to grid coord `x as i32`. The column line at grid index `x` sits between cells at grid coords `x-1` and `x`. Out-of-bounds lookups return 0 (the default), which is correct for border edges.

   World-space endpoints for each segment:
   ```rust
   let wx = x as f32 - half_w;
   let wz_start = z as f32 - half_h;
   let wz_end = (z + 1) as f32 - half_h;
   gizmos.line(Vec3::new(wx, y, wz_start), Vec3::new(wx, y, wz_end), color);
   ```

   **Horizontal lines (along X):** Same pattern but transposed — iterate columns within each row. For row `z`, iterate `x` in `min_x..max_x`:
   ```rust
   let elev_above = elevation_map.get(x as i32, z as i32 - 1) as f32;
   let elev_below = elevation_map.get(x as i32, z as i32) as f32;
   let y = elev_above.max(elev_below) * ELEVATION_HEIGHT_STEP + 0.005;
   ```

3. **Alpha computation**: The distance-based alpha still applies per-line. For segments, compute distance from camera to the segment midpoint (or continue using the column/row distance as-is, which is simpler and visually equivalent since segments are only 1 cell long).

4. **Remove clip_min/clip_max variables** (lines 224-228) — no longer needed since segments are per-cell, not full-span.

### System registration
- `draw_grid_lines` is registered in `WorldPlugin` (`game/world/mod.rs` line 26) in the `Update` schedule. No ordering changes needed — `ElevationMap` is populated during startup (`spawn_grid` at map.rs line 92) and is read-only thereafter.

### Existing tests (map.rs lines 585-699)
- Tests for `grid_line_alpha`, cull range, and clipping are pure math tests that don't call `draw_grid_lines` directly — they won't break.
- No system-level tests for `draw_grid_lines` exist (it uses `Gizmos` which is hard to test).
- Consider adding a unit test that verifies the elevation Y calculation logic: `max(elev_left, elev_right) * ELEVATION_HEIGHT_STEP + 0.005` for a few known elevation pairs.

### Edge cases
- Grid boundaries: column 0 has no cell to its left — `elevation_map.get(-1, z)` returns 0, which is correct (water/border tiles are elevation 0).
- Grid column `width` (64) has no cell to its right — `elevation_map.get(64, z)` returns 0, same reasoning.
- Elevation range: 0-16 (u8), so max Y = 16 * 0.1 + 0.005 = 1.605.

## Dependencies

None — this is a standalone visual fix. The `ElevationMap` resource already exists and is populated during map generation.
