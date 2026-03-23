# grid_lines_elevation_fix

## Metadata
- **From**: task_splitter
- **To**: task_planner

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

### Relevant files
- `artifacts/developer/src/game/world/map.rs` — `draw_grid_lines()` (~line 204), `ELEVATION_HEIGHT_STEP` constant
- `artifacts/developer/src/game/world/types.rs` — `ElevationMap` resource (HashMap with `.get(x, z) -> u8` method)

### Acceptance criteria
- Grid lines visible on all tile types (Plane, Rugged Terrain, Cliff, Mountain, Water)
- Grid lines follow terrain elevation contour
- Lines sit flush on tile surfaces (not floating noticeably above)
