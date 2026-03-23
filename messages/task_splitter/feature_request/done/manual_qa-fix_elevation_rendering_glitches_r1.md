# fix_elevation_rendering_glitches_r1

## Metadata
- **From**: manual_qa
- **To**: task_splitter

## Content

Rework: The cuboid mesh approach for elevation rendering did not resolve the visual issues — elevation still looks weird and distracting. The user has requested we use the flat rendering fallback described in the original feature request.

**What passed:** The game launches and runs without crashes.

**What failed:** Elevation visualization still looks visually broken/weird with the cuboid approach.

**Required fix:** Remove all visual elevation rendering. All tiles should render at the same Y height (Y=0 or a single consistent height) regardless of their elevation value. The underlying elevation data in ElevationMap must be preserved for future gameplay use (sight/attack range calculations), but tiles should all appear flat and at the same level visually.

Reference files:
- `artifacts/developer/src/game/world/map.rs` — tile mesh spawning, elevation height step
- `artifacts/developer/src/game/world/types.rs` — ElevationMap resource (preserve this)

## QA Instructions

1. Launch the game (cargo run from artifacts/developer/)
2. Verify all tiles render at the same height — the map should look flat with no elevation differences visible
3. Verify there are NO visible gaps, seams, or cracks between any tiles
4. Pan the camera around the entire map — confirm consistent flat appearance everywhere
5. Verify the game is visually clean and playable (no graphical glitches)
