# camera-fixed-zoom

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-scale-camera-system.md

## Task

Rework the camera system in `main.rs` to enforce a fixed 28 GridUnit horizontal view with no zoom controls. Remove `camera_zoom` system, switch to orthographic projection, and add a `camera_projection_sync` system that maintains the 28 GU horizontal constraint accounting for viewport aspect ratio.

## Technical Context

### Files to modify

1. **`artifacts/developer/src/main.rs`** (primary file — all changes here)
   - **Remove `camera_zoom` function** (lines 104-124) entirely
   - **Remove `camera_zoom` from system registration** in `GamePlugin::build()` (line 48)
   - **Add constant**: `const CAMERA_HORIZONTAL_GRID_UNITS: f32 = 28.0;`
   - **Modify `setup()` function** (lines 55-76): Switch from `Camera3d::default()` (perspective) to orthographic projection
   - **Add `camera_projection_sync` system** to keep the projection synchronized with viewport size

2. **`artifacts/developer/src/shared/testing/test_app.rs`** — If the MainCamera spawn changes significantly (e.g., adding projection component), update the dummy camera spawn (line 67-76) to match. The test app currently spawns a minimal `Camera` + `GlobalTransform` + `Transform` + `MainCamera`. If orthographic projection is added as a component, it should be added here too for test parity.

### World coordinate system

- Grid is 64x64, `cell_size = 1.0` (see `GridMap::default()` in `artifacts/developer/src/game/world/types.rs` line 14-22)
- **1 GridUnit = 1.0 world unit** (cell_size is 1.0)
- Therefore 28 GridUnits = 28.0 world units horizontal extent
- Grid is centered at world origin (half_size = 32.0)

### Camera setup pattern (Bevy 0.17)

For an orthographic top-down camera in Bevy 0.17:
```rust
commands.spawn((
    Camera3d::default(),
    OrthographicProjection {
        scaling_mode: ScalingMode::Fixed { width: 28.0, height: initial_height },
        near: -100.0,  // allow seeing below camera Y
        ..OrthographicProjection::default_3d()
    },
    Transform::from_xyz(0.0, 40.0, 25.0)
        .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
    MainCamera,
));
```

**Important**: In Bevy 0.17, `OrthographicProjection` is a separate component (not bundled). Use `OrthographicProjection::default_3d()` as the base (sets `near` to something sensible for 3D). `ScalingMode::Fixed { width, height }` gives exact pixel-independent control. The `near` value may need to be negative or very small since the camera is at Y=40 looking down at Y=0.

### `camera_projection_sync` system

This system should:
1. Query `(&Camera, &mut OrthographicProjection)` with `With<MainCamera>`
2. Read the camera viewport dimensions (from `Camera::logical_viewport_size()` or compute from `Viewport`)
3. Calculate: `aspect_ratio = viewport_width / viewport_height`
4. Set `projection.scaling_mode = ScalingMode::Fixed { width: 28.0, height: 28.0 / aspect_ratio }`
5. Register in `GamePlugin::build()` in the `DiagCategory::Camera` set alongside `camera_movement` and `update_camera_viewport`

The viewport is set by `update_camera_viewport` (excludes HUD_TOP_BAR_HEIGHT=32px and HUD_BOTTOM_PANEL_HEIGHT=220px), so the projection sync should run **after** `update_camera_viewport` to use the correct viewport dimensions. Use `.after(update_camera_viewport)` ordering.

### Existing ray-casting compatibility

Multiple systems use `camera.viewport_to_world(camera_transform, cursor_pos)` for ground-plane intersection:
- `artifacts/developer/src/game/world/map.rs` line 249 (tile click debug)
- `artifacts/developer/src/ui/command_panel.rs` line 190 (placement mode)
- `artifacts/developer/src/game/world/faction.rs` lines 590, 1173 (rally point, building placement)

These all use `Ray3d` with `t = -ray.origin.y / ray.direction.y` intersection. This works correctly with orthographic projection — `viewport_to_world` produces parallel rays with the same Y-direction component. **No changes needed** in these files.

### Camera angle consideration

The current camera is at (0, 40, 25) looking at origin — this is an angled view, not straight top-down. With orthographic projection at this angle, the 28 GU horizontal constraint holds at the ground plane (Y=0) as long as `ScalingMode::Fixed` width = 28.0. The projection width corresponds to the cameras view plane, which maps to ground width since X-axis is perpendicular to the cameras viewing angle in the XZ plane. Verify the X-axis extent matches 28 GU by testing.

### HUD constants (for reference)

From `artifacts/developer/src/ui/types.rs`:
- `HUD_TOP_BAR_HEIGHT: f32 = 32.0` (logical pixels)
- `HUD_BOTTOM_PANEL_HEIGHT: f32 = 220.0` (logical pixels)

### Tests to add

In `main.rs` test module (create `#[cfg(test)] mod tests` at bottom):
- Test that `CAMERA_HORIZONTAL_GRID_UNITS == 28.0`
- Test that `camera_zoom` function no longer exists (this is implicitly verified by compilation — if removed, any reference fails)

## Dependencies

None — this is a standalone camera rework. The camera systems run in `DiagCategory::Camera` which has no ordering dependencies on other DiagCategory sets. The `update_camera_viewport` system already exists and will continue to set the viewport; the new `camera_projection_sync` just needs to run after it.
