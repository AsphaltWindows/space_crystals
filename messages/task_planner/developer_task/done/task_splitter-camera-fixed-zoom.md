# camera-fixed-zoom

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-scale-camera-system.md

## Task

Rework the camera system in `main.rs` to enforce a fixed 28 GridUnit horizontal view with no zoom controls.

**What exists now:**
- Camera spawns at (0, 40, 25) looking at origin with Camera3d + MainCamera marker
- `camera_movement` system: arrow key panning (works, keep as-is)
- `camera_zoom` system: Q/E key zoom (MUST BE REMOVED — spec says no zoom controls)
- `update_camera_viewport` system: adjusts viewport to exclude HUD top bar and bottom panel (works, keep as-is)

**What needs to change:**

1. **Remove `camera_zoom` system entirely** — delete the function and remove it from the GamePlugin system registration. The spec explicitly says no zoom in/out controls.

2. **Add a constant** `CAMERA_HORIZONTAL_GRID_UNITS: f32 = 28.0` — the camera must always display exactly 28 GridUnits horizontally.

3. **Switch to orthographic projection** (or compute the correct perspective height) so that the ground plane (Y=0) shows exactly 28 GridUnits horizontally. With an orthographic camera this means setting the projection scaling area width to `28.0 * SPACE_UNITS_PER_GRID_UNIT` (if world coords use SpaceUnits) or `28.0` (if world coords use GridUnits). Check how existing code maps GridUnits to world coordinates and be consistent.

4. **Add a `camera_projection_sync` system** that runs every frame (or on window resize) to recalculate the vertical extent based on: viewport aspect ratio = (viewport_width / viewport_height), then vertical_extent = horizontal_extent / aspect_ratio. This ensures the vertical GridUnit coverage adjusts with aspect ratio and HUD size.

5. **Camera should look straight down** at the ground plane (top-down RTS view) or at an angle — preserve the existing look-at angle but ensure the 28 GridUnit horizontal constraint holds at the ground plane (Y=0).

**Key files:**
- `artifacts/developer/src/main.rs` — GamePlugin, camera systems
- `artifacts/developer/src/simulation/mod.rs` — SPACE_UNITS_PER_GRID_UNIT constant
- `artifacts/developer/src/ui/types.rs` — HUD_TOP_BAR_HEIGHT, HUD_BOTTOM_PANEL_HEIGHT

**Tests to add:**
- Unit test verifying CAMERA_HORIZONTAL_GRID_UNITS == 28.0
- Integration test verifying camera_zoom system no longer exists (no zoom on Q/E/scroll)
