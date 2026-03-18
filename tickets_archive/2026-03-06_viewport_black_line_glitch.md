# Ticket: Fix Horizontal Black Line Visual Glitch at Viewport Boundary

## Current State
A persistent horizontal black line appears near the center of the camera view during gameplay. The likely cause is a sub-pixel rounding gap in the viewport calculation within `update_camera_viewport()` (`src/main.rs:122-147`). The viewport's `physical_position.y` is set using `HUD_TOP_BAR_HEIGHT * scale_factor`, which can produce fractional values that round differently from the UI element height, creating a 1px seam between the HUD bar and the game viewport.

## Desired State
No black line or visual seam is visible at the viewport/HUD boundary. The viewport calculation correctly accounts for sub-pixel rounding so that the game viewport and HUD elements meet seamlessly at all scale factors and window sizes.

## Justification
User-reported visual defect discussed in `forum/horizontal_black_line_visual_glitch.md`. Task_planner and developer both confirmed the viewport calculation as the most likely root cause. The game uses Camera3dBundle, making tile seam issues less likely than viewport/UI boundary artifacts.

## QA Steps
1. Launch the game at the default window size.
2. Observe the area near the center of the camera view (around the HUD top bar boundary).
3. Verify no horizontal black line or seam is visible.
4. Resize the window to a different size (smaller, then larger).
5. Verify no black line appears at any window size.
6. If zoom is supported, zoom in and out — verify no line appears at any zoom level.
7. Pan the camera (if supported) across different map areas — verify the line does not reappear.

## Expected Experience
The game viewport should display a clean, seamless view with no black lines, seams, or visual artifacts at the boundary between the HUD bar and the game world, regardless of window size, zoom level, or camera position.
