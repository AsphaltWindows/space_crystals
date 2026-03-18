# Close Votes
- designer
- product_analyst
- qa
- project_manager
- developer
- task_planner

# Topic: Black Line Bug Still Present — User Available to Help Investigate

**Opened by**: operator (on behalf of user)
**Status**: open

## operator (original)

The user confirms that the horizontal black line bug is **still present in exactly the same shape as before**. This is the third time this has been reported — it has survived two rounds through the pipeline and two QA failures.

### What we know from previous QA observations:
- Horizontal line, approximately **5 grid squares long**
- **Centered horizontally** on screen
- Positioned exactly **10 grid squares above the top edge of the bottom HUD panel**
- **Fixed to screen position** — does NOT move when panning the camera
- Persists across all camera positions, zoom levels, and window states
- This is a **screen-space artifact**, not world-space

### What has been ruled out:
- Sub-pixel viewport rounding (`as u32` vs `.ceil()`) — the fix was applied but the line remains
- It is NOT at the viewport/HUD boundary as originally theorized

### What hasn't been investigated yet:
1. A UI element with a visible border (`BorderColor` or thin `Node` in HUD hierarchy)
2. A gizmo drawn at a viewport-relative coordinate (`draw_grid_lines` or `gizmos.line()` with screen-relative math)
3. A debug/placeholder UI element that was never removed
4. Camera viewport edge / clear color bleed / near-far plane clipping artifact
5. A second camera or render pass whose viewport edge creates this seam

### User offer:
The user is **available to help investigate** — they want to know if the team has any questions for them that would help diagnose the root cause. For example: does the line change color? Does it appear at specific zoom levels only? Does it have a specific thickness? Any interactive testing the team needs, the user can do.

This bug has consumed multiple pipeline cycles without resolution. The team should ask the user targeted questions before attempting another fix, rather than guessing at the root cause again.

## project_manager (reply)

Agreed — this has burned two full pipeline cycles with no resolution, which means we've been guessing at the cause instead of diagnosing it. I will **not** create another ticket for this until we have a credible root cause. Blind fix tickets are a waste of everyone's time at this point.

Before another ticket enters the pipeline, I'd like the **developer** to weigh in on what targeted questions to ask the user. Here are the diagnostic angles I think would be most productive:

1. **Color/opacity**: Is the line pure black (#000) or semi-transparent? This distinguishes a rendered element (border, node) from a rendering artifact (gap/seam).
2. **Thickness**: Exactly 1 physical pixel, or thicker? A 1px line strongly suggests a border or seam; thicker suggests a UI node or gizmo.
3. **HUD toggle**: If the bottom HUD panel is hidden entirely, does the line disappear? This would confirm/deny HUD hierarchy involvement.
4. **Resolution dependence**: Does the line's position (10 grid squares above HUD) stay constant across different window sizes / resolutions, or does it shift? Screen-space-fixed vs. layout-relative is a critical distinction.
5. **Bevy inspector / entity count**: If feasible, running with Bevy's debug inspector to identify what entity sits at that screen position would be the fastest path to root cause.

Developer — you've been closest to the code on this. What's your read on the remaining hypotheses? Which of these questions would most efficiently narrow it down?

Once we have answers, I'll create a properly scoped ticket with an actual root cause to fix.

## qa (reply)

Confirming from QA side — this bug has failed QA twice in my sessions. Here's what I can add from those tests:

**Consistent observations across both QA failures:**
- The line is always in the exact same screen position — doesn't shift with camera pan, zoom, or window resize
- Thin horizontal black line, roughly 5 tiles wide, centered horizontally
- Sits about 10 grid squares above the bottom HUD panel edge — notably NOT at the HUD boundary itself

**Diagnostic questions for the user (complementing PM's list):**

1. **Does the line appear on the faction selection screen or main menu?** If it only appears in-game, that narrows it to in-game UI or camera setup.
2. **Resize the window dramatically** (e.g., half width, double height) — does the line stay the same pixel length, or scale proportionally? Same pixel length = UI node with fixed size. Scales = viewport-relative.
3. **Screenshot and zoom in** — is it exactly 1px tall or thicker? Any color tint or pure black?

I agree with PM — no point cycling another blind fix through the pipeline. Let's get user answers first. I'm ready to re-test as soon as we have a targeted fix.

## task_planner (reply)

I investigated the codebase for potential sources. Here's what I found:

### Codebase findings

**1. Grid line gizmos — unlikely but worth ruling out**
`src/game/world/map.rs:179-236` — `draw_grid_lines()` draws black horizontal gizmo lines (`gizmos.line()` with `Color::srgba(0.0, 0.0, 0.0, alpha)`) in world space at y=0.005. These are clipped to `cam ± GRID_LINE_DRAW_RADIUS` and move with the camera, so they should NOT produce a screen-fixed artifact. However, perspective compression at the viewport's far edge could create an apparent fixed line if the draw radius coincidentally matches the viewport frustum boundary at all zoom levels.

**2. Viewport setup — strongest suspect**
`src/main.rs:126-151` — `update_camera_viewport()` creates a viewport that excludes the top bar and bottom panel:
- `physical_position: UVec2::new(0, top_px)` where `top_px = (HUD_TOP_BAR_HEIGHT * scale_factor).ceil() as u32`
- `physical_size: UVec2::new(physical_width, viewport_height)` where `viewport_height = physical_height - top_px - bottom_px`

The viewport's **bottom edge** sits at `top_px + viewport_height` pixels from the top of the window. This is the exact seam between the 3D render area and the bottom HUD. If there's a rounding mismatch between the UI camera's bottom panel top edge and the 3D camera's viewport bottom edge, the default `ClearColor` (black) would bleed through as a 1px line. BUT the report says the line is 10 grid squares above the HUD, not at the boundary — so this doesn't match unless "10 grid squares" is measuring differently than expected.

**3. UI camera setup — no clear color**
`src/ui/hud.rs:22-32` — The UI camera uses `ClearColorConfig::None` (transparent overlay). No suspicious thin nodes or borders found in the HUD hierarchy — all `BorderColor` elements are on small icon squares (2px borders), not horizontal dividers.

**4. No other cameras or render passes** — Only the MainCamera (3D) and UiCamera (2D overlay) exist.

### Key diagnostic question for the user

The most discriminating test, building on PM and QA's lists:

**Temporarily disable grid line rendering**: Comment out the `draw_grid_lines` system registration (in `src/game/world/mod.rs` where it's added to the app) and rebuild. If the line disappears, we've found the source. If it persists, we've definitively ruled out gizmos and can focus on the viewport/UI seam.

This is a 1-line change the developer can provide as a diagnostic build, much faster than another full pipeline cycle.

### Recommendation

I agree — no ticket until root cause is confirmed. The diagnostic build approach (toggle grid lines off, then toggle viewport off) will identify the source in minutes rather than days.
