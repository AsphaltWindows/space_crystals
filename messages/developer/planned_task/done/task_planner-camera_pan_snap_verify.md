# camera-pan-snap-verify

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-camera-pan-snap.md

## Task

Verify that camera instant-snap centering is correctly implemented and matches the design spec in artifacts/designer/design/camera.md.

The camera already has two snap-centering code paths:

1. **Alt-click portrait** in `ui/hud.rs` (`selection_portrait_click_system`): Alt+Left-click on a SelectionPortrait centers the camera instantly on the portrait's entity using the z_offset formula `cam.y * 25.0 / 40.0`.

2. **Double-tap control group recall** in `game/world/resources.rs` (`control_group_system`): Double-tapping a control group number key centers the camera on the group centroid using the same z_offset formula.

Both already implement instant snap (no smooth scrolling). The design spec confirms this is correct behavior.

**Verification steps**:
- Confirm both code paths set camera.translation.x and camera.translation.z directly (instant, no lerp/animation).
- Confirm the z_offset formula is consistent between both locations (it is: `cam.y * 25.0 / 40.0`).
- Confirm no smooth panning or animation systems interfere with the snap.
- If any tests can be added to validate snap centering, add them. Otherwise confirm existing tests cover this.
- Run `cargo test` and `cargo build` to ensure no regressions.

## Technical Context

### Code Path 1: Alt-Click Portrait Snap
- **File**: `artifacts/developer/src/ui/hud.rs`, lines 1126-1163
- **System**: `selection_portrait_click_system`
- Registered in `artifacts/developer/src/ui/mod.rs` line 42, runs in `DiagCategory::UiHud` set
- Lines 1155-1157: Direct assignment to `cam_transform.translation.x` and `.z` — no lerp, no animation
- z_offset formula at line 1155: `cam_transform.translation.y * 25.0 / 40.0`
- Existing tests at line 1611+ cover shift-click and ctrl-click portrait behavior but do NOT test Alt-click camera centering. **A unit test for Alt-click snap centering would be a good addition.**

### Code Path 2: Double-Tap Control Group Recall
- **File**: `artifacts/developer/src/game/world/resources.rs`, lines 720-778
- **System**: `control_group_system` (NOT a separate recall system — the recall/centering logic is embedded in `control_group_system`)
- Registered in `artifacts/developer/src/game/world/mod.rs` line 77, runs in `DiagCategory::Faction` set
- Lines 745-747: Direct assignment to `cam_transform.translation.x` and `.z` — no lerp, no animation
- z_offset formula at line 745: `cam_transform.translation.y * 25.0 / 40.0` — **identical** to portrait path
- Existing unit tests at lines 1414-1450: `recall_center_z_offset_default_height`, `recall_center_z_offset_zoomed_in`, `recall_center_z_offset_zero_means_origin`, `recall_center_x_is_exact_centroid` — these validate the z_offset formula at multiple camera heights

### Camera Movement System (potential interference check)
- **File**: `artifacts/developer/src/main.rs`, lines 90-111
- **System**: `camera_movement` — arrow key panning, runs in `DiagCategory::Camera` set
- This is purely additive movement (`+= speed * delta`) gated by arrow key presses — it will NOT override snap-set values unless arrow keys are held in the same frame (expected/acceptable behavior)
- No lerp, smooth follow, or animation systems exist anywhere in the codebase that would fight snap centering

### Design Spec
- **File**: `artifacts/designer/design/camera.md`
- Spec says: "The camera pans via standard pan (currently implemented as instant snap to the target position). When the camera centers on a location — such as via Alt-click portrait in the SelectionPanel — it snaps instantly so the target is at the center of the viewport."
- Both code paths match this spec exactly

### Patterns for Adding Tests
- The z_offset formula tests in resources.rs (lines 1414-1450) are pure math tests (no ECS) — good pattern for any additional formula verification
- For testing the portrait Alt-click snap, you'd need a Bevy `World` with a `MainCamera` entity (Transform), a target entity (Transform), and verify the resulting camera position. The existing portrait tests at hud.rs:1611+ use `World::new()` and manipulate `Selection` directly — follow this pattern.
- Integration tests using `TestApp` (shared/testing/test_app.rs) are possible but overkill for verifying snap behavior — prefer pure math or minimal World tests.

### Key Types
- `MainCamera` — marker component on the camera entity
- `SelectionPortrait` — UI component with `.entity` field pointing to the selected entity
- `Selection` — resource with `groups: Vec<SelectionGroup>`
- `ControlGroups` — resource storing 10 control groups
- `LastRecallState` — tracks last recall for double-tap detection (`.is_double_tap(group_idx, time)`)

## Dependencies

None. This is a standalone verification task. Both snap-centering code paths are already implemented and the task is to confirm correctness and optionally add tests.
