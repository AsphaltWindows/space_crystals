# fix_grid_lines_elevation

## Metadata
- **From**: qa_router
- **To**: manual_qa

## Content

## Content

## Content

## Grid Lines Hidden by Tile Elevation — Regression Fix

Grid lines are no longer visible in-game because the elevation system (commit c5792d1) raises tile geometry above the hardcoded grid line Y position.

### Problem
`draw_grid_lines()` in `artifacts/developer/src/game/world/map.rs` (line ~204) draws all grid lines at a hardcoded `y = 0.005`. The elevation system renders tiles at `elevation * 0.1` world-space Y, so even the lowest non-water tile (Plane, elevation 2, surface at y=0.2) completely occludes the grid lines.

### Required Fix
Each grid line segment must be drawn at the Y position matching the elevation of the tiles it borders. The `ElevationMap` resource (defined in `artifacts/developer/src/game/world/types.rs`) is already populated during `spawn_grid` and contains per-cell elevation data.

For each grid line segment between two adjacent cells, use the maximum elevation of the bordering tiles plus a small offset (e.g., `max_neighbor_elevation * 0.1 + 0.005`) so lines sit just above the terrain surface. This ensures grid lines follow the terrain contour and remain visible on all tile types.

### Relevant Files
- `artifacts/developer/src/game/world/map.rs` — `draw_grid_lines()`, `ELEVATION_HEIGHT_STEP`, `determine_elevation()`
- `artifacts/developer/src/game/world/types.rs` — `ElevationMap` resource

### Design Context
No design document changes needed. The design docs (`entities.md`, `scale.md`) define tiles by gameplay properties and grid units but do not specify elevation rendering details — elevation is an implementation-level visual feature. Grid lines should always be visible on all tile types.

## QA Instructions

1. Launch the game and observe the map
2. Verify grid lines are visible on **all** tile types: Plane, Rugged Terrain, Cliff, Mountain, and Water
3. Verify grid lines follow the terrain elevation — lines on higher tiles should appear higher than lines on lower tiles
4. Verify grid lines sit flush on tile surfaces (not floating noticeably above)
5. Check tile boundaries between different elevation levels — lines at elevation transitions should connect smoothly or follow the higher tile's elevation
6. Compare before/after: grid lines should look similar to how they appeared before the elevation system was added, but now respecting terrain height
