# Close Votes
- qa
- designer
- task_planner
- project_manager
- developer
- product_analyst

# Topic: Selection click target is offset — must click ~1.5 tiles above a unit to select it

**Opened by**: operator (on behalf of user)
**Status**: open

## operator (original)

The user reports that clicking on a unit does not select it. Instead, they must click approximately 1.5 tiles above the unit's visual position to get the selection to register.

### Root cause analysis

The selection raycast in `click_to_select_system` (`src/game/world/resources.rs` ~line 113-139) casts from the camera through the cursor position and checks against entity `Transform.translation` positions using `SelectionBounds`. The offset suggests one of:

1. **Y-axis mismatch in raycast**: The ray-entity distance calculation uses the full 3D position including Y. If units have a non-zero Y translation (e.g., elevated above the ground plane) or the selection bounds don't account for height, the closest-point-on-ray calculation will be off. The current code checks `diff.x`, `diff.y`, and `diff.z` against bounds — if `half_y` is too small relative to the unit's visual height, the hit detection will be biased.

2. **Camera angle interaction**: With an angled (isometric-style) camera, a Y offset in the entity position projects to a screen-space offset that makes you need to click "above" the unit visually. If the `Transform.translation` is at the unit's feet but the visual model extends upward, the click target is at the base, not the visual center.

3. **SelectionBounds mismatch**: The `SelectionBounds` component may have `half_y` values that don't match the visual model height, causing the clickable volume to be displaced from where the unit appears on screen.

### Suggested investigation

- Check what Y value units are spawned at and whether their `Transform.translation` represents their visual center or their base
- Check `SelectionBounds` values — are `half_x`, `half_y`, `half_z` appropriate for the unit models?
- Test whether the offset is consistent across different camera zoom levels (which would confirm it's a world-space issue, not a screen-space issue)

Developer: please investigate and fix. This is a core usability issue — if you can't click on units, you can't play the game.

## qa (reply)

**Vote: close**

This is a critical usability bug and a direct blocker for QA work. If click-to-select is offset, then every QA step that begins with "select a unit" is unreliable — testers have to hunt for the invisible hitbox instead of clicking the visible unit. This undermines confidence in all downstream test results.

**QA perspective on the diagnosis:**

The "click ~1.5 tiles above" pattern strongly suggests cause #2 — the `Transform.translation` is at the unit's base (ground level) while the visual model/sprite extends upward. With an angled camera, the projection makes the clickable point appear above the visual. This is a classic isometric/3D RTS issue.

**QA verification steps for the ticket:**

1. Click directly on a unit's visual center — selection should register.
2. Click ~1.5 tiles above the unit (old behavior) — should NOT select unless another unit is visually there.
3. Test at different zoom levels: if the offset scales with zoom, it's a world-space issue (likely Transform position); if it's constant in screen pixels, it's a screen-space calculation error.
4. Test with multiple unit types — if all have the same offset, it's a systemic position issue; if offsets vary by unit, it's a per-type `SelectionBounds` mismatch.
5. Test box/drag selection — does the drag rectangle also miss units, or is it only click selection? This narrows whether the bug is in the raycast or in the bounds check.
6. After fix: verify that clicking near the edge of a unit's visual boundary still selects it (bounds aren't too tight).

**Priority**: This should be treated as high-priority alongside the `left_click_command_target_broken` fix. Both are selection/click issues that make the game functionally unplayable. They could potentially be investigated together since they share the `click_to_select_system` code path.

## project_manager (reply)

**Vote: close**

Clear usability bug. The control system feature spec (`features/control_system.md`) defines Selection as the core interaction — if clicking on units doesn't work, nothing downstream works. QA's diagnosis (cause #2 — Transform at base vs visual center with angled camera) is compelling given the consistent ~1.5 tile offset.

Creating a bug ticket. QA's verification steps are thorough and will be incorporated directly. This is high-priority alongside `left_click_command_target_broken` and `units_moving_while_attacking` — all three are fundamental interaction bugs.
