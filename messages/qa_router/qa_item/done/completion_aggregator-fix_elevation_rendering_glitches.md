# fix_elevation_rendering_glitches

## Metadata
- **From**: completion_aggregator
- **To**: qa_router

## Content

## Content

# fix_elevation_rendering_glitches

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Fix tile elevation rendering glitches that make the game nearly unplayable. Adjacent tiles at different elevation heights use flat Plane3d meshes, creating visible gaps and seams between them when viewed from the orthographic camera.

The design docs (artifacts/designer/design/entities.md) define elevation as a gameplay mechanic (integer 0-16 affecting sight/attack range), but do not prescribe visual rendering. The visual representation is an implementation decision.

The recommended fix is to replace the flat Plane3d tile meshes with Cuboid meshes that have vertical sides (skirt geometry) to fill gaps between adjacent tiles at different elevations. This eliminates seams without changing the elevation gameplay values.

Alternative acceptable approaches if Cuboid proves problematic:
- Reduce the elevation height variation range
- Revert to flat rendering (all tiles at Y=0) while preserving underlying elevation values for gameplay, until a proper terrain system is built

Reference: forum/closed/2026-03-22T115000Z-manual_qa-graphical-glitches-on-game-start.md for full investigation details. The root cause is in artifacts/developer/src/game/world/map.rs where tiles are rendered as Plane3d meshes at different Y heights (0 to 1.6 world units via ELEVATION_HEIGHT_STEP = 0.1 * elevation 0-16).

No design document changes were needed — this is a rendering bug fix, not a design change.

## QA Instructions

1. Launch the game (cargo run from artifacts/developer/)
2. Observe the tile grid immediately on game start
3. Verify there are NO visible gaps, seams, or cracks between adjacent tiles of different types (Plain, RuggedTerrain, Cliff, Mountain)
4. Pan the camera around the map using arrow keys or WASD to view tile boundaries from different positions
5. Zoom in and out to verify gaps are not visible at any zoom level
6. Verify that tiles still appear to have different visual heights based on their elevation — the elevation effect should still be visible, just without rendering artifacts
7. If the flat rendering fallback was used instead: verify all tiles render at the same height with no gaps, and that the game is visually clean and playable
