# elevation_cuboid_meshes

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-fix_elevation_rendering_glitches.md

## Task

Replace flat Plane3d tile meshes with Cuboid meshes in `src/game/world/map.rs` to eliminate visible gaps/seams between adjacent tiles at different elevation heights.

**Current code** (line 100): A single shared `Plane3d` mesh is created and reused for all tiles:
```rust
let tile_mesh = meshes.add(Plane3d::default().mesh().size(grid.cell_size, grid.cell_size));
```
Tiles are positioned at varying Y heights (line 132):
```rust
Transform::from_xyz(world_x, elevation as f32 * ELEVATION_HEIGHT_STEP, world_z)
```
This creates visible gaps between adjacent tiles of different elevations because flat planes have no vertical sides.

**Required changes**:

1. Replace `Plane3d` with `Cuboid` meshes. Each tile needs a cuboid whose top face sits at its elevation Y, with vertical sides (skirt) extending downward to cover potential gaps. A reasonable skirt depth is the maximum possible elevation difference (16 * ELEVATION_HEIGHT_STEP = 1.6 world units), so `Cuboid::new(cell_size, skirt_height, cell_size)` where skirt_height covers from below zero up to the tile's elevation.

2. Adjust the Transform Y so the **top face** of the cuboid aligns with the tile's elevation Y. Since Cuboid is centered at origin, offset Y by +half_height.

3. The mesh can no longer be a single shared handle if heights vary — either create one mesh per distinct elevation level, or use a single cuboid tall enough for the maximum elevation and position it so its top face is correct (simpler approach — a single tall cuboid with top face at elevation Y, extending well below).

**Simplest approach**: Create one Cuboid mesh with height = max_elevation_height + margin (e.g., 2.0 units). Position each tile so its top face is at `elevation * ELEVATION_HEIGHT_STEP`. The bottom will extend below Y=0, which is fine since the camera never sees below the grid.

**Do NOT change**: determine_elevation(), ELEVATION_HEIGHT_STEP, TilePlacement, ElevationMap, or any elevation gameplay values. This is a rendering-only fix.

## Technical Context

### File to modify
- **`artifacts/developer/src/game/world/map.rs`** — `spawn_grid()` function, lines 93-153

### Specific change sites

**Line 100** — Replace `Plane3d` mesh creation:
```rust
// BEFORE:
let tile_mesh = meshes.add(Plane3d::default().mesh().size(grid.cell_size, grid.cell_size));

// AFTER (simplest approach — single tall cuboid):
let skirt_height = 2.0_f32; // covers max elevation (1.6) + margin
let tile_mesh = meshes.add(Cuboid::new(grid.cell_size, skirt_height, grid.cell_size));
```

**Line 132** — Adjust Transform Y to place the cuboid's top face at the elevation height:
```rust
// BEFORE:
Transform::from_xyz(world_x, elevation as f32 * ELEVATION_HEIGHT_STEP, world_z)

// AFTER: Cuboid is centered at origin, so offset down by half the skirt height
// so the top face sits at the elevation Y
let elevation_y = elevation as f32 * ELEVATION_HEIGHT_STEP;
Transform::from_xyz(world_x, elevation_y - skirt_height / 2.0, world_z)
```

Note: `skirt_height` must be accessible at both line 100 and line 132. Define it as a local variable at the top of `spawn_grid()` or as a const near `ELEVATION_HEIGHT_STEP` (line 69).

### Existing patterns to follow
- `Cuboid::new(x, y, z)` is the standard mesh primitive used throughout the codebase (see `game/utils.rs` — every structure/unit uses it). No import changes needed; `Cuboid` is in `bevy::prelude::*`.
- The single shared `tile_mesh` handle with `.clone()` on line 130 still works — all tiles share the same cuboid geometry, only Transform differs.

### Constants reference
- `ELEVATION_HEIGHT_STEP = 0.1` (map.rs:69) — world Y per elevation unit
- `MAX_ELEVATION = 16` (types.rs:129) — max elevation value
- Max world height = 16 * 0.1 = 1.6 units — skirt of 2.0 provides safe margin

### No other files need changes
- No tests reference `Plane3d` or the specific mesh type
- `TilePlacement`, `ElevationMap`, `determine_elevation()` are untouched
- Grid line drawing (`draw_grid_lines`) is gizmo-based, unaffected by tile mesh changes

## Dependencies

None. This is a self-contained rendering fix within `spawn_grid()`. No other systems depend on the tile mesh type.
