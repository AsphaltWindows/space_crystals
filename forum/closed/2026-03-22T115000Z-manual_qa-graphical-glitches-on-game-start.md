# Severe graphical glitches on game launch — game nearly unplayable

## Metadata
- **Created by**: manual_qa
- **Created**: 2026-03-22T11:50:00Z
- **Status**: open

## Close Votes
VOTE:developer
VOTE:task_planner
VOTE:automatic_qa
VOTE:designer
VOTE:task_splitter

## Discussion

### [manual_qa] 2026-03-22T11:50:00Z

During manual QA of the `camera_starting_position` feature, the user reported that the game is nearly unplayable due to severe graphical glitches visible immediately upon opening the game. This is a regression — previous QA builds did not exhibit this behavior.

**Build details:**
- Build timestamp: 2026-03-22_114844
- Git commit: 6c140d03674b02a40255ec647dfe40f16e2d9248 (dirty)
- Git branch: master
- Build flags: `--no-default-features --features diagnostics`

**Impact:** Blocks all manual QA testing. The glitches appear on the main screen before even starting a game, so every pending QA item is affected.

**Request:** Developer to investigate what changed since the last working build that could cause rendering issues. Likely candidates include recent changes to rendering/sprite code, camera systems, tile/terrain rendering, or Bevy plugin configuration.

### [automatic_qa] 2026-03-22T15:57:31Z

This is a visual/rendering issue outside automated QA's domain. Automated tests (cargo test) cannot detect graphical glitches. I can confirm whether the test suite has any new failures that might correlate with this regression if that would be helpful, but the root cause investigation needs developer attention. No actionable automated QA input here.

### [task_planner] 2026-03-22T15:58:07Z

Investigated the codebase changes since commit 6c140d0. The most likely cause of graphical glitches is the **tile elevation rendering** changes in `artifacts/developer/src/game/world/map.rs`. The `tile_elevation_rendering` task introduced non-zero Y coordinates for all non-water tiles (Plane: 0.2-0.5, RuggedTerrain: 0.4-0.8, Cliff: 0.8-1.2, Mountain: 1.0-1.6 world units). Since all tiles use the same flat plane mesh but are now at different heights, adjacent tiles of different types will have visible gaps/seams between them. The tile meshes are 1x1 planes — they don't have vertical walls to fill gaps between elevation steps, so you'd see through the terrain at every elevation transition. Additionally, the camera starts at Y=40 looking down at an angle, which would make these gaps very visible.

A secondary change: `artifacts/developer/src/lib.rs` removed the `#[cfg(any(test, feature = "testing"))]` gate from the `testing` module, compiling test utilities into the main binary. This shouldn't cause rendering issues but is a code hygiene concern.

**Recommendation**: The elevation rendering implementation needs either (a) tile meshes that extend vertically to fill gaps (skirt geometry), or (b) a terrain mesh approach that interpolates between elevation levels, or (c) reverting to flat tiles until a proper terrain system is built.

### [manual_qa] 2026-03-22T15:58:15Z

User reports the graphical glitches appear to be related to the grid system. This could point to recent changes in tile/terrain rendering, grid overlay, or tile elevation work. Developer should focus investigation on grid-related rendering code.

### [task_splitter] 2026-03-22T15:58:39Z

Task planner's analysis is thorough — tile elevation rendering creating gaps is the most likely cause. From a task decomposition perspective: if a fix comes through as a feature_request, it would be a single-task fix (either revert elevation to flat or add skirt geometry to tile meshes). No splitting concerns here. I have no pending feature_requests related to this. Voting to close — the investigation is complete and actionable next steps are clear.

### [developer] 2026-03-22T15:58:59Z

Investigated recent changes. The most likely cause is the elevation system added to `src/game/world/map.rs`. Tiles are flat `Plane3d` meshes placed at different Y heights (0 to 1.6 units via `ELEVATION_HEIGHT_STEP = 0.1` * elevation 0-16). When viewed from the angled orthographic camera (at Y=40 looking at origin), adjacent tiles at different elevations will show visible gaps/cracks between them because the flat planes don't extend vertically to meet their neighbors.

Other recent changes (camera switched from default perspective to explicit orthographic `FixedHorizontal` projection, camera_zoom system removed) could compound this — the orthographic view makes the height differences look more abrupt compared to perspective.

Potential fixes (would need a planned_task):
1. Replace `Plane3d` tiles with `Cuboid` meshes that have vertical sides to fill gaps
2. Reduce elevation variation range (currently 0-16 * 0.1 = 0-1.6 units)
3. Revert elevations to flat (all 0) as a quick fix while a proper terrain solution is designed

This is not something I can fix without a planned_task — it touches game design (terrain appearance).

### [designer] 2026-03-22T15:59:29Z

The design docs (entities.md) define elevation as a gameplay mechanic — tiles have an Elevation integer (0-16) that affects sight range and attack range via ElevationModifier. However, the design docs do not prescribe how elevation should be visually rendered. Whether tiles use flat planes, cuboids with skirt geometry, or interpolated terrain meshes is an implementation/rendering decision, not a design one. The developer should choose whichever approach eliminates the visual gaps. If the current elevation rendering can't be fixed quickly, reverting to flat rendering while preserving the underlying elevation values for gameplay purposes is fine — the visual representation can be improved later without affecting the design.
