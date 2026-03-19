# Designer Review: Visual Bug Fix & QA Infrastructure

## Metadata
- **Created by**: operator
- **Created**: 2026-03-19T10:04:00Z
- **Status**: open

## Close Votes

## Discussion

### [operator] 2026-03-19T10:04:00Z

The following items cover a persistent visual glitch and a QA infrastructure improvement. Neither has been implemented yet. **Designer**: please review and produce `feature_request` messages (or confirm these are purely technical and can go directly to development without design input).

---

### 1. Viewport Black Line Visual Glitch

A persistent horizontal black line appears during gameplay with these characteristics:
- Horizontal, exactly **5 grid squares long**
- Stays **centered horizontally** on screen
- Positioned exactly **10 grid squares above the bottom HUD panel** (the panel containing minimap, info panel, selection panel, and control panels)
- **Fixed to screen position** -- does not move when panning the camera
- Persists across all camera positions, zoom levels, and window states

**Previous fix attempts (all failed):**
- Original root cause analysis blamed viewport sub-pixel rounding in `update_camera_viewport()`. The `.ceil()` fix was applied but did NOT resolve the glitch.
- The line is NOT at the viewport/HUD boundary as originally hypothesized.
- This is a screen-space artifact, not world-space.

**Investigation leads (unresolved):**
1. A UI element with a visible border/line (check for BorderColor or thin Node elements in the HUD hierarchy)
2. A gizmo drawn at a viewport-relative coordinate
3. A debug/placeholder UI element that was never removed
4. Camera viewport artifact -- the 3D camera viewport boundary, clear color bleed, or a near/far plane clipping artifact

**QA Steps:**
1. [auto] Launch the game at the default window size.
2. [human] Observe the area near the center of the camera view (around 10 grid squares above the bottom HUD).
3. [human] Verify no horizontal black line or seam is visible.
4. [human] Resize the window to a different size -- verify no black line at any window size.
5. [human] Zoom in and out -- verify no line at any zoom level.
6. [human] Pan the camera across different map areas -- verify the line does not reappear.

---

### 2. Automated QA Re-tagging of UI Tasks

Pure infrastructure task: review 8 UI-focused QA task files and change `[human]`/`[semi]` tags to `[auto]` on steps that verify deterministic ECS-queryable UI state (button visibility, slot assignments, interface state transitions, info panel content). Steps requiring visual verification remain `[human]`.

**Re-tagging rules:**
- **Change to [auto]**: steps checking button presence/absence at specific grid slots, interface state transitions, info panel showing correct entity data, ActiveGroup highlighting on correct unit type, command panel slot layout verification
- **Keep as [human]**: steps requiring visual rendering verification, UX feel assessment, rally point marker visibility, right-click interaction observation, rapid input testing

**Affected task areas (8 total):**
1. Barracks interface state (3 steps -> [auto])
2. Supply Tower interface state (5 steps -> [auto])
3. DC default state cancel commands (7 steps -> [auto])
4. DC/EF no auto-enter construction submenu (4 steps -> [auto])
5. Standard bottom row commands (5 steps -> [auto])
6. Info panel stale on control group switch (2 steps -> [auto])
7. Basic combat unit interface state (steps checking grid slots -> [auto])
8. Selection panel (steps checking ActiveGroup highlight -> [auto])

**Dependency:** Requires automated_qa_ui_state_queries to be completed first (provides assert_command_visible, assert_interface_state, assert_info_panel_shows, assert_active_group_type, get_selection_panel_portraits, get_visible_commands).

**No Rust code changes** -- markdown file editing only.

**QA Steps:**
1. [semi] Open each re-tagged file. Verify every step checking button presence/slot assignment is tagged [auto]. Verify every step requiring visual verification remains [human].
2. [auto] Count [auto] tags across all modified files -- verify each file has at least 1 [auto] step.
3. [semi] Spot-check 2-3 re-tagged [auto] steps: confirm the check described maps to a UI State Query API method.

---

### Notes for designer:
- The black line glitch is a technical debugging issue. The designer may want to simply confirm it's a priority and let it go to development for investigation, rather than producing a detailed feature_request.
- The QA re-tagging is purely pipeline infrastructure. The designer may want to confirm the tagging rules are appropriate (which checks can be automated vs. need human eyes) but this doesn't require design decisions.
- If these are purely technical, the designer can acknowledge and delegate to the developer with minimal feature_request overhead.
