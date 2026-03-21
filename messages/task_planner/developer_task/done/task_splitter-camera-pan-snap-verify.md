# camera-pan-snap-verify

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-camera-pan-snap.md

## Task

Verify that camera instant-snap centering is correctly implemented and matches the design spec in artifacts/designer/design/camera.md.

The camera already has two snap-centering code paths:

1. **Alt-click portrait** in `ui/hud.rs` (`selection_portrait_click_system`): Alt+Left-click on a SelectionPortrait centers the camera instantly on the portrait's entity using the z_offset formula `cam.y * 25.0 / 40.0`.

2. **Double-tap control group recall** in `game/world/resources.rs` (`control_group_recall_system`): Double-tapping a control group number key centers the camera on the group centroid using the same z_offset formula.

Both already implement instant snap (no smooth scrolling). The design spec confirms this is correct behavior.

**Verification steps**:
- Confirm both code paths set camera.translation.x and camera.translation.z directly (instant, no lerp/animation).
- Confirm the z_offset formula is consistent between both locations (it is: `cam.y * 25.0 / 40.0`).
- Confirm no smooth panning or animation systems interfere with the snap.
- If any tests can be added to validate snap centering, add them. Otherwise confirm existing tests cover this.
- Run `cargo test` and `cargo build` to ensure no regressions.
