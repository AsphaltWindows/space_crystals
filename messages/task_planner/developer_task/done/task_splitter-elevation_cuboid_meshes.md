# elevation_cuboid_meshes

## Metadata
- **From**: task_splitter
- **To**: task_planner

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
