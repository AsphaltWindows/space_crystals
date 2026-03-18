# Ticket: Fix Horizontal Black Line Visual Glitch at Viewport Boundary

## Current State
A persistent horizontal black line appears near the center of the camera view during gameplay. The likely cause is a sub-pixel rounding gap in the viewport calculation within `update_camera_viewport()` (`src/main.rs:122-147`). The viewport's `physical_position.y` is set using `HUD_TOP_BAR_HEIGHT * scale_factor`, which can produce fractional values that round differently from the UI element height, creating a 1px seam between the HUD bar and the game viewport.

## Desired State
No black line or visual seam is visible at the viewport/HUD boundary. The viewport calculation correctly accounts for sub-pixel rounding so that the game viewport and HUD elements meet seamlessly at all scale factors and window sizes.

## Justification
User-reported visual defect discussed in `forum/horizontal_black_line_visual_glitch.md`. Task_planner and developer both confirmed the viewport calculation as the most likely root cause. The game uses Camera3dBundle, making tile seam issues less likely than viewport/UI boundary artifacts.

## QA Steps
1. [auto] Launch the game at the default window size.
2. [human] Observe the area near the center of the camera view (around the HUD top bar boundary).
3. [human] Verify no horizontal black line or seam is visible.
4. [human] Resize the window to a different size (smaller, then larger).
5. [human] Verify no black line appears at any window size.
6. [human] If zoom is supported, zoom in and out — verify no line appears at any zoom level.
7. [human] Pan the camera (if supported) across different map areas — verify the line does not reappear.

## Expected Experience
The game viewport should display a clean, seamless view with no black lines, seams, or visual artifacts at the boundary between the HUD bar and the game world, regardless of window size, zoom level, or camera position.

## Technical Context

### Root Cause
The bug is in `update_camera_viewport()` at `src/main.rs:122-147`. The viewport boundary calculations use `as u32` truncation:

```rust
let top_px = (ui::types::HUD_TOP_BAR_HEIGHT * scale_factor) as u32;  // line 132
let bottom_px = (ui::types::HUD_BOTTOM_PANEL_HEIGHT * scale_factor) as u32;  // line 133
```

`as u32` truncates (floors) the float. Bevy's UI layout engine rounds `Val::Px(32.0)` to physical pixels using its own rounding logic (typically `round()`). When `scale_factor` produces a fractional result (e.g., `32.0 * 1.25 = 40.0` is fine, but `32.0 * 1.1 = 35.2` truncates to 35 while UI may round to 35), the 3D camera viewport and the HUD bar don't perfectly abut — leaving a 1px gap where the 3D camera's clear color (black/dark) shows through.

### Architecture
- **3D game camera**: `Camera3dBundle` with `MainCamera` marker, order 0 (default), spawned in `setup()` at `src/main.rs:48-55`. Viewport restricted to exclude HUD regions.
- **2D UI camera**: `Camera2dBundle` with `UiCamera` marker, order 1 (renders on top), spawned in `setup_hud()` at `src/ui/hud.rs:14-24`. Full window, `ClearColorConfig::None` (transparent overlay).
- **HUD top bar**: Spawned in `setup_hud()` at `src/ui/hud.rs:28-48`. Uses `height: Val::Px(HUD_TOP_BAR_HEIGHT)` (32.0 logical px), `position_type: PositionType::Absolute`, `top: Val::Px(0.0)`.
- **Constants**: `HUD_TOP_BAR_HEIGHT = 32.0` and `HUD_BOTTOM_PANEL_HEIGHT = 220.0` in `src/ui/types.rs:5-8`.

### Files to Modify
1. **`src/main.rs:132-133`** — Fix the rounding in `update_camera_viewport()`. Change `as u32` truncation to `.ceil() as u32` for `top_px` (ensures viewport starts at or below the HUD bar bottom edge, eliminating any gap). For `bottom_px`, also use `.ceil() as u32` to avoid a similar gap at the bottom.

### Fix Approach
Replace:
```rust
let top_px = (ui::types::HUD_TOP_BAR_HEIGHT * scale_factor) as u32;
let bottom_px = (ui::types::HUD_BOTTOM_PANEL_HEIGHT * scale_factor) as u32;
```
With:
```rust
let top_px = (ui::types::HUD_TOP_BAR_HEIGHT * scale_factor).ceil() as u32;
let bottom_px = (ui::types::HUD_BOTTOM_PANEL_HEIGHT * scale_factor).ceil() as u32;
```

Using `.ceil()` ensures the viewport always starts at or 1px below the HUD bar edge — the HUD's opaque background covers any overlap, so a 1px overlap is invisible, but a 1px gap shows the black clear color. Ceiling eliminates the gap.

**Alternative**: Use `.round() as u32` instead of `.ceil()`. This matches Bevy's typical UI rounding but could still produce a 0.5px-rounds-down gap. `.ceil()` is safer — it guarantees no gap at the cost of potentially 1px of the game viewport being hidden behind the HUD bar (invisible since the bar is opaque).

### No Test Impact
This is a visual rendering fix — no unit tests to write. QA steps cover verification.

## QA FAILURE — 2026-03-06

**Failed step**: Step 3 — black line still visible
**Observed**: The black line is NOT at the viewport/HUD boundary. The ticket's root cause analysis is WRONG. The actual black line has these characteristics:
- Horizontal, exactly **5 grid squares long**
- Stays **centered horizontally** on screen
- Positioned exactly **10 grid squares above the bottom HUD panel**
- Does NOT move with camera pan (stays fixed relative to viewport)

This is NOT a sub-pixel viewport rounding issue. The line appears to be a rendered element (possibly a debug gizmo, a misplaced UI element, or an artifact from grid line drawing) at a fixed screen-relative position. The `.ceil()` rounding fix is irrelevant to this bug. Root cause needs to be re-investigated — search for anything that draws a fixed-length horizontal line or spawns geometry at a viewport-relative position.

## QA Observation — 2026-03-08 (confirmed still present)

User re-confirmed exact position:
- **Horizontal line, ~5 grid spaces long**
- **Centered horizontally** on screen
- **Exactly 10 grid squares above the top edge of the bottom HUD panel** (the panel containing minimap, info panel, selection panel, and control panels)
- **Fixed to screen position** — does not move when panning the camera
- Persists across all camera positions, zoom levels, and window states

This is a screen-space artifact, not world-space. Key investigation leads:
1. A UI element with a visible border/line (check for `BorderColor` or thin `Node` elements in the HUD hierarchy)
2. A gizmo drawn at a viewport-relative coordinate (check `draw_grid_lines` or any system using `gizmos.line()` with screen-relative math)
3. A debug/placeholder UI element that was never removed
4. **Camera viewport artifact** — the 3D camera viewport is manually restricted via `update_camera_viewport()` to exclude HUD regions. The viewport boundary itself, clear color bleed at the viewport edge, or a near/far plane clipping artifact could produce a visible line at a fixed screen position. Check if the line aligns with a viewport boundary calculation. Also check if there's a second camera or render pass whose viewport edge creates this seam.

## Automated QA Results (2026-03-08)
- Step 1 [auto]: PASS — Game launches successfully, GridMap resource initialized with valid dimensions.
- Steps 2-7 [human]: DEFERRED to human review — visual verification of black line glitch.

## QA Failure — 2026-03-09
**Step 2-3**: FAIL — Black line is still present. Previously characterized as 5 tiles long, centered horizontally, ~10 grid squares above bottom HUD. The fix did not resolve the visual glitch.

## Dependencies
None — this is a standalone bug fix with no dependency on other tasks.
